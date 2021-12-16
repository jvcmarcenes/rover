
use std::collections::HashMap;

use crate::{ast::{identifier::Identifier, expression::{BinaryData, CallData, ExprType, ExprVisitor, Expression, FieldData, IndexData, LambdaData, LiteralData, LogicData, UnaryData, BindData}, statement::{AssignData, Block, DeclarationData, IfData, StmtVisitor, AttrDeclarationData}}, utils::{result::{ErrorList, Result}, source_pos::SourcePos, global_ids::get_global_identifiers}};

macro_rules! with_ctx {
	($self:ident, $block:expr, $ctx:ident: $val:expr) => {{
		let prev = $self.ctx.clone();
		$self.ctx.$ctx = $val;
		let res = $block;
		$self.ctx = prev;
		res
	}};
	($self:ident, $block:expr, $head:ident: $val_head:expr, $($tail:ident: $val_tail:expr),*) => {{
		let prev = $self.ctx.clone();
		$self.ctx.$head = $val_ead;
		with_ctx!($block, $($tail: $val_tail),*)
		$self.ctx = prev;
	}};
}

#[derive(Clone, Debug)]
pub struct IdentifierData {
	id: usize,
	constant: bool,
}

impl IdentifierData {
	pub fn new(id: usize, constant: bool) -> Self {
		Self { id, constant }
	}
}

type SymbolTable = HashMap<String, IdentifierData>;

fn allowed(cond: bool, msg: &str, pos: SourcePos) -> Result<()> {
	if cond { Ok(()) }
	else { ErrorList::comp(msg.to_owned(), pos).err() }
}

#[derive(Clone, Debug, Default)]
struct Context {
	in_function: bool,
	in_loop: bool,
	// in_obj: bool,
	// in_method: bool,
	overwriting: bool,
}

#[derive(Debug)]
pub struct Resolver {
	last_id: usize,
	tables: Vec<SymbolTable>,
	globals: SymbolTable,
	ctx: Context,
}

impl Resolver {
	
	pub fn new() -> Self {
		let globals = get_global_identifiers();

		Resolver {
			last_id: globals.len() + 1,
			globals: globals.clone(),
			tables: vec![globals],
			ctx: Context::default(),
		}
	}
	
	pub fn resolve(&mut self, block: &Block) -> Result<()> {
		let mut errors = ErrorList::new();
		
		// dbg!(&block);
		
		self.push_scope();
		for stmt in block.clone() {
			errors.try_append(stmt.accept(self));
		}
		self.pop_scope();
		
		errors.if_empty(())
	}
	
	fn add(&mut self, iden: Identifier, constant: bool, pos: SourcePos) -> Result<()> {
		if self.globals.contains_key(&iden.get_name()) {
			return ErrorList::comp(format!("Cannot redefine global constant '{}'", iden), pos).err();
		}
		
		*iden.id.borrow_mut() = self.last_id;
		// eprintln!("DEBUG: {} > {}: {}", self.tables.len(), iden.get_name(), iden.get_id());
		self.tables.last_mut().unwrap().insert(iden.get_name(), IdentifierData::new(iden.get_id(), constant));
		self.last_id += 1;
		Ok(())
	}
	
	fn push_scope(&mut self) {
		self.tables.push(SymbolTable::new());
	}
	
	fn pop_scope(&mut self) {
		self.last_id -= self.tables.last().unwrap().len();
		self.tables.pop();
	}
	
	fn get_var(&self, name: &str) -> Option<IdentifierData> {
		let mut cur = self.tables.as_slice();
		while let [rest @ .., table] = cur {
			match table.get(name) {
				Some(id) => return Some(id.clone()),
				None => cur = rest,
			}
		}
		None
	}
	
}

impl ExprVisitor<()> for Resolver {
	
	fn literal(&mut self, data: LiteralData, pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		
		let exprs = match data {
			LiteralData::List(exprs) => exprs,
			LiteralData::Template(exprs) => exprs,
			LiteralData::Object(map, attrs) => {
				let exprs = map.clone().into_values();
				// with_ctx!(self, for expr in exprs { errors.try_append(expr.accept(self)); }, in_obj: true);
				for expr in exprs { errors.try_append(expr.accept(self)); };
				for attr in attrs {
					if let Some(var) = self.get_var(&attr.get_name()) {
						*attr.id.borrow_mut() = var.id;
					} else {
						errors.add_comp(format!("Use of undefined attribute {}", attr.get_name()), pos);
					}
				}
				return errors.if_empty(());
			}
			LiteralData::Error(val) => return val.accept(self),
			_ => return Ok(()),
		};
		
		for expr in exprs {
			errors.try_append(expr.accept(self));
		}
		
		errors.if_empty(())
	}
	
	fn binary(&mut self, data: BinaryData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		errors.try_append(data.lhs.accept(self));
		errors.try_append(data.rhs.accept(self));
		errors.if_empty(())
	}
	
	fn unary(&mut self, data: UnaryData, _pos: SourcePos) -> Result<()> {
		data.expr.accept(self)
	}
	
	fn logic(&mut self, data: LogicData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		errors.try_append(data.lhs.accept(self));
		errors.try_append(data.rhs.accept(self));
		errors.if_empty(())
	}
	
