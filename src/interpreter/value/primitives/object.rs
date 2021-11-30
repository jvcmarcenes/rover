
use std::collections::HashMap;

use crate::{interpreter::{Interpreter, value::{ValueRef, ValueType, primitives::string::Str, macros::castf}}, utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap}};

use super::super::Value;

pub type ObjectMap = HashMap<String, ValueRef>;

#[derive(Debug, Clone)]
pub struct Object {
	data: ObjectMap,
	attributes: Vec<usize>,
}

impl Object {
	pub fn new(data: ObjectMap, attributes: Vec<usize>) -> Box<dyn Value> {
		Self { data, attributes }.wrap()
	}
	
	fn method_call(&self, method: &str, interpreter: &mut Interpreter, pos: SourcePos, args: Vec<(Box<dyn Value>, SourcePos)>, default: Result<Box<dyn Value>>) -> Result<Box<dyn Value>> {
		if let Ok(field) = self.get_field(method, interpreter, pos) {
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
	
	fn get_attributes(&self) -> Vec<usize> { self.attributes.clone() }
	
	fn get_field(&self, field: &str, interpreter: &mut Interpreter, pos: SourcePos) -> Result<ValueRef> {
		if let Some(val) = self.data.get(field) {
			val.clone().wrap()
		} else {
			let attrs = self.get_attributes();
			let mut cur = attrs.as_slice();
			while let [ rest @ .., top ] = cur {
				let attr = interpreter.env.get(*top);
				match castf!(attr attr).get(field) {
					Some(method) => return method.wrap(),
					None => cur = rest,
				}
			}
			ErrorList::run(format!("Property {} is undefined for {}", field, self.get_type()), pos).err()
		}
	}
	
	fn to_string(&self, interpreter: &mut Interpreter, pos: SourcePos) -> Result<String> {
		self.method_call("to_string", interpreter, pos, vec![], Str::from("<object>").wrap())?.to_string(interpreter, pos)
	}
	
	fn equ(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<bool> {
		self.method_call("equals", interpreter, pos, vec![(other, other_pos)], ErrorList::run(format!("Property equals is undefined for {}", self.get_type()), pos).err())?.is_truthy().wrap()
	}
	
	fn add(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> {
		self.method_call("add", interpreter, pos, vec![(other, other_pos)], ErrorList::run(format!("Property add is undefined for {}", self.get_type()), pos).err())
	}
	
	fn sub(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> {
		self.method_call("sub", interpreter, pos, vec![(other, other_pos)], ErrorList::run(format!("Property sub is undefined for {}", self.get_type()), pos).err())
	}
	
	fn mul(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> {
		self.method_call("mul", interpreter, pos, vec![(other, other_pos)], ErrorList::run(format!("Property mul is undefined for {}", self.get_type()), pos).err())
	}
	
	fn div(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> {
		self.method_call("div", interpreter, pos, vec![(other, other_pos)], ErrorList::run(format!("Property div is undefined for {}", self.get_type()), pos).err())
	}
	
}