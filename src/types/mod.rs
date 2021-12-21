
use std::fmt::Display;

use crate::utils::{wrap::Wrap, result::Result};

use self::Type::*;

#[derive(Clone, Debug, PartialEq)]
pub enum TypePrim { Num, Str, Bool, Any, None }

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
	Void,
	Primitive(TypePrim),
	Or(Vec<Type>)
}

impl Type {
	
	pub fn accepts(&self, other: &Type) -> Result<bool> {
		let other = other.clone();
		match self {
			Void => false,
			Primitive(t) => match t {
				TypePrim::None => false,
				TypePrim::Num => other == Primitive(TypePrim::Num),
				TypePrim::Str => other == Primitive(TypePrim::Str),
				TypePrim::Bool => other == Primitive(TypePrim::Bool),
				TypePrim::Any => other != Type::Void,
			},
			Or(types) => types.iter().any(|typ| typ.accepts(&other).unwrap_or_default()),
		}.wrap()
	}
	
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Void => write!(f, "void"),
			Primitive(t) => match t {
				TypePrim::None => write!(f, "none"),
				TypePrim::Num => write!(f, "number"),
				TypePrim::Str => write!(f, "string"),
				TypePrim::Bool => write!(f, "bool"),
				TypePrim::Any => write!(f, "any"),
			},
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
