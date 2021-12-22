
use std::{fmt::Display, collections::HashMap};

use crate::{utils::{result::{Result, Stage, ErrorList}, source_pos::SourcePos}, ast::identifier::Identifier};

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
	Named(Identifier)
}

impl Type {
	
	pub const NUM: Type = Type::Primitive(TypePrim::Num);
	pub const STR: Type = Type::Primitive(TypePrim::Str);
	pub const BOOL: Type = Type::Primitive(TypePrim::Bool);
	
	pub fn accepts(&self, other: &Type, type_map: &HashMap<usize, Type>) -> bool {
		match (self, other) {
			(Void, _) | (_, Void) => false,
			(Type::Any, _) => true,
			(Unknow, _) | (_, Unknow) => true,
			(t0, t1) if t0 == t1 => true,
			(Named(n0), Named(n1)) => n0.get_id() == n1.get_id(),
			(t0, Named(name)) => t0.accepts(type_map.get(&name.get_id()).unwrap(), type_map),
			(Primitive(t0), Primitive(t1)) => t0 == t1,
			(List(t0), List(t1)) => t0.accepts(t1, type_map),
			(Object(m0), Object(m1)) => m0.iter().all(|(k, t0)| m1.get(k).map_or(false, |t1| t0.accepts(t1, type_map))),
			(t0, Or(t1)) => t1.iter().all(|typ1| t0.accepts(typ1, type_map)),
			(Or(types), other) => types.iter().any(|typ| typ.accepts(other, type_map)),
			(Named(name), t1) => type_map.get(&name.get_id()).unwrap().accepts(t1, type_map),
			_ => false,
		}
	}
	
	pub fn validate(&self, stage: Stage, pos: SourcePos) -> Result<()> {
		match self {
			List(typ) => typ.validate(stage, pos),
			Object(map) => {
				map.iter()
					.map(|(_, t)| t.validate(stage.clone(), pos))
					.filter_map(|v| if let Err(err) = v { Some(err) } else { Option::None })
					.fold(ErrorList::new(), |mut errors, err| { errors.append(err); errors})
					.if_empty(())
			}
			Or(types) => {
				if types.len() < 2 { panic!("Cannot build 'or' type with less than 2 variants") }
				if types.contains(&Type::Any) { return ErrorList::from("A type cannot be 'any' or something else".to_owned(), pos, stage).err() }
				Ok(())
			}
			_ => Ok(())
		}
	}
	
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::None => write!(f, "none"),
			Type::Any => write!(f, "any"),
			Unknow => write!(f, "<unknow>"),
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
				let mut str = String::new();
				let mut types = types.iter().peekable();
				while let Some(typ) = types.next() {
					str.push_str(&typ.to_string());
					if let Some(_) = types.peek() { str.push_str(" or "); }
				}
				write!(f, "{}", str)
			}
			Named(name) => write!(f, "{}", name.get_name()),
		}
	}
}
