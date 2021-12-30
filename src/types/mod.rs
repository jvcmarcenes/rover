
pub mod global_types;

use std::{fmt::Display, collections::HashMap};

use crate::{utils::{result::{Result, Stage, ErrorList, append}, source_pos::SourcePos, wrap::Wrap}, ast::identifier::Identifier};

use self::Type::*;

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
}

impl Type {

	pub const NUM: Type = Type::Primitive(TypePrim::Num);
	pub const STR: Type = Type::Primitive(TypePrim::Str);
	pub const BOOL: Type = Type::Primitive(TypePrim::Bool);

	pub fn accepts(&self, other: &Type, type_map: &HashMap<usize, Type>) -> bool {
		// this should return a result with the error clarifying why type 0 does not accept type 1

		match (self, other) {
			(Void, _) | (_, Void) => false,
			(Type::Any, _) => true,
			(Unknow, _) | (_, Unknow) => true,

			(t0, t1) if t0 == t1 => true,

			(t0, Named(name)) => t0.accepts(type_map.get(&name.get_id()).unwrap(), type_map),
			(Named(name), t1) => type_map.get(&name.get_id()).unwrap().accepts(t1, type_map),

			(Primitive(t0), Primitive(t1)) => t0 == t1,

			(List(t0), List(t1)) => t0.accepts(t1, type_map),

			(Object(m0), Object(m1)) => m0.iter().all(|(k, t0)| m1.get(k).map_or(false, |t1| t0.accepts(t1, type_map))),

			(t0, Or(t1)) => t1.iter().all(|typ1| t0.accepts(typ1, type_map)),
			(Or(types), other) => types.iter().any(|typ| typ.accepts(other, type_map)),

			(t0, And(t1)) => t1.iter().any(|typ1| t0.accepts(typ1, type_map)),
			(And(types), other) => types.iter().all(|typ| typ.accepts(other, type_map)),

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
			typ => typ.clone()
		};
		errors.if_empty(typ)
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
		}
	}
}
