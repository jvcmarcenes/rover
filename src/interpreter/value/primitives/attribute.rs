
use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::{interpreter::{Interpreter, value::{Value, ValueType, macros::castf}}, utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap}, ast::identifier::Identifier};

use super::object::ObjectMap;

#[derive(Debug, Clone)]
pub struct Attribute {
	id: Identifier,
	methods: ObjectMap,
	fields: ObjectMap,
	attributes: HashSet<usize>,
}

impl Attribute {
	pub fn new(id: Identifier, methods: ObjectMap, fields: ObjectMap, attributes: HashSet<usize>) -> Box<dyn Value> {
		Self { id, methods, fields, attributes }.wrap()
	}
	
	pub fn get(&self, method: &str) -> Option<Box<dyn Value>> {
		self.methods.get(method).map(|v| v.borrow().cloned())
	}
	
	pub fn get_id(&self) -> usize {
		self.id.get_id()
	}
	
	pub fn super_attrs(&self) -> HashSet<usize> {
		self.attributes.clone()
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
			None => {
				match self.methods.get(field) {
					Some(val) => val.clone().wrap(),
					None => {
						ErrorList::run(format!("Property {} is undefined for {}", field, self.get_type()), pos).err()
					}
				}
			}
		}
	}
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String> {
		self.id.get_name().wrap()
	}
	
	fn equ(&self, _other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> {
		false.wrap()
	}
	
	fn has_attr(&self, attr: usize, interpreter: &mut Interpreter) -> bool {
		fn find(layer: HashSet<usize>, attr: usize, interpreter: &mut Interpreter) -> bool {
			let v = layer.iter().cloned().collect::<Vec<_>>();
			let mut cur = v.as_slice();
			while let [ rest @ .., top ] = cur {
				if *top == attr { return true; }
				let val = interpreter.env.get(*top);
				let val = castf!(attr val);
				if find(val.super_attrs(), attr, interpreter) { return true; }
				cur = rest;
			}
			false
		}
		find(self.attributes.clone(), attr, interpreter)
	}
}