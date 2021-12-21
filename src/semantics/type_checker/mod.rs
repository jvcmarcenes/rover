
#![allow(unused_variables)]

use std::collections::HashMap;

use crate::{ast::{expression::*, statement::*, identifier::Identifier}, types::{Type, TypePrim}, utils::{source_pos::SourcePos, result::{Result, ErrorList}, wrap::Wrap}};

pub struct TypeChecker {
	type_map: HashMap<usize, Type>,
	infer: bool,
}

impl TypeChecker {
	
	pub fn new(infer: bool,) -> Self {
		Self {
			type_map: HashMap::new(),
			infer,
		}
	}
	
	fn check_block(&mut self, block: &Block) -> Result<Type> {
		let mut errors = ErrorList::new();
		let mut typ = Type::Void;
		for stmt in block.clone() {
			match stmt.accept(self) {
				Ok(Type::Void) => (),
				Ok(typr) => typ = typr,
				Err(err) => errors.append(err),
			}
		}
		errors.if_empty(typ)
	}
	
	pub fn check(&mut self, block: &Block) -> Result<()> {
		self.check_block(&block)?;
		Ok(())
	}
	
}

impl ExprVisitor<Type> for TypeChecker {
	
	fn literal(&mut self, data: LiteralData, pos: SourcePos) -> Result<Type> {
		match data {
			LiteralData::None => Type::Primitive(TypePrim::None),
			LiteralData::Str(_) => Type::Primitive(TypePrim::Str),
			LiteralData::Num(_) => Type::Primitive(TypePrim::Num),
			LiteralData::Bool(_) => Type::Primitive(TypePrim::Bool),
			_ => todo!(),
		}.wrap()
	}
	
	fn binary(&mut self, data: BinaryData, pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		let mut typ = None;
		let lhs_typ = match data.lhs.accept(self) { Ok(typ) => typ, Err(err) => { errors.append(err); Type::Void } };
		let rhs_typ = match data.rhs.accept(self) { Ok(typ) => typ, Err(err) => { errors.append(err); Type::Void } };
		if !errors.is_empty() { return errors.err(); }
		let expect = match data.op {
			BinaryOperator::Add => Type::Or(vec![Type::Primitive(TypePrim::Num), Type::Primitive(TypePrim::Str)]),
			BinaryOperator::Sub => Type::Primitive(TypePrim::Num),
			BinaryOperator::Mul => Type::Primitive(TypePrim::Num),
			BinaryOperator::Div => Type::Primitive(TypePrim::Num),
			BinaryOperator::Rem => Type::Primitive(TypePrim::Num),
			BinaryOperator::Equ => { typ = Some(Type::Primitive(TypePrim::Bool)); Type::Primitive(TypePrim::Any) },
			BinaryOperator::Neq => { typ = Some(Type::Primitive(TypePrim::Bool)); Type::Primitive(TypePrim::Any) },
			BinaryOperator::Lst => { typ = Some(Type::Primitive(TypePrim::Bool)); Type::Primitive(TypePrim::Num) },
			BinaryOperator::Lse => { typ = Some(Type::Primitive(TypePrim::Bool)); Type::Primitive(TypePrim::Num) },
			BinaryOperator::Grt => { typ = Some(Type::Primitive(TypePrim::Bool)); Type::Primitive(TypePrim::Num) },
			BinaryOperator::Gre => { typ = Some(Type::Primitive(TypePrim::Bool)); Type::Primitive(TypePrim::Num) },
			BinaryOperator::Typ => todo!(),
		};
		let typ = if expect.accepts(&lhs_typ)? && expect.accepts(&rhs_typ)? {
			if let Some(typ) = typ { typ }
			else if rhs_typ == Type::Primitive(TypePrim::Str) { rhs_typ }
			else { lhs_typ }
		} else {
			errors.add_comp(format!("Illegal operation for types '{}' and '{}' expected '{}'", lhs_typ, rhs_typ, expect), pos);
			Type::Void
		};
		errors.if_empty(typ)
	}
	
