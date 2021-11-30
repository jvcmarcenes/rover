
use crate::{interpreter::{Interpreter, value::{ValueType, macros::castf}}, utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap}};

use super::super::Value;

#[derive(Debug, Clone)]
pub struct Number {
	data: f64,
}

impl Number {
	pub fn new(data: f64) -> Box<dyn Value> {
		Self { data }.wrap()
	}
}

impl Value for Number {
	fn get_type(&self) -> ValueType { ValueType::Num }
	
	fn to_num(&self, _pos: SourcePos) -> Result<f64> { self.data.wrap() }
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String> {
		self.data.to_string().wrap()
	}
	
	fn add(&self, other: Box<dyn Value>, other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<Box<dyn Value>> {
		Number::new(self.data + other.to_num(other_pos)?).wrap()
	}
	
	fn sub(&self, other: Box<dyn Value>, other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<Box<dyn Value>> {
		Number::new(self.data - other.to_num(other_pos)?).wrap()
	}
	
	fn mul(&self, other: Box<dyn Value>, other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<Box<dyn Value>> {
		Number::new(self.data * other.to_num(other_pos)?).wrap()
	}
	
	fn div(&self, other: Box<dyn Value>, other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<Box<dyn Value>> {
		let r = other.to_num(other_pos)?;
		if r == 0.0 {
			ErrorList::run("Cannot divide by zero".to_owned(), other_pos).err()
		} else {
			Number::new(self.data / r).wrap()
		}
	}
	
	fn equ(&self, other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> {
		(self.data == castf!(num other)).wrap()
	}
	
}