
use crate::{interpreter::{Interpreter, value::ValueType}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::super::Value;

#[derive(Debug, Clone)]
pub struct Bool {
	data: bool,
}

impl Bool {
	pub fn new(data: bool) -> Box<dyn Value> {
		Self { data }.wrap()
	}
}

impl Value for Bool {
	fn get_type(&self) -> ValueType { ValueType::Bool }
	
	fn is_truthy(&self) -> bool { self.data }
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String> {
		self.data.to_string().wrap()
	}
	
	fn equ(&self, _other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> { panic!() }
	
	fn equals(&self, other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> {
		(self.data == other.is_truthy()).wrap()
	}
}