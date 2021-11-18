
use std::collections::HashMap;

use crate::{ast::{Identifier, expression::{BinaryData, CallData, ExprType, ExprVisitor, Expression, IndexData, LambdaData, LiteralData, LogicData, UnaryData}, statement::{AssignData, Block, DeclarationData, IfData, StmtVisitor}}, interpreter::globals::Globals, utils::{result::{ErrorList, Result}, source_pos::SourcePos}};

type SymbolTable = HashMap<String, usize>;

pub struct Resolver {
	last_id: usize,
	tables: Vec<SymbolTable>,
	globals: Globals,
	is_overriding: bool,
}

impl Resolver {

	pub fn new(globals: Globals) -> Self {
		Resolver {
			last_id: globals.ids.len() + 1,
			globals: globals.clone(),
			tables: vec![globals.ids],
			is_overriding: false,
		}
	}

	pub fn resolve(&mut self, block: &Block) -> Result<()> {
		let mut errors = ErrorList::empty();

		self.push_scope();
		for stmt in block.clone() {
			errors.try_append(stmt.accept(self));
		}
		self.pop_scope();

		errors.if_empty(())
	}

	fn add(&mut self, iden: Identifier, pos: SourcePos) -> Result<()> {
		if self.globals.ids.contains_key(&iden.get_name()) {
			return ErrorList::comp(format!("Cannot redefine global constant '{}'", iden), pos).err();
		}

		*iden.id.borrow_mut() = self.last_id;
		self.tables.last_mut().unwrap().insert(iden.get_name(), iden.get_id());
		self.last_id += 1;
		Ok(())
	}

	fn push_scope(&mut self) {
		self.tables.push(SymbolTable::new());
	}

	fn pop_scope(&mut self) {
		self.tables.pop();
	}

	fn get_var(&self, name: &str) -> Option<usize> {
		let mut cur = self.tables.as_slice();
		while let [rest @ .., table] = cur {
			match table.get(name) {
				Some(&id) => return Some(id),
				None => cur = rest,
			}
		}
		None
	}

}

impl ExprVisitor<()> for Resolver {

	fn literal(&mut self, data: LiteralData, _pos: SourcePos) -> Result<()> {
		let exprs = match data {
			LiteralData::List(exprs) => exprs,
			LiteralData::Template(exprs) => exprs,
			_ => return Ok(()),
		};

		let mut errors = ErrorList::empty();

		for expr in exprs {
			errors.try_append(expr.accept(self));
		}

		errors.if_empty(())
	}
	
	fn binary(&mut self, data: BinaryData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		errors.try_append(data.lhs.accept(self));
		errors.try_append(data.rhs.accept(self));
		errors.if_empty(())
	}
	
	fn unary(&mut self, data: UnaryData, _pos: SourcePos) -> Result<()> {
		data.expr.accept(self)
	}
	
	fn logic(&mut self, data: LogicData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		errors.try_append(data.lhs.accept(self));
		errors.try_append(data.rhs.accept(self));
		errors.if_empty(())
	}
	
	fn grouping(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<()> {
		data.accept(self)
	}
	
	fn variable(&mut self, data: Identifier, pos: SourcePos) -> Result<()> {
		if self.globals.ids.contains_key(&data.name) {
			*data.id.borrow_mut() = self.globals.ids.get(&data.name).unwrap().clone();
			Ok(())
		} else {
			match self.get_var(&data.name) {
				Some(id) => {
					if self.is_overriding && id < self.globals.ids.len() {
						ErrorList::comp(format!("Cannot assign to global constant '{}'", data), pos).err()
					} else {
						*data.id.borrow_mut() = id;
						Ok(())
					}
				},
				None => ErrorList::comp(format!("Use of undefined variable '{}'", data), pos).err()
			}
		}
	}
	
	fn lambda(&mut self, data: LambdaData, pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		for param in data.params {
			errors.try_append(self.add(param, pos));
		}
		errors.try_append(self.resolve(&data.body));
		errors.if_empty(())
	}
	
	fn call(&mut self, data: CallData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		errors.try_append(data.calee.accept(self));
		for arg in data.args {
			errors.try_append(arg.accept(self));
		}
		errors.if_empty(())
	}
	
	fn index(&mut self, data: IndexData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		errors.try_append(data.head.accept(self));
		errors.try_append(data.index.accept(self));
		errors.if_empty(())
	}

}

impl StmtVisitor<()> for Resolver {

	fn expr(&mut self, expr: Box<Expression>, _pos: SourcePos) -> Result<()> {
		expr.accept(self)
	}
	
	fn declaration(&mut self, data: DeclarationData, pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		if let ExprType::Lambda(_) = data.expr.typ {
			errors.try_append(self.add(data.name, pos));
			errors.try_append(data.expr.accept(self));
		} else {
			errors.try_append(data.expr.accept(self));
			errors.try_append(self.add(data.name, pos));
		}
		errors.if_empty(())
	}
	
	fn assignment(&mut self, data: AssignData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		self.is_overriding = true;
		errors.try_append(data.head.accept(self));
		self.is_overriding = false;
		errors.try_append(data.expr.accept(self));
		errors.if_empty(())
	}
	
	fn if_stmt(&mut self, data: IfData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		errors.try_append(data.cond.accept(self));
		errors.try_append(self.resolve(&data.then_block));
		errors.try_append(self.resolve(&data.else_block));
		errors.if_empty(())
	}
	
	fn loop_stmt(&mut self, block: Block, _pos: SourcePos) -> Result<()> {
		self.resolve(&block)
	}
	
	fn break_stmt(&mut self, _pos: SourcePos) -> Result<()> { Ok(()) }
	
	fn continue_stmt(&mut self, _pos: SourcePos) -> Result<()> { Ok(()) }
	
	fn return_stmt(&mut self, expr: Box<Expression>, _pos: SourcePos) -> Result<()> {
		expr.accept(self)
	}

}
