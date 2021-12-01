
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{interpreter::{Interpreter, value::{Value, ValueType}}, utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap}, ast::identifier::Identifier};

use super::object::ObjectMap;

#[derive(Debug, Clone)]
pub struct Attribute {
	id: Identifier,
	methods: HashMap<String, Box<dyn Value>>,
	fields: ObjectMap,
}

impl Attribute {
	pub fn new(id: Identifier, fields: ObjectMap, methods: HashMap<String, Box<dyn Value>>) -> Box<dyn Value> {
		Self { id, methods, fields }.wrap()
	}
	
	pub fn get(&self, method: &str) -> Option<Box<dyn Value>> {
		self.methods.get(method).map(|v| v.cloned())
	}

	pub fn get_id(&self) -> usize {
		self.id.get_id()
	}
}

impl Value for Attribute {
	fn get_type(&self) -> ValueType { ValueType::Attribute }
	
	fn to_attr(&self, _pos: SourcePos) -> Result<Attribute> { self.clone().wrap() }
	
	fn cloned(&self) -> Box<dyn Value> {
		self.clone().wrap()
	}
	
	fn get_field(&self, field: &str, _interpreter: &mut Interpreter, pos: SourcePos) -> Result<Rc<RefCell<Box<dyn Value>>>> {
		match self.fields.get(field) {
			Some(val) => val.clone().wrap(),
			None => ErrorList::run(format!("Static property {} is undefined for {}", field, self.get_type()), pos).err(),
		}
	}
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String> {
		self.id.get_name().wrap()
	}
	
	fn equ(&self, _other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> {
		false.wrap()
	}
}