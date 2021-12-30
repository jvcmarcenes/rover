
use std::collections::HashMap;

use crate::{ast::{expression::*, statement::*, identifier::Identifier}, types::{Type, global_types::global_types}, utils::{source_pos::SourcePos, result::{Result, ErrorList, throw, append}, wrap::Wrap}};

pub struct TypeChecker {
	infer: bool,
	type_map: HashMap<usize, Type>,
	return_stack: Vec<Type>,
}

impl TypeChecker {

	pub fn new(infer: bool) -> Self {
		Self {
			infer,
			type_map: global_types(),
			return_stack: Vec::new(),
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

	fn allowed_ret(&self) -> &Type {
		self.return_stack.last().unwrap()
	}

}

impl ExprVisitor<Type> for TypeChecker {

	fn literal(&mut self, data: LiteralData, _pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		let typ = match data {
			LiteralData::None => Type::None,
			LiteralData::Str(_) => Type::STR,
			LiteralData::Num(_) => Type::NUM,
			LiteralData::Bool(_) => Type::BOOL,
			LiteralData::Template(exprs) => {
				let mut errors = ErrorList::new();
				for expr in exprs { errors.try_append(expr.accept(self)); }
				Type::STR
			}
			LiteralData::List(exprs) => {
				let mut types = Vec::new();
				let mut is_any = false;
				for expr in exprs {
					match expr.accept(self) {
						Ok(Type::Any) => { is_any = true; break; }
						Ok(typ) => if !types.contains(&typ) { types.push(typ); },
						Err(err) => errors.append(err),
					}
				}
				let typ = match types.len() {
					_ if is_any => Type::Any,
					0 => Type::Unknow,
					1 => types[0].clone(),
					_ => Type::Or(types),
				};
				Type::List(typ.wrap())
			}
			LiteralData::Object(map, _) => {
				let mut typ_map = HashMap::new();
				for (key, expr) in map {
					let typ = append!(expr.accept(self); to errors; dummy Type::Void);
					typ_map.insert(key.clone(), typ);
				}
				Type::Object(typ_map)
			}
			LiteralData::Error(_) => todo!(),
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

	fn logic(&mut self, data: LogicData, _pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		errors.try_append(data.lhs.accept(self));
		errors.try_append(data.rhs.accept(self));
		errors.if_empty(Type::BOOL)
	}

	fn grouping(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<Type> {
		data.accept(self)
	}

	fn variable(&mut self, data: Identifier, _pos: SourcePos) -> Result<Type> {
		self.type_map.get(&data.get_id()).expect(&format!("use of unresolved symbol '{}'", data)).clone().wrap()
	}

	fn lambda(&mut self, data: LambdaData, _pos: SourcePos) -> Result<Type> {
		let param_types = data.types.iter().map(|typ| typ.clone().unwrap_or(Type::Any)).collect::<Vec<_>>();
		let returns = data.returns.unwrap_or(if self.infer { Type::Void } else { Type::Any });

		for (key, typ) in data.params.iter().zip(param_types.iter()) {
			self.type_map.insert(key.get_id(), typ.clone());
		}

		self.return_stack.push(returns.clone());

		self.check_block(&data.body)?;

		Type::Function { params: param_types, returns: returns.wrap() }.wrap()
	}

	fn call(&mut self, data: CallData, pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		let mut typ = data.calee.accept(self)?;

		if typ == Type::Any { return Type::Any.wrap(); }

		while let Type::Named(name) = typ { typ = self.type_map.get(&name.get_id()).unwrap().clone(); }

		if let Type::Function { params, returns } = typ {
			if params.len() != data.args.len() {
				errors.add_comp(format!("Expected {} arguments, got {}", params.len(), data.args.len()), pos);
			}
			for (t0, arg) in params.iter().zip(data.args.iter().cloned()) {
				match arg.accept(self) {
					Ok(t1) => if !t0.accepts(&t1, &self.type_map) { errors.add_comp(format!("Cannot pass argument of type '{}' to '{}'", t1, t0), pos); },
					Err(err) => errors.append(err),
				}
			}
			errors.if_empty(*returns)
		} else {
			ErrorList::comp(format!("Type '{}' cannot be called", typ), pos).err()
		}
	}

	fn index(&mut self, data: IndexData, pos: SourcePos) -> Result<Type> {
		let mut typ = data.head.accept(self)?;
		while let Type::Named(name) = typ { typ = self.type_map.get(&name.get_id()).unwrap().clone(); }
		match typ {
			Type::Any => Type::Any,
			Type::List(typ) => *typ,
			Type::STR => Type::STR,
			_ => return ErrorList::comp(format!("Cannot index '{}'", typ), pos).err(),
		}.wrap()
	}

	fn field(&mut self, data: FieldData, pos: SourcePos) -> Result<Type> {
		let mut typ = data.head.accept(self)?;
		while let Type::Named(name) = typ { typ = self.type_map.get(&name.get_id()).unwrap().clone(); }
		match &typ {
			Type::Any => Type::Any,
			Type::Object(map) => if let Some(typ) = map.get(&data.field) { typ.clone() } else { return ErrorList::comp(format!("Property '{}' does not exist on '{}'", data.field, typ), pos).err(); },
			_ => return ErrorList::comp(format!("Type '{}' does not exposes fields", typ), pos).err(),
		}.wrap()
	}

	fn self_ref(&mut self, _pos: SourcePos) -> Result<Type> {
		todo!()
	}

	fn do_expr(&mut self, block: Block, pos: SourcePos) -> Result<Type> {
		match self.check_block(&block)? {
			Type::Void => ErrorList::comp("do block did not evaluate to a value".to_owned(), pos).err(),
			typ => typ.wrap()
		}
	}

	fn bind_expr(&mut self, _data: BindData, _pos: SourcePos) -> Result<Type> {
		todo!()
	}

}

impl StmtVisitor<Type> for TypeChecker {

	fn expr(&mut self, expr: Box<Expression>, _pos: SourcePos) -> Result<Type> {
		expr.accept(self)
	}

	fn declaration(&mut self, data: DeclarationData, pos: SourcePos) -> Result<Type> {
		match data.expr.typ {
			ExprType::Literal(LiteralData::Object(_, _)) => {
				self.type_map.insert(data.name.get_id(), Type::Unknow);
			},
			ExprType::Lambda(LambdaData { params: _, ref types, ref returns, body: _ }) => {
				let params = types.iter().map(|typ| typ.clone().unwrap_or(Type::Any)).collect();
				let returns = returns.clone().unwrap_or(if self.infer { Type::Void } else { Type::Any }).wrap();
				let typ = Type::Function { params, returns };
				self.type_map.insert(data.name.get_id(), typ);
			}
			_ => (),
		}

		let expr_typ = data.expr.accept(self)?;
		if expr_typ == Type::Void {
			return ErrorList::comp("Cannot declare a variable as void".to_owned(), pos).err();
		}

		if let Some(typ) = data.type_restriction {
			self.type_map.insert(data.name.get_id(), typ.clone());
			if !typ.accepts(&expr_typ, &self.type_map) {
				return ErrorList::comp(format!("Cannot assign '{}' to '{}'", expr_typ, typ), pos).err();
			}
		} else {
			self.type_map.insert(data.name.get_id(), if self.infer { expr_typ } else { Type::Any });
		}

		Type::Void.wrap()
	}

	fn attr_declaration(&mut self, _data: AttrDeclarationData, _pos: SourcePos) -> Result<Type> {
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

	fn if_stmt(&mut self, data: IfData, _pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		errors.try_append(data.cond.accept(self));
		errors.try_append(self.check_block(&data.then_block));
		errors.try_append(self.check_block(&data.else_block));
		errors.if_empty(Type::Void)
	}

	fn loop_stmt(&mut self, block: Block, _pos: SourcePos) -> Result<Type> {
		self.check_block(&block)
	}

	fn break_stmt(&mut self, _pos: SourcePos) -> Result<Type> {
		Type::Void.wrap()
	}

	fn continue_stmt(&mut self, _pos: SourcePos) -> Result<Type> {
		Type::Void.wrap()
	}

	fn return_stmt(&mut self, expr: Option<Box<Expression>>, pos: SourcePos) -> Result<Type> {
		let typ = expr.map_or(Type::Void.wrap(), |expr| expr.accept(self))?;
		let expected = self.allowed_ret();
		if Type::Void == *expected && Type::Void == typ || expected.accepts(&typ, &self.type_map) {
			typ.wrap()
		} else {
			ErrorList::comp(format!("Invalid return type '{}', function must return '{}'", typ, expected), pos).err()
		}
	}

	fn scoped_stmt(&mut self, block: Block, _pos: SourcePos) -> Result<Type> {
		self.check_block(&block)
	}

	fn type_alias(&mut self, data: AliasData, _pos: SourcePos) -> Result<Type> {
		self.type_map.insert(data.alias.get_id(), data.typ.clone());
		Type::Void.wrap()
	}

}
