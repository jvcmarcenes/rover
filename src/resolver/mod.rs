
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{ast::{Identifier, expression::{BinaryData, CallData, ExprType, ExprVisitor, Expression, FieldData, IndexData, LambdaData, LiteralData, LogicData, UnaryData}, statement::{AssignData, Block, DeclarationData, IfData, StmtVisitor}}, interpreter::globals::Globals, utils::{result::{ErrorList, Result}, source_pos::SourcePos}};

type SymbolTable = HashMap<String, usize>;

fn allowed(cond: bool, msg: &str, pos: SourcePos) -> Result<()> {
	if cond { Ok(()) }
	else { ErrorList::comp(msg.to_owned(), pos).err() }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
	allow_return: bool,
	allow_break: bool,
	allow_continue: bool,
	allow_self: bool,
	self_binding: Option<usize>,
	assignment: bool,
}

#[derive(Debug)]
pub struct Resolver {
	last_id: usize,
	tables: Vec<SymbolTable>,
	globals: Globals,
	ctx: Context,
}

impl Resolver {

	pub fn new(globals: Globals) -> Self {
		Resolver {
			last_id: globals.ids.len() + 1,
			globals: globals.clone(),
			tables: vec![globals.ids],
			ctx: Context::default(),
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
		let mut errors = ErrorList::empty();

		let exprs = match data {
			LiteralData::List(exprs) => exprs,
			LiteralData::Template(exprs) => exprs,
			LiteralData::Object(map) => {
				let prev = self.ctx;
				self.ctx.allow_self = true;
				let exprs = map.into_values().collect::<Vec<_>>();
				for expr in exprs {
					errors.try_append(expr.accept(self));
				}
				self.ctx = prev;
				return errors.if_empty(());
			},
			_ => return Ok(()),
		};

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
			return Ok(());
		}
		
		if let Some(id) = self.get_var(&data.name) {
			if self.ctx.assignment && id < self.globals.ids.len() {
				ErrorList::comp(format!("Cannot assign to global constant '{}'", data), pos).err()
			} else {
				*data.id.borrow_mut() = id;
				Ok(())
			}
		} else {
			ErrorList::comp(format!("Use of undefined variable '{}'", data), pos).err()
		}
	}
	
	fn lambda(&mut self, data: LambdaData, pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		for param in data.params {
			errors.try_append(self.add(param, pos));
		}
		let prev = self.ctx;
		self.ctx.allow_return = true;
		errors.try_append(self.resolve(&data.body));
		self.ctx = prev;
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

	fn field(&mut self, data: FieldData, _pos: SourcePos) -> Result<()> {
		data.head.accept(self)
	}

	fn self_ref(&mut self, data: Rc<RefCell<usize>>, pos: SourcePos) -> Result<()> {
		allowed(self.ctx.allow_self && self.ctx.self_binding.is_some(), "Invalid self expression", pos)?;
		*data.borrow_mut() = self.ctx.self_binding.unwrap();
		Ok(())
	}

}

impl StmtVisitor<()> for Resolver {

	fn expr(&mut self, expr: Box<Expression>, _pos: SourcePos) -> Result<()> {
		expr.accept(self)
	}
	
	fn declaration(&mut self, data: DeclarationData, pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		match data.expr.typ {
			ExprType::Lambda(_) => {
					errors.try_append(self.add(data.name, pos));
					errors.try_append(data.expr.accept(self));
			},
			ExprType::Literal(LiteralData::Object(_)) => {
				let name = data.name.name.clone();
				errors.try_append(self.add(data.name, pos));
				let prev = self.ctx;
				self.ctx.self_binding = self.get_var(&name);
				errors.try_append(data.expr.accept(self));
				self.ctx = prev;
			},
			_ => {
				errors.try_append(data.expr.accept(self));
				errors.try_append(self.add(data.name, pos));
			}
		}
		errors.if_empty(())
	}
	
	fn assignment(&mut self, data: AssignData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		self.ctx.assignment = true;
		errors.try_append(data.head.accept(self));
		self.ctx.assignment = false;
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
		let prev = self.ctx;
		self.ctx.allow_break = true;
		self.ctx.allow_continue = true;
		let res = self.resolve(&block);
		self.ctx = prev;
		res
	}
	
	fn break_stmt(&mut self, pos: SourcePos) -> Result<()> {
		allowed(self.ctx.allow_break, "Invalid break statement", pos)
	}
	
	fn continue_stmt(&mut self, pos: SourcePos) -> Result<()> {
		allowed(self.ctx.allow_continue, "Invalid continue statement", pos)
	}
	
	fn return_stmt(&mut self, expr: Box<Expression>, pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::empty();
		errors.try_append(allowed(self.ctx.allow_return, "Invalid return statement", pos));
		errors.try_append(expr.accept(self));
		errors.if_empty(())
	}

}
