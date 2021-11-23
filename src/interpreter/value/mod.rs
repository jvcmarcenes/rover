
pub mod callable;
pub mod function;

use crate::utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap};

use self::{Value::*, callable::Callable};

use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub enum Value {
	Str(String),
	Num(f64),
	Bool(bool),
	List(Vec<Value>),
	Callable(Rc<RefCell<dyn Callable>>),
	Object(HashMap<String, Rc<RefCell<Value>>>),
	None,
}

impl Value {
	pub fn to_num(self, pos: SourcePos) -> Result<f64> {
		if let Value::Num(n) = self { return Ok(n) }
		ErrorList::run("Value isn't a number".to_owned(), pos).err()
	}
	
	pub fn to_bool(self, pos: SourcePos) -> Result<bool> {
		if let Value::Bool(b) = self { return Ok(b) }
		ErrorList::run("Value isn't a bool".to_owned(), pos).err()
	}
	
	pub fn to_list(self, pos: SourcePos) -> Result<Vec<Value>> {
		if let Value::List(list) = self { return Ok(list) }
		ErrorList::run("Value isn't a list".to_owned(), pos).err()
	}

	pub fn to_callable(self, pos: SourcePos) -> Result<Rc<RefCell<dyn Callable>>> {
		if let Value::Callable(c) = self { return Ok(c) }
		ErrorList::run("Value isn't a function".to_owned(), pos).err()
	}
	
	pub fn to_obj(self, pos: SourcePos) -> Result<HashMap<String, Rc<RefCell<Value>>>> {
		if let Value::Object(map) = self { return Ok(map) }
		ErrorList::run("Value isn't an object".to_owned(), pos).err()
	}

	pub fn get_field(&self, field: &str, pos: SourcePos) -> Result<Rc<RefCell<Value>>> {
		match self {
			Object(map) => match map.get(field) {
				Some(val) => return val.clone().wrap(),
				_ => (),
			}
			_ => (),
		}
		ErrorList::run(format!("Property {} is undefined for {}", field, self.get_type()), pos).err()
	}

	pub fn is_truthy(&self) -> bool {
		match *self {
			None => false,
			Bool(b) => b,
			_ => true,
		}
	}

	pub fn get_type(&self) -> String {
		match self {
			Str(_) => "string",
			Num(_) => "number",
			Bool(_) => "boolean",
			List(_) => "list",
			Callable(_) => "function",
			Object(_) => "object",
			None => "none",
		}.to_owned()
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Str(s) => write!(f, "{}", s),
			Num(n) => write!(f, "{}", n),
			Bool(b) => write!(f, "{}", b),
			List(list) => {
				write!(f, "[")?;
				let mut i = 0;
				loop {
					if i >= list.len() { break; }
					write!(f, "{}", list[i])?;
					if i + 1 < list.len() { write!(f, ", ")?; }
					i += 1;
				}
				write!(f, "]")
			},
			Callable(c) => write!(f, "{}", c.borrow()),
			Object(_) => write!(f, "<object>"),
			None => write!(f, "none"),
		}
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Str(l), Str(r)) => l == r,
			(Num(l), Num(r)) => l == r,
			(Bool(l), Bool(r)) => l == r,
			(List(l), List(r)) => l == r,
			(Callable(_), Callable(_)) => false,
			(Object(l), Object(r)) => l == r,
			(None, None) => true,
			_ => false,
		}
	}
}
