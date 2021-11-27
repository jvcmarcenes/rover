
pub mod macros;
pub mod callable;
pub mod function;

use crate::{interpreter::value::macros::pass_msg, utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap}};

use self::{Value::*, callable::Callable};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{Interpreter, Message};

#[derive(Debug, Clone)]
pub enum Value {
	None,
	Str(String),
	Num(f64),
	Bool(bool),
	List(Vec<Value>),
	Callable(Rc<RefCell<dyn Callable>>),
	Object(HashMap<String, Rc<RefCell<Value>>>),
	Messenger(Box<Message>), // Internal type
	Error(Box<Value>),
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
		// Add resolution to Attribute methods here
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

	pub fn is_error(&self) -> bool {
		if let Error(_) = self { true } else { false }
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
			Error(_) => "error",
			Messenger(_) => "<messenger> (internal type, should never be accesible to end user)",
		}.to_owned()
	}

	pub fn method_call(&self, method: &str, interpreter: &mut Interpreter, pos: SourcePos, args: Vec<(Value, SourcePos)>, default: Result<Value>) -> Result<Value> {
		if let Ok(field) = self.get_field(method, pos) {
			let callable = field.borrow().clone().to_callable(pos)?;
			callable.borrow_mut().bind(self.clone());
			let res = callable.borrow_mut().call(pos, interpreter, args);
			res
		} else {
			default
		}
	}

	pub fn to_string(&self, interpreter: &mut Interpreter, pos: SourcePos) -> Result<String> {
		match &self {
			Str(str) => str.clone(),
			Num(num) => num.to_string(),
			Bool(bool) => bool.to_string(),
			List(list) => {
				let mut str = String::from("[");
				let mut values = list.iter().peekable();
				while let Some(value) = values.next() {
					str.push_str(&value.to_string(interpreter, pos)?);
					if let Some(_) = values.peek() { str.push_str(", "); }
				}
				str.push(']');
				str
			},
			Callable(c) => c.borrow().to_string(),
			Object(_) => self.method_call("to_string", interpreter, pos, Vec::new(), Value::Str("<object>".to_owned()).wrap())?.to_string(interpreter, pos)?,
			Error(err) => format!("{}: {}", ansi_term::Color::Red.paint("error"), err.to_string(interpreter, pos)?),
			Messenger(_) => "<messenger>".to_owned(),
			None => "none".to_owned(),
		}.wrap()
	}

	pub fn equals(&self, other: &Value, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<bool> {
		match (self, other) {
			(Str(l), Str(r)) => l == r,
			(Num(l), Num(r)) => l == r,
			(Bool(l), Bool(r)) => l == r,
			(List(l), List(r)) => {
				if l.len() != r.len() { return false.wrap() } 
				for (lv, rv) in l.iter().zip(r.iter()) {
					if !lv.equals(rv, other_pos, interpreter, pos)? { return false.wrap() }
				}
				return true.wrap();
			}
			(Callable(_), Callable(_)) => false,
			(Object(_), Object(_)) => self.method_call("equals", interpreter, pos, vec![(other.clone(), other_pos)], Value::Bool(false).wrap())?.is_truthy(),
			(None, None) => true,
			_ => false,
		}.wrap()
	}

	pub fn add(&self, other: &Value, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Value> {
		let err = ErrorList::run(format!("Operation ADD is not defined for {} and {}", self.get_type(), other.get_type()), pos);
		match (pass_msg!(self), pass_msg!(other)) {
			(Str(_), _) | (_, Str(_)) => Str(format!("{}{}", self.to_string(interpreter, pos)?, other.to_string(interpreter, pos)?)),
			(Num(l), Num(r)) => Num(l + r),
			(Object(_), Object(_)) => self.method_call("add", interpreter, pos, vec![(other.clone(), other_pos)], err.err())?,
			_ => return err.err(),
		}.wrap()
	}

	pub fn sub(&self, other: &Value, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Value> {
		let err = ErrorList::run(format!("Operation SUB is not defined for {} and {}", self.get_type(), other.get_type()), pos);
		match (pass_msg!(self), pass_msg!(other)) {
			(Num(l), Num(r)) => Num(l - r),
			(Object(_), Object(_)) => self.method_call("sub", interpreter, pos, vec![(other.clone(), other_pos)], err.err())?,
			_ => return err.err(),
		}.wrap()
	}

	pub fn mul(&self, other: &Value, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Value> {
		let err = ErrorList::run(format!("Operation MUL is not defined for {} and {}", self.get_type(), other.get_type()), pos);
		match (pass_msg!(self), pass_msg!(other)) {
			(Num(l), Num(r)) => Num(l * r),
			(Object(_), Object(_)) => self.method_call("mul", interpreter, pos, vec![(other.clone(), other_pos)], err.err())?,
			_ => return err.err(),
		}.wrap()
	}

	pub fn div(&self, other: &Value, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Value> {
		let err = ErrorList::run(format!("Operation DIV is not defined for {} and {}", self.get_type(), other.get_type()), pos);
		match (pass_msg!(self), pass_msg!(other)) {
			(Num(_), Num(r)) if *r == 0.0 => return ErrorList::run("Cannot divide by zero".to_owned(), other_pos).err(),
			(Num(l), Num(r)) => Num(l / r),
			(Object(_), Object(_)) => self.method_call("div", interpreter, pos, vec![(other.clone(), other_pos)], err.err())?,
			_ => return err.err(),
		}.wrap()
	}

}
