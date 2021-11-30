
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{interpreter::{Interpreter, value::{Value, ValueType}}, utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap}};

#[derive(Debug, Clone)]
pub struct Attribute {
	name: String,
	methods: HashMap<String, Box<dyn Value>>
}

impl Attribute {
	pub fn new(name: String, methods: HashMap<String, Box<dyn Value>>) -> Box<dyn Value> {
		Self { name, methods }.wrap()
	}
	
	pub fn get(&self, method: &str) -> Option<Box<dyn Value>> {
		self.methods.get(method).map(|v| v.cloned())
	}
}

impl Value for Attribute {
	fn get_type(&self) -> ValueType { ValueType::Attribute }
	
	fn to_attr(&self, _pos: SourcePos) -> Result<Attribute> { self.clone().wrap() }
	
	fn cloned(&self) -> Box<dyn Value> {
		self.clone().wrap()
	}
	
	fn get_field(&self, _field: &str, _interpreter: &mut Interpreter, pos: SourcePos) -> Result<Rc<RefCell<Box<dyn Value>>>> {
		ErrorList::run("Cannot access fields on an attribute".to_owned(), pos).err()
	}
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String> {
		self.name.clone().wrap()
	}
	
	fn equ(&self, _other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> {
		false.wrap()
	}
}