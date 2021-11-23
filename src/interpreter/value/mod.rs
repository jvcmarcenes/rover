
pub mod callable;
pub mod function;

use crate::utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap};

use self::{Value::*, callable::Callable};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::Interpreter;

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

	pub fn to_string(&self, interpreter: &mut Interpreter, pos: SourcePos) -> Result<String> {
		match &self {
			Str(str) => str.clone(),
			Num(num) => num.to_string(),
			Bool(bool) => bool.to_string(),
			List(list) => {
				let mut str = String::new();
				str.push('[');
				let mut i = 0;
				loop {
					if i >= list.len() { break; }
					str.push_str(&list[i].to_string(interpreter, pos)?);
					if i + 1 < list.len() { str.push_str(", "); }
					i += 1;
				}
				str.push(']');
				str
			},
			Callable(c) => c.borrow().to_string(),
			Object(_) => {
				if let Ok(field) = self.get_field("to_string", pos) {
					let callable = field.borrow().clone().to_callable(pos)?;
					callable.borrow_mut().bind(self.clone());
					let res = callable.borrow_mut().call(pos, interpreter, Vec::new())?;
					res.to_string(interpreter, pos)?
				} else {
					"<object>".to_owned()
				}
			},
			None => "none".to_owned(),
		}.wrap()
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
