
use std::{cell::RefCell, rc::Rc};

use crate::{interpreter::{Interpreter, globals::attributes::list::LIST_ATTR, value::{ValueType, macros::castf}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap, global_ids::global_id}};

use super::super::Value;

pub type ListData = Rc<RefCell<Vec<Box<dyn Value>>>>;

#[derive(Debug, Clone)]
pub struct List {
	data: ListData,
}

impl List {
	pub fn new(data: Vec<Box<dyn Value>>) -> Box<dyn Value> {
		Self { data: data.wrap() }.wrap()
	}
}

impl Value for List {
	fn get_type(&self) -> ValueType { ValueType::Vector }
	
	fn to_list(&self, _pos: SourcePos) -> Result<ListData> { self.data.clone().wrap() } 
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn get_attributes(&self) -> Vec<usize> { vec![global_id(LIST_ATTR)] }

	fn to_string(&self, interpreter: &mut Interpreter, pos: SourcePos) -> Result<String> {
		let mut str = String::from("[");
		let data = self.data.borrow();
		let mut values = data.iter().peekable();
		while let Some(value) = values.next() {
			str.push_str(&value.to_string(interpreter, pos)?);
			if let Some(_) = values.peek() { str.push_str(", "); }
		}
		str.push(']');
		str.wrap()
	}
	
	fn equ(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<bool> {
		let other = castf!(vec other);

		if self.data.borrow().len() != other.borrow().len() { return false.wrap() }
		for (lv, rv) in self.data.borrow().iter().zip(other.borrow().iter()) {
			if !lv.equals(rv.clone(), other_pos, interpreter, pos)? { return false.wrap() }
		}
		true.wrap()
	}
}