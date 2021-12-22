
use std::{fmt::Display, collections::HashMap};

use crate::{utils::{result::{Result, Stage, ErrorList}, source_pos::SourcePos}, ast::identifier::Identifier};

use self::Type::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypePrim { Num, Str, Bool }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
	Any, None, Void,
	Primitive(TypePrim),
	List(Box<Type>),
	Or(Vec<Type>),
	Named(Identifier)
}

impl Type {
	
	pub const NUM: Type = Type::Primitive(TypePrim::Num);
	pub const STR: Type = Type::Primitive(TypePrim::Str);
	pub const BOOL: Type = Type::Primitive(TypePrim::Bool);
	
	pub fn accepts(&self, other: &Type, type_map: &HashMap<usize, Type>) -> bool {
		let st = self.simplified(type_map);
		let ot = other.simplified(type_map);
		match (&st, &ot) {
			(Void, _) | (_, Void) => false,
			(Type::Any, _) => true,
			(t0, Named(name)) => t0.accepts(type_map.get(&name.get_id()).unwrap(), type_map),
			(Primitive(t0), Primitive(t1)) => t0 == t1,
			(List(t0), List(t1)) => t0.accepts(t1, type_map),
			(Or(t0), Or(t1)) => t1.iter().all(|typ1| t0.iter().any(|typ0| typ0.accepts(typ1, type_map))),
			(Or(types), other) => types.iter().any(|typ| typ.accepts(other, type_map)),
			(Named(name), other) => type_map.get(&name.get_id()).unwrap().accepts(other, type_map),
			(t0, t1) => t0 == t1,
		}
	}
	
	pub fn simplified(&self, type_map: &HashMap<usize, Type>) -> Type {
		match self {
			List(typ) => typ.simplified(type_map),
			Or(types) => {
				let types = types.iter()
					.map(|typ| typ.simplified(type_map))
					.flat_map(|typ| if let Or(types) = typ { types } else { vec![typ] });
				let mut dedup_types = Vec::new();
				for typ in types {
					if !dedup_types.contains(&typ) { dedup_types.push(typ); }
				}
				Or(dedup_types)
			},
			Named(name) => type_map.get(&name.get_id()).unwrap().simplified(type_map),
			typ => typ.clone(),
		}
	}
	
	pub fn validate(&self, stage: Stage, pos: SourcePos) -> Result<()> {
		match self {
			List(typ) => typ.validate(stage, pos),
			Or(types) => {
				if types.len() < 2 { panic!("Cannot build 'or' type with less than 2 variants") }
				if types.contains(&Type::Any) { return ErrorList::from("A type cannot be 'any' or something else".to_owned(), pos, stage).err() }
				Ok(())
			},
			_ => Ok(())
		}
	}
	
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::None => write!(f, "none"),
			Type::Any => write!(f, "any"),
			Void => write!(f, "void"),
			Primitive(t) => match t {
				TypePrim::Num => write!(f, "number"),
				TypePrim::Str => write!(f, "string"),
				TypePrim::Bool => write!(f, "bool"),
			},
			List(typ) => write!(f, "[{}]", typ),
			Or(types) => {
				let mut str = String::new();
				let mut types = types.iter().peekable();
				while let Some(typ) = types.next() {
					str.push_str(&typ.to_string());
					if let Some(_) = types.peek() { str.push_str(" or "); }
				}
				write!(f, "{}", str)
			},
			Named(name) => write!(f, "{}", name.get_name()),
		}
	}
}