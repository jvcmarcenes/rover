
#![allow(unused_variables)]

use std::collections::HashMap;

use crate::{ast::{expression::*, statement::*, identifier::Identifier}, types::Type, utils::{source_pos::SourcePos, result::{Result, ErrorList, throw}, wrap::Wrap}};

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
	
	fn declare(&mut self, id: usize, typ: Type) {
		let typ = typ.simplified(&self.type_map);
		self.type_map.insert(id, typ);
	}

}

impl ExprVisitor<Type> for TypeChecker {
	
	fn literal(&mut self, data: LiteralData, pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		let typ = match data {
			LiteralData::None => Type::None,
			LiteralData::Str(_) => Type::STR,
			LiteralData::Num(_) => Type::NUM,
			LiteralData::Bool(_) => Type::BOOL,
			LiteralData::Template(_) => Type::STR,
			LiteralData::List(exprs) => {
				let mut types = Vec::new();
				let mut is_any = false;
				for expr in exprs {
					match expr.accept(self) {
						Ok(Type::Any) => { is_any = true; break; }
						// Ok(Type::Named(name)) => {
						// 	let typ = self.type_map.get(&name.get_id()).unwrap().clone();
						// 	if !types.contains(&typ) { types.push(typ); }
						// },
						Ok(typ) => if !types.contains(&typ) { types.push(typ); },
						Err(err) => errors.append(err),
					}
				}
				let typ = match types.len() {
					_ if is_any => Type::Any,
					0 => Type::Void,
					1 => types[0].clone(),
					_ => Type::Or(types),
				};
				Type::List(typ.wrap())
			}
			_ => todo!(),
		};
		errors.if_empty(typ)
	}
	
	fn binary(&mut self, data: BinaryData, pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		let mut typ = None;
		let lhs_typ = match data.lhs.accept(self) { Ok(typ) => typ, Err(err) => { errors.append(err); Type::Void } };
		let rhs_typ = match data.rhs.accept(self) { Ok(typ) => typ, Err(err) => { errors.append(err); Type::Void } };
		throw!(errors);
		let expect = match data.op {
			BinaryOperator::Add => Type::Or(vec![Type::NUM, Type::STR]),
			BinaryOperator::Sub => Type::NUM,
			BinaryOperator::Mul => Type::NUM,
			BinaryOperator::Div => Type::NUM,
			BinaryOperator::Rem => Type::NUM,
			BinaryOperator::Equ => { typ = Some(Type::BOOL); Type::Any },
			BinaryOperator::Neq => { typ = Some(Type::BOOL); Type::Any },
			BinaryOperator::Lst => { typ = Some(Type::BOOL); Type::NUM },
			BinaryOperator::Lse => { typ = Some(Type::BOOL); Type::NUM },
			BinaryOperator::Grt => { typ = Some(Type::BOOL); Type::NUM },
			BinaryOperator::Gre => { typ = Some(Type::BOOL); Type::NUM },
			BinaryOperator::Typ => todo!(),
		};
		let typ = if lhs_typ == Type::Any || rhs_typ == Type::Any || expect.accepts(&lhs_typ, &self.type_map) && expect.accepts(&rhs_typ, &self.type_map) {
			if let Some(typ) = typ { typ }
			else if rhs_typ == Type::STR { rhs_typ }
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
				let (expect, ret) = match data.op {
					UnaryOperator::Not => (Type::Any, Type::BOOL),
					UnaryOperator::Pos => (Type::NUM, Type::NUM),
					UnaryOperator::Neg => (Type::NUM, Type::NUM),
				};
				if expr_typ == Type::Any || expect.accepts(&expr_typ, &self.type_map) {
					typ = ret;
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
		errors.if_empty(Type::BOOL)
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
		Type::Any.wrap()
	}
	
	fn index(&mut self, data: IndexData, pos: SourcePos) -> Result<Type> {
		let typ = data.head.accept(self)?;
		match typ {
			Type::Any => Type::Any,
			Type::List(typ) => *typ,
			Type::STR => Type::STR,
			_ => return ErrorList::comp(format!("Cannot index '{}'", typ), pos).err(),
		}.wrap()
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
			if !typ.accepts(&expr_typ, &self.type_map) {
				return ErrorList::comp(format!("Cannot assign '{}' to '{}'", expr_typ, typ), pos).err();
			}
		} else {
			// self.type_map.insert(data.name.get_id(), if self.infer { expr_typ } else { Type::Any });
			self.declare(data.name.get_id(), if self.infer { expr_typ } else { Type::Any });
		}
		Type::Void.wrap()
	}
	
	fn attr_declaration(&mut self, data: AttrDeclarationData, pos: SourcePos) -> Result<Type> {
		todo!()
	}
	
	fn assignment(&mut self, data: AssignData, pos: SourcePos) -> Result<Type> {
		let head_type = data.head.accept(self)?;
		let expr_typ = data.expr.accept(self)?;
		if head_type.accepts(&expr_typ, &self.type_map) {
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
	
	fn type_alias(&mut self, data: AliasData, pos: SourcePos) -> Result<Type> {
		// self.type_map.insert(data.alias.get_id(), data.typ);
		self.declare(data.alias.get_id(), data.typ);
		Type::Void.wrap()
	}
	
}