	fn grouping(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<()> {
		data.accept(self)
	}
	
	fn variable(&mut self, data: Identifier, pos: SourcePos) -> Result<()> {
		if let Some(var) = self.get_var(&data.name) {
			if self.ctx.overwriting {
				if var.id < self.globals.len() {
					return ErrorList::comp(format!("Cannot assign to global constant '{}'", data), pos).err();
				} else if var.constant {
					return ErrorList::comp(format!("Cannot assign to constant '{}'", data), pos).err();
				}
			}
			
			*data.id.borrow_mut() = var.id;
			Ok(())
		} else {
			ErrorList::comp(format!("Use of undefined variable '{}'", data), pos).err()
		}
	}
	
	fn lambda(&mut self, data: LambdaData, pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		self.push_scope();
		for param in data.params {
			errors.try_append(self.add(param, false, pos));
		}
		with_ctx!(self, errors.try_append(self.resolve(&data.body)), in_function: true);
		self.pop_scope();
		errors.if_empty(())
	}
	
	fn call(&mut self, data: CallData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		errors.try_append(data.calee.accept(self));
		for arg in data.args {
			errors.try_append(arg.accept(self));
		}
		errors.if_empty(())
	}
	
	fn index(&mut self, data: IndexData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		errors.try_append(data.head.accept(self));
		with_ctx!(self, errors.try_append(data.index.accept(self)), overwriting: false);
		errors.if_empty(())
	}
	
	fn field(&mut self, data: FieldData, _pos: SourcePos) -> Result<()> {
		data.head.accept(self)
	}
	
	fn self_ref(&mut self, pos: SourcePos) -> Result<()> {
		allowed(self.ctx.in_function, "Invalid self expression", pos)
	}
	
	fn do_expr(&mut self, block: Block, _pos: SourcePos) -> Result<()> {
		self.resolve(&block)
	}
	
	fn bind_expr(&mut self, data: BindData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		errors.try_append(data.expr.accept(self));
		errors.try_append(data.method.accept(self));
		errors.if_empty(())
	}

}

impl StmtVisitor<()> for Resolver {
	
	fn expr(&mut self, expr: Box<Expression>, _pos: SourcePos) -> Result<()> {
		expr.accept(self)
	}
	
	fn declaration(&mut self, data: DeclarationData, pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		match data.expr.typ.clone() {
			ExprType::Lambda(_) | ExprType::Literal(LiteralData::Object(_, _)) => {
				errors.try_append(self.add(data.name, data.constant, pos));
				errors.try_append(data.expr.accept(self));
			},
			_ => {
				errors.try_append(data.expr.accept(self));
				errors.try_append(self.add(data.name, data.constant, pos));
			}
		}
		errors.if_empty(())
	}
	
	fn attr_declaration(&mut self, data: AttrDeclarationData, pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		errors.try_append(self.add(data.name, true, pos));
		for method in data.methods {
			self.push_scope();
			for param in method.params {
				errors.try_append(self.add(param, false, pos));
			}
			// with_ctx!(self, errors.try_append(self.resolve(&method.body)), in_method: true);
			with_ctx!(self, errors.try_append(self.resolve(&method.body)), in_function: true);
			self.pop_scope();
		}
		for expr in data.fields.into_values() {
			errors.try_append(expr.accept(self))
		}
		for attr in data.attributes {
			if let Some(var) = self.get_var(&attr.get_name()) {
				*attr.id.borrow_mut() = var.id;
			} else {
				errors.add_comp(format!("Use of undefined attribute {}", attr.get_name()), pos);
			}
		}
		errors.if_empty(())
	}
	
	fn assignment(&mut self, data: AssignData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		with_ctx!(self, errors.try_append(data.head.accept(self)), overwriting: true);
		errors.try_append(data.expr.accept(self));
		errors.if_empty(())
	}
	
	fn if_stmt(&mut self, data: IfData, _pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		errors.try_append(data.cond.accept(self));
		errors.try_append(self.resolve(&data.then_block));
		errors.try_append(self.resolve(&data.else_block));
		errors.if_empty(())
	}
	
	fn loop_stmt(&mut self, block: Block, _pos: SourcePos) -> Result<()> {
		with_ctx!(self, self.resolve(&block), in_loop: true)
	}
	
	fn break_stmt(&mut self, pos: SourcePos) -> Result<()> {
		allowed(self.ctx.in_loop, "Invalid break statement", pos)
	}
	
	fn continue_stmt(&mut self, pos: SourcePos) -> Result<()> {
		allowed(self.ctx.in_loop, "Invalid continue statement", pos)
	}
	
	fn return_stmt(&mut self, expr: Box<Expression>, pos: SourcePos) -> Result<()> {
		let mut errors = ErrorList::new();
		errors.try_append(allowed(self.ctx.in_function, "Invalid return statement", pos));
		errors.try_append(expr.accept(self));
		errors.if_empty(())
	}
	
	fn scoped_stmt(&mut self, block: Block, _pos: SourcePos) -> Result<()> {
		self.resolve(&block)
	}
	
}
