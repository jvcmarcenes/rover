
pub mod callable;
pub mod function;

use crate::utils::{result::{ErrorList, Result}, source_pos::SourcePos};

use self::{Value::*, callable::Callable};

use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub enum Value {
	Str(String),
	Num(f64),
	Bool(bool),
	List(Vec<Value>),
	Callable(Rc<RefCell<dyn Callable>>),
	None,
}

impl Value {
	pub fn to_num(self, pos: SourcePos) -> Result<f64> {
		if let Value::Num(n) = self { return Ok(n) }
		else { ErrorList::new("Value isn't a number".to_owned(), pos).err() }
	}

	pub fn to_bool(self, pos: SourcePos) -> Result<bool> {
		if let Value::Bool(b) = self { return Ok(b) }
		else { ErrorList::new("Value isn't a bool".to_owned(), pos).err() }
	}
	
	pub fn to_list(self, pos: SourcePos) -> Result<Vec<Value>> {
		if let Value::List(list) = self { return Ok(list) }
		else { ErrorList::new("Value isn't a list".to_owned(), pos).err() }
	}

	pub fn to_callable(self, pos: SourcePos) -> Result<Rc<RefCell<dyn Callable>>> {
		if let Value::Callable(c) = self { return Ok(c) }
		ErrorList::new("Value isn't callable".to_owned(), pos).err()
	}

	pub fn is_truthy(&self) -> bool {
		match *self {
			Self::None => false,
			Self::Bool(b) => b,
			_ => true,
		}
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
			None => write!(f, ""),
		}
	}
}
