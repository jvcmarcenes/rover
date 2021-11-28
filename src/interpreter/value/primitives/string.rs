
use crate::{interpreter::{Interpreter, value::{ValueType, macros::castf}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::super::Value;

#[derive(Debug, Clone)]
pub struct Str {
	data: String,
	attributes: Vec<usize>,
}

impl Str {
	pub fn new(data: String) -> Box<dyn Value> {
		Self { data, attributes: vec![1] }.wrap()
	}
	
	pub fn from(data: &str) -> Box<dyn Value> {
		Self::new(data.to_owned())
	}
}

impl Value for Str {
	fn get_type(&self) -> ValueType { ValueType::Str }
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn get_attributes(&self) -> Vec<usize> { self.attributes.clone() }
	
	fn to_str(&self, _pos: SourcePos) -> Result<String> { self.data.clone().wrap() }
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String> {
		self.data.clone().wrap()
	}
	
	fn add(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> {
		Str::new(format!("{}{}", self.to_string(interpreter, pos)?, other.to_string(interpreter, other_pos)?)).wrap()
	}
	
	fn equ(&self, other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> {
		(self.data == castf!(str other)).wrap()
	}
}