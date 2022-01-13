
pub mod global_types;

use std::{fmt::Display, collections::HashMap};

use crate::{utils::{result::{Result, Stage, ErrorList, append}, source_pos::SourcePos, wrap::Wrap}, ast::identifier::Identifier};

use self::Type::*;

pub fn mismatched_types(t0: &Type, t1: &Type, pos: SourcePos) -> ErrorList {
	ErrorList::comp(format!("Mismatched types, expected {}, found {}", t0, t1), pos)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericType {
	pub params: HashMap<usize, Type>,
	pub result: Box<Type>,
}

impl GenericType {
	pub fn new(params: HashMap<usize, Type>, result: Box<Type>) -> Self {
		Self { params, result }
	}

	pub fn apply(&self, args: Vec<Type>, checker: &dyn TypeEnv, pos: SourcePos) -> Result<Type> {
		if self.params.len() != args.len() {
			return ErrorList::comp(format!("Expected {} type arguments, got {}", self.params.len(), args.len()), pos).err();
		}

		let mut errors = ErrorList::new();
		let mut typ = *self.result.clone();
		for ((name, expected), got) in self.params.iter().zip(args.iter()) {
			if expected.accepts(got, checker, pos) {
				typ = typ.substitute_name(*name, got.clone());
			} else {
				errors.append(mismatched_types(expected, got, pos));
			}
		}

		errors.if_empty(typ)
	}
}

pub trait TypeEnv {
	fn get_type_map(&self) -> &HashMap<usize, Type>;
	fn get_generics_map(&self) -> &HashMap<usize, GenericType>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypePrim { Num, Str, Bool }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
	Any, None, Void, Unknow,
	Primitive(TypePrim),
	List(Box<Type>),
	Object(HashMap<String, Type>),
	Or(Vec<Type>),
	And(Vec<Type>),
	Named(Identifier),
	Function { params: Vec<Type>, returns: Box<Type> },
	Generic { base: Identifier, args: Vec<Type> },
	UnboundGeneric(GenericType),
}

impl Type {

	pub const NUM: Type = Type::Primitive(TypePrim::Num);
	pub const STR: Type = Type::Primitive(TypePrim::Str);
	pub const BOOL: Type = Type::Primitive(TypePrim::Bool);

	pub fn accepts(&self, other: &Type, checker: &dyn TypeEnv, pos: SourcePos) -> bool {

		match (self, other) {
			(Void, _) | (_, Void) => false,
			(Type::Any, _) => true,
			(Unknow, _) | (_, Unknow) => true,

			(t0, t1) if t0 == t1 => true,

			(t0, Named(name)) => t0.accepts(checker.get_type_map().get(&name.get_id()).unwrap(), checker, pos),
			(Named(name), t1) => checker.get_type_map().get(&name.get_id()).unwrap().accepts(t1, checker, pos),

			(Primitive(t0), Primitive(t1)) if t0 == t1 => true,

			(List(t0), List(t1)) => t0.accepts(t1, checker, pos),

			(Object(m0), Object(m1)) => m0.iter().all(|(k, t0)| m1.get(k).map_or(false, |t1| t0.accepts(t1, checker, pos))),

			(t0, Or(t1)) => t1.iter().all(|typ1| t0.accepts(typ1, checker, pos)),
			(Or(types), other) => types.iter().any(|typ| typ.accepts(other, checker, pos)),

			(t0, And(t1)) => t1.iter().any(|typ1| t0.accepts(typ1, checker, pos)),
			(And(types), other) => types.iter().all(|typ| typ.accepts(other, checker, pos)),

			(Generic { base, args }, t1) => checker.get_generics_map().get(&base.get_id()).unwrap().apply(args.clone(), checker, pos).map_or(false, |t0| t0.accepts(t1, checker, pos)),
			(t0, Generic { base, args }) => checker.get_generics_map().get(&base.get_id()).unwrap().apply(args.clone(), checker, pos).map_or(false, |t1| t0.accepts(&t1, checker, pos)),

			_ => false,
		}
	}

	pub fn validate(&self, stage: Stage, pos: SourcePos) -> Result<Type> {
		let mut errors = ErrorList::new();
		let typ = match self {
			List(typ) => List(append!(typ.validate(stage, pos); to errors; dummy Type::Void).wrap()),
			Object(map) => {
				let map = map.iter().map(|(k, t)| (k.clone(), append!(t.validate(stage, pos); to errors; dummy Type::Void))).collect();
				Object(map)
			}
			Or(types) => {
				if types.len() < 2 { panic!("Cannot build 'or' type with less than 2 variants") }
				let mut new_types = Vec::new();
				for typ in types {
					let typ = append!(typ.validate(stage, pos); to errors; dummy Type::Void);
					if typ == Type::Any { errors.add("A type cannot be 'any' or something else".to_owned(), pos, stage) }
					else if let Or(types) = typ { for typ in types { new_types.push(append!(typ.validate(stage, pos); to errors; dummy Type::Void)) } }
					else { new_types.push(append!(typ.validate(stage, pos); to errors; dummy Type::Void)) }
				}
				Or(new_types)
			}
			And(types) => {
				if types.len() < 2 { panic!("Cannot build 'and' type with less than 2 variants") }
				let mut new_types = Vec::new();
				for typ in types {
					let typ = append!(typ.validate(stage, pos); to errors; dummy Type::Void);
					if typ == Type::Any { errors.add("A type cannot be 'any' and something else".to_owned(), pos, stage) }
					else if typ == Type::None { errors.add("A type cannot be 'none' and something else".to_owned(), pos, stage) }
					else if matches!(typ, Type::Primitive(_)) { errors.add("A type cannot be a 'primitive' and something else".to_owned(), pos, stage) }
					else if matches!(typ, Type::List(_)) { errors.add("A type cannot be a 'list' and something else".to_owned(), pos, stage) }
					else if let Or(ref types) = typ {
						if types.contains(&Type::None) { errors.add("A type cannot be 'none' and something else".to_owned(), pos, stage) }
						else if types.iter().any(|typ| matches!(typ, Type::Primitive(_))) { errors.add("A type cannot be a 'primitive' and something else".to_owned(), pos, stage) }
						else { new_types.push(typ) }
					} else if let And(types) = typ { for typ in types { new_types.push(append!(typ.validate(stage, pos); to errors; dummy Type::Void)) } }
					else { new_types.push(append!(typ.validate(stage, pos); to errors; dummy Type::Void)) }
				}
				And(new_types)
			}
			Function { params, returns } => {
				let mut new_params = Vec::new();
				for param in params { new_params.push(append!(param.validate(stage, pos); to errors; dummy Type::Void)); }
				let returns = append!(returns.validate(stage, pos); to errors; dummy Type::Void).wrap();
				Function { params: new_params, returns }
			}
			Generic { base, args } => {
				let mut new_args = Vec::new();
				for arg in args { new_args.push(append!(arg.validate(stage, pos); to errors; dummy Type::Void)); }
				Generic { base: base.clone(), args: new_args }
			}
			typ => typ.clone()
		};
		errors.if_empty(typ)
	}

	fn substitute_name(&self, name: usize, typ: Type) -> Type {
		match self {
			List(t0) => List(t0.substitute_name(name, typ.clone()).wrap()),
			Object(map) => Object(map.iter().map(|(k, t)| (k.clone(), t.substitute_name(name, typ.clone()))).collect()),
			Or(types) => Or(types.iter().cloned().map(|t| t.substitute_name(name, typ.clone())).collect()),
			And(types) => And(types.iter().cloned().map(|t| t.substitute_name(name, typ.clone())).collect()),
			Function { params, returns } => Function {
				params: params.iter().cloned().map(|t| t.substitute_name(name, typ.clone())).collect(),
				returns: returns.substitute_name(name, typ).wrap(),
			},
			Generic { base, args } => Generic {
				base: base.clone(),
				args: args.iter().cloned().map(|t| t.substitute_name(name, typ.clone())).collect(),
			},
			Named(id) if id.get_id() == name => typ.clone(),
			typ => typ.clone(),
		}
	}

}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::None => write!(f, "none"),
			Type::Any => write!(f, "any"),
			Unknow => write!(f, "unknow"),
			Void => write!(f, "void"),
			Primitive(t) => match t {
				TypePrim::Num => write!(f, "number"),
				TypePrim::Str => write!(f, "string"),
				TypePrim::Bool => write!(f, "bool"),
			}
			List(typ) => write!(f, "[{}]", typ),
			Object(map) => {
				write!(f, "{{")?;
				let mut map = map.iter().collect::<Vec<_>>();
				map.sort_by(|(a, _), (b, _)| a.cmp(b));
				let mut map = map.into_iter().peekable();
				while let Some((key, typ)) = map.next() {
					write!(f, " {}: {}", key, typ)?;
					if map.peek().is_some() { write!(f, ",")?; }
				}
				write!(f, " }}")?;
				Ok(())
			}
			Or(types) => {
				let mut types = types.iter().peekable();
				while let Some(typ) = types.next() {
					write!(f, "{}", typ)?;
					if let Some(_) = types.peek() { write!(f, " or ")?; }
				}
				Ok(())
			}
			And(types) => {
				let mut types = types.iter().peekable();
				while let Some(typ) = types.next() {
					write!(f, "{}", typ)?;
					if let Some(_) = types.peek() { write!(f, " and ")?; }
				}
				Ok(())
			}
			Named(name) => write!(f, "{}", name.get_name()),
			Function { params, returns } => {
				write!(f, "function(")?;
				let mut types = params.iter().peekable();
				while let Some(typ) = types.next() {
					write!(f, "{}", typ)?;
					if let Some(_) = types.peek() { write!(f, ", ")?; }
				}
				write!(f, ") -> {}", returns)?;
				Ok(())
			}
			Generic { base, args } => {
				write!(f, "{}<", base)?;
				let mut types = args.iter().peekable();
				while let Some(typ) = types.next() {
					write!(f, "{}", typ)?;
					if let Some(_) = types.peek() { write!(f, ", ")?; }
				}
				write!(f, ">")?;
				Ok(())
			}
			UnboundGeneric(data) => {
				let mut res = *data.result.clone();
				for (name, typ) in data.params.iter() {
					res = res.substitute_name(name.clone(), typ.clone());
				}
				write!(f, "{}", res)
			}
		}
	}
}