	fn unary(&mut self, data: UnaryData, pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		let mut typ = Type::Void;
		match data.expr.accept(self) {
			Ok(expr_typ) => {
				let expect = match data.op {
					UnaryOperator::Not => Type::Primitive(TypePrim::Any),
					UnaryOperator::Pos => Type::Primitive(TypePrim::Num),
					UnaryOperator::Neg => Type::Primitive(TypePrim::Num),
				};
				if expect.accepts(&expr_typ)? {
					typ = expect;
				} else {
					errors.add_comp(format!("Illegal operation for '{}', expected '{}'", expr_typ, expect), pos);
				}
			}
			Err(err) => errors.append(err),
		}
		errors.if_empty(typ)
	}
	
	fn logic(&mut self, data: LogicData, pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		errors.try_append(data.lhs.accept(self));
		errors.try_append(data.rhs.accept(self));
		errors.if_empty(Type::Primitive(TypePrim::Bool))
	}
	
	fn grouping(&mut self, data: Box<Expression>, pos: SourcePos) -> Result<Type> {
		data.accept(self)
	}
	
	fn variable(&mut self, data: Identifier, pos: SourcePos) -> Result<Type> {
		self.type_map.get(&data.get_id()).unwrap().clone().wrap()
	}
	
	fn lambda(&mut self, data: LambdaData, pos: SourcePos) -> Result<Type> {
		todo!()
	}
	
	fn call(&mut self, data: CallData, pos: SourcePos) -> Result<Type> {
		Ok(Type::Primitive(TypePrim::Any))
	}
	
	fn index(&mut self, data: IndexData, pos: SourcePos) -> Result<Type> {
		todo!()
	}
	
	fn field(&mut self, data: FieldData, pos: SourcePos) -> Result<Type> {
		todo!()
	}
	
	fn self_ref(&mut self, pos: SourcePos) -> Result<Type> {
		todo!()
	}
	
	fn do_expr(&mut self, block: Block, pos: SourcePos) -> Result<Type> {
		match self.check_block(&block)? {
			Type::Void => ErrorList::comp("do block did not evaluate to a value".to_owned(), pos).err(),
			typ => typ.wrap()
		}
	}
	
	fn bind_expr(&mut self, data: BindData, pos: SourcePos) -> Result<Type> {
		todo!()
	}
	
}

impl StmtVisitor<Type> for TypeChecker {
	
	fn expr(&mut self, expr: Box<Expression>, pos: SourcePos) -> Result<Type> {
		expr.accept(self)
	}
	
	fn declaration(&mut self, data: DeclarationData, pos: SourcePos) -> Result<Type> {
		let expr_typ = data.expr.accept(self)?;
		if let Some(typ) = data.type_restriction {
			self.type_map.insert(data.name.get_id(), typ.clone());
			if !typ.accepts(&expr_typ)? {
				return ErrorList::comp(format!("Cannot assign '{}' to '{}'", expr_typ, typ), pos).err();
			}
		} else {
			self.type_map.insert(data.name.get_id(), if self.infer { expr_typ } else { Type::Primitive(TypePrim::Any) });
		}
		Type::Void.wrap()
	}
	
	fn attr_declaration(&mut self, data: AttrDeclarationData, pos: SourcePos) -> Result<Type> {
		todo!()
	}
	
	fn assignment(&mut self, data: AssignData, pos: SourcePos) -> Result<Type> {
		let head_type = data.head.accept(self)?;
		let expr_typ = data.expr.accept(self)?;
		if head_type.accepts(&expr_typ)? {
			Type::Void.wrap()
		} else {
			ErrorList::comp(format!("Cannot assign '{}' to '{}'", expr_typ, head_type), pos).err()
		}
	}
	
	fn if_stmt(&mut self, data: IfData, pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		errors.try_append(data.cond.accept(self));
		errors.try_append(self.check_block(&data.then_block));
		errors.try_append(self.check_block(&data.else_block));
		errors.if_empty(Type::Void)
	}
	
	fn loop_stmt(&mut self, block: Block, pos: SourcePos) -> Result<Type> {
		self.check_block(&block)
	}
	
	fn break_stmt(&mut self, pos: SourcePos) -> Result<Type> {
		Type::Void.wrap()
	}
	
	fn continue_stmt(&mut self, pos: SourcePos) -> Result<Type> {
		Type::Void.wrap()
	}
	
	fn return_stmt(&mut self, expr: Box<Expression>, pos: SourcePos) -> Result<Type> {
		todo!()
	}
	
	fn scoped_stmt(&mut self, block: Block, pos: SourcePos) -> Result<Type> {
		self.check_block(&block)
	}
	
}
