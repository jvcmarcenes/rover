
use crate::{interpreter::{Interpreter, value::ValueType}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::super::Value;

#[derive(Debug, Clone)]
pub struct ValNone;

impl ValNone {
	pub fn new() -> Box<dyn Value> { Self.wrap() }
}

impl Value for ValNone {
	fn get_type(&self) -> ValueType { ValueType::None }
	
	fn is_truthy(&self) -> bool { false }
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String> {
		"none".to_owned().wrap()
	}

	fn equ(&self, _other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> {
		true.wrap()
	}
}