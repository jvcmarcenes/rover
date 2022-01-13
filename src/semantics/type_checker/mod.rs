
use std::collections::HashMap;

use crate::{ast::{expression::*, statement::*, identifier::Identifier, Block, module::Module}, types::{Type, global_types::global_types, TypeEnv, GenericType, mismatched_types}, utils::{source_pos::SourcePos, result::{Result, ErrorList, throw, append}, wrap::Wrap}};

pub struct TypeChecker {
	infer: bool,
	type_map: HashMap<usize, Type>,
	generic_types: HashMap<usize, GenericType>,
	return_stack: Vec<Type>,
}

impl TypeChecker {
	
	pub fn new(infer: bool) -> Self {
		Self {
			infer,
			type_map: global_types(),
			generic_types: HashMap::new(),
			return_stack: Vec::new(),
		}
	}

	fn decl_type(&mut self, id: usize, stmt: &Statement, infer: bool) {
		match stmt.typ {
			StmtType::FuncDeclaration(FunctionData { ref name, ref type_params, ref types, ref returns, .. }) => {
				let params = types.iter().cloned().map(|typ| typ.unwrap_or(Type::Any)).collect();
				let returns = returns.clone().unwrap_or(if infer || name.get_name() == "main" { Type::Void } else { Type::Any }).wrap();
				let typ = Type::Function { params, returns };
				
				if type_params.is_empty() {
					self.type_map.insert(id, typ);
				} else {
					let type_params = type_params.iter().cloned().map(|(id, t)| (id.get_id(), t)).collect();
					self.generic_types.insert(id, GenericType::new(type_params, typ.wrap()));
				}
			},
			StmtType::AttrDeclaration(_) => todo!(),
			StmtType::TypeAlias(AliasData { alias: _, ref type_params, ref typ }) => if type_params.is_empty() {
				self.type_map.insert(id, typ.clone());
			} else {
				let type_params = type_params.iter().cloned().map(|(id, t)| (id.get_id(), t)).collect();
				self.generic_types.insert(id, GenericType::new(type_params, typ.clone().wrap()));
			}
			_ => panic!("decl_type should ever only be called with declaration statements"),
		}
	}
	
	fn check_rec_type(&mut self, names: &Vec<usize>, typ: &Type) -> bool {
		match typ {
			Type::Named(name) => {
				if names.contains(&name.get_id()) { return true; }
				if !self.type_map.contains_key(&name.get_id()) { return false; }
				let mut names = names.clone();
				names.push(name.get_id());
				let typ = self.type_map.get(&name.get_id()).unwrap().clone();
				self.check_rec_type(&names, &typ);
			},
			Type::Or(types) | Type::And(types) => {
				for typ in types {
					if self.check_rec_type(names, typ) { return true; }
				}
			},
			_ => (),
		}
		false
	}
	
	pub fn check(&mut self, module: &Module) -> Result<()> {
		let mut errors = ErrorList::new();

		let mut env = module.env.iter().collect::<Vec<_>>();
		env.sort_by(|_, (_, b)| if matches!(b.typ, StmtType::TypeAlias(_)) { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less });
		
		for (id, stmt) in env.iter().cloned() {
			if let StmtType::TypeAlias(data) = stmt.typ.clone() {
				let names = vec![data.alias.get_id()];
				if self.check_rec_type(&names, &data.typ) {
					errors.add_comp(format!("Illegal recursive type '{}'", data.alias), stmt.pos);
				}
			}
			self.decl_type(id.get_id(), stmt, self.infer);
		}
		
		errors.if_empty(())?;
		
		for stmt in module.env.values().cloned() {
			errors.try_append(stmt.accept(self));
		}
		
		if let Some(main_id) = module.main_id.borrow().clone() {
			let typ = self.type_map.get(&main_id).unwrap();
			if let Type::Function { returns, .. } = typ.clone() {
				match *returns {
					Type::Void => (),
					_ => errors.add_mod_comp(format!("Illegal return type for 'main' function {}", typ)),
				}
			}
		}
		
		errors.if_empty(())
	}
	
	pub fn check_block(&mut self, block: &Block) -> Result<Type> {
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
	
	fn allowed_ret(&self) -> &Type {
		self.return_stack.last().unwrap()
	}
	
}

impl TypeEnv for TypeChecker {
	fn get_type_map(&self) -> &HashMap<usize, Type> { &self.type_map }
	fn get_generics_map(&self) -> &HashMap<usize, GenericType> { &self.generic_types }
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
	
	fn binary(&mut self, data: BinaryData, _pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		let mut typ = None;
		let (lhs_pos, rhs_pos) = (data.lhs.pos, data.rhs.pos);
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
		let typ = {
			if !(lhs_typ == Type::Any || rhs_typ == Type::Any) {
				if !expect.accepts(&lhs_typ, self, lhs_pos) { errors.append(mismatched_types(&expect, &lhs_typ, lhs_pos)); };
				if !expect.accepts(&rhs_typ, self, rhs_pos) { errors.append(mismatched_types(&expect, &rhs_typ, rhs_pos)); };
			}
			if let Some(typ) = typ { typ }
			else if rhs_typ == Type::STR { rhs_typ }
			else { lhs_typ }
		};
		errors.if_empty(typ)
	}
	
	fn unary(&mut self, data: UnaryData, pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		let mut typ = Type::Void;
		let expr_pos = data.expr.pos;
		match data.expr.accept(self) {
			Ok(expr_typ) => {
				let (expect, ret) = match data.op {
					UnaryOperator::Not => (Type::Any, Type::BOOL),
					UnaryOperator::Pos => (Type::NUM, Type::NUM),
					UnaryOperator::Neg => (Type::NUM, Type::NUM),
				};
				if expr_typ == Type::Any || expect.accepts(&expr_typ, self, expr_pos) {
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
		if self.generic_types.contains_key(&data.get_id()) {
			let gen = self.generic_types.get(&data.get_id()).unwrap().clone();
			Type::UnboundGeneric(gen).wrap()
		} else {
			self.type_map.get(&data.get_id()).expect(&format!("Use of unresolved symbol '{:?}'", data)).clone().wrap()
		}
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

		let mut arg_types = Vec::new();
		for arg in data.args.iter().cloned() {
			arg_types.push(append!(arg.accept(self); to errors; dummy Type::Any));
		}

		let mut typ = data.calee.accept(self)?;
		
		if typ == Type::Any { return Type::Any.wrap(); }
		
		while let Type::Named(name) = typ { typ = self.type_map.get(&name.get_id()).unwrap().clone(); }
		
		if let Type::Function { params, returns } = typ {
			if params.len() != data.args.len() {
				errors.add_comp(format!("Expected {} arguments, got {}", params.len(), data.args.len()), pos);
			}

			for (t0, arg_typ) in params.iter().zip(arg_types) {
				if !t0.accepts(&arg_typ, self, pos) {
					errors.append(mismatched_types(t0, &arg_typ, pos));
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
			Type::Object(map) => if let Some(typ) = map.get(&data.field) { typ.clone() } else { return ErrorList::comp(format!("Property '{}' does not exist on '{}'", data.field, typ), pos).err(); },
			Type::Any => Type::Any,
			_ => Type::Any,
			// _ => return ErrorList::comp(format!("Type '{}' does not exposes fields", typ), pos).err(),
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
	
	fn generic_call_expr(&mut self, data: GenericData, pos: SourcePos) -> Result<Type> {
		let typ = data.expr.accept(self)?;
		if let Type::UnboundGeneric(gen) = typ {
			if gen.params.len() != data.args.len() {
				return ErrorList::comp(format!("Expected {} type arguments, got {}", gen.params.len(), data.args.len()), pos).err();
			}
			
			let mut errors = ErrorList::new();
			for ((id, expected), got) in gen.params.iter().zip(data.args.iter()) {
				if expected.accepts(got, self, pos) {
					self.type_map.insert(*id, got.clone());
				} else {
					errors.append(mismatched_types(expected, got, pos));
				}
			}
			errors.if_empty(())?;
			
			let result = append!(gen.apply(data.args, self, pos); to errors; dummy Type::Void);
			errors.if_empty(result)
		} else {
			ErrorList::comp(format!("Type '{}' is not generic", typ), pos).err()
		}
	}

}

impl StmtVisitor<Type> for TypeChecker {
	
	fn func_declaration(&mut self, data: FunctionData, _pos: SourcePos) -> Result<Type> {

		for (type_param, typ) in data.type_params {
			self.type_map.insert(type_param.get_id(), typ);
		}

		let param_types = data.types.iter().map(|typ| typ.clone().unwrap_or(Type::Any)).collect::<Vec<_>>();
		let returns = data.returns.unwrap_or(if self.infer || data.name.get_name() == "main" { Type::Void } else { Type::Any });
		
		self.type_map.insert(data.name.get_id(), Type::Function { params: param_types.clone(), returns: returns.clone().wrap() });
		
		for (key, typ) in data.params.iter().zip(param_types.iter()) {
			self.type_map.insert(key.get_id(), typ.clone());
		}
		
		self.return_stack.push(returns);
		
		self.check_block(&data.body)?;
		
		Type::Void.wrap()
	}
	
	fn attr_declaration(&mut self, _data: AttrDeclarationData, _pos: SourcePos) -> Result<Type> {
		todo!()
	}
	
	fn type_alias(&mut self, _data: AliasData, _pos: SourcePos) -> Result<Type> {
		Type::Void.wrap()
	}
	
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
		
		let expr_typ = match data.expr.accept(self) {
			Ok(typ) => typ,
			err => {
				self.type_map.insert(data.name.get_id(), Type::Any);
				return err;
			}
		};
		
		if expr_typ == Type::Void {
			return ErrorList::comp("Cannot declare a variable as void".to_owned(), pos).err();
		}
		
		if let Some(typ) = data.type_restriction {
			self.type_map.insert(data.name.get_id(), typ.clone());
			if !typ.accepts(&expr_typ, self, pos) { return mismatched_types(&typ, &expr_typ, pos).err(); };
		} else {
			self.type_map.insert(data.name.get_id(), if self.infer { expr_typ } else { Type::Any });
		}
		
		Type::Void.wrap()
	}
	
	fn assignment(&mut self, data: AssignData, _pos: SourcePos) -> Result<Type> {
		let head_type = data.head.accept(self)?;
		let expr_pos = data.expr.pos;
		let expr_typ = data.expr.accept(self)?;
		if !head_type.accepts(&expr_typ, self, expr_pos) { return mismatched_types(&head_type, &expr_typ, expr_pos).err(); };
		Type::Void.wrap()
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
		let typ = expr.clone().map_or(Type::Void.wrap(), |expr| expr.accept(self))?;
		let expected = self.allowed_ret();
		if !(Type::Void == *expected && Type::Void == typ) {
			let pos = expr.map_or(pos, |e| e.pos);
			if !expected.accepts(&typ, self, pos) { return mismatched_types(&expected, &typ, pos).err(); };
		}
		Type::Void.wrap()
	}
	
	fn scoped_stmt(&mut self, block: Block, _pos: SourcePos) -> Result<Type> {
		self.check_block(&block)
	}
	
}
