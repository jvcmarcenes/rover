
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{interpreter::{Interpreter, value::{ValueType, primitives::string::Str}}, utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap}};

use super::super::Value;

pub type ObjectMap = HashMap<String, Rc<RefCell<Box<dyn Value>>>>;

#[derive(Debug, Clone)]
pub struct Object {
	data: ObjectMap,
}

impl Object {
	pub fn new(data: ObjectMap) -> Box<dyn Value> {
		Self { data }.wrap()
	}

	fn method_call(&self, method: &str, interpreter: &mut Interpreter, pos: SourcePos, args: Vec<(Box<dyn Value>, SourcePos)>, default: Result<Box<dyn Value>>) -> Result<Box<dyn Value>> {
		if let Ok(field) = self.get_field(method, pos) {
			let callable = field.borrow().clone().to_callable(pos)?;
			callable.borrow_mut().bind(self.cloned());
			let res = callable.borrow_mut().call(pos, interpreter, args);
			res
		} else {
			default
		}
	}
}

impl Value for Object {
	fn get_type(&self) -> ValueType { ValueType::Object }
	
	fn to_obj(&self, _pos: SourcePos) -> Result<ObjectMap> { self.data.clone().wrap() }
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn get_field(&self, field: &str, pos: SourcePos) -> Result<Rc<RefCell<Box<dyn Value>>>> {
		match self.data.get(field) {
			Some(val) => val.clone().wrap(),
			None => ErrorList::run(format!("Property {} is undefined for {}", field, self.get_type()), pos).err()
		}
	}
	
	fn to_string(&self, interpreter: &mut Interpreter, pos: SourcePos) -> Result<String> {
		self.method_call("to_string", interpreter, pos, vec![], Str::from("<object>").wrap())?.to_string(interpreter, pos)
	}
	
	fn equ(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<bool> {
		self.method_call("equals", interpreter, pos, vec![(other, other_pos)], Str::from("<object>").wrap())?.is_truthy().wrap()
	}

	fn add(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> {
		self.method_call("add", interpreter, pos, vec![(other, other_pos)], Str::from("<object>").wrap())
	}

	fn sub(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> {
		self.method_call("sub", interpreter, pos, vec![(other, other_pos)], Str::from("<object>").wrap())
	}

	fn mul(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> {
		self.method_call("mul", interpreter, pos, vec![(other, other_pos)], Str::from("<object>").wrap())
	}

	fn div(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> {
		self.method_call("div", interpreter, pos, vec![(other, other_pos)], Str::from("<object>").wrap())
	}

}