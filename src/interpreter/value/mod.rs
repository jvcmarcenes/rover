
use crate::utils::{result::{Error, Result}, source_pos::SourcePos};

use self::Value::*;

use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Str(String),
	Num(f64),
	Bool(bool),
	None,
}

impl Value {
	pub fn to_num(&self, pos: SourcePos) -> Result<f64> {
		if let Value::Num(n) = self { return Ok(*n) }
		else { Error::new("Cannot cast value to number".to_owned(), pos).into() }
	}

	pub fn to_bool(&self, pos: SourcePos) -> Result<bool> {
		if let Value::Bool(b) = self { return Ok(*b) }
		else { Error::new("Cannot cast value to number".to_owned(), pos).into() }
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Str(s) => write!(f, "{}", s),
			Num(n) => write!(f, "{}", n),
			Bool(b) => write!(f, "{}", b),
			None => write!(f, ""),
		}
	}
}
