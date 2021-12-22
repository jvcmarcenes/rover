
use std::fmt::Display;

use crate::utils::{wrap::Wrap, result::{Result, Stage, ErrorList}, source_pos::SourcePos};

use self::Type::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypePrim { Num, Str, Bool }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
	Any, None, Void,
	Primitive(TypePrim),
	List(Box<Type>),
	Or(Vec<Type>)
}

impl Type {
	
	pub const NUM: Type = Type::Primitive(TypePrim::Num);
	pub const STR: Type = Type::Primitive(TypePrim::Str);
	pub const BOOL: Type = Type::Primitive(TypePrim::Bool);

	pub fn accepts(&self, other: &Type) -> Result<bool> {
		match (self, other) {
			(Void, _) | (_, Void) => false,
			(Type::Any, _) => true,
			(Primitive(t0), Primitive(t1)) => t0 == t1,
			(List(t0), List(t1)) => t0.accepts(t1).unwrap_or_default(),
			(Or(types), other) => types.iter().any(|typ| typ.accepts(other).unwrap_or_default()),
			(t0, t1) => t0 == t1,
		}.wrap()
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
		}
	}
}
