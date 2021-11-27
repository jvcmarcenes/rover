
use crate::{interpreter::{Interpreter, value::{ValueType, macros::castf}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::super::Value;

#[derive(Debug, Clone)]
pub struct Vector {
	data: Vec<Box<dyn Value>>,
}

impl Vector {
	pub fn new(data: Vec<Box<dyn Value>>) -> Box<dyn Value> {
		Self { data }.wrap()
	}
}

impl Value for Vector {
	fn get_type(&self) -> ValueType { ValueType::Vector }
	
	fn to_vector(&self, _pos: SourcePos) -> Result<Vec<Box<dyn Value>>> { self.data.clone().wrap() } 
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn to_string(&self, interpreter: &mut Interpreter, pos: SourcePos) -> Result<String> {
		let mut str = String::from("[");
		let mut values = self.data.iter().peekable();
		while let Some(value) = values.next() {
			str.push_str(&value.to_string(interpreter, pos)?);
			if let Some(_) = values.peek() { str.push_str(", "); }
		}
		str.push(']');
		str.wrap()
	}
	
	fn equ(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<bool> {
		let other = castf!(vec other);

		if self.data.len() != other.len() { return false.wrap() }
		for (lv, rv) in self.data.iter().zip(other.iter()) {
			if !lv.equals(rv.clone(), other_pos, interpreter, pos)? { return false.wrap() }
		}
		true.wrap()
	}
}