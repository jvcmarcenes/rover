
use crate::{interpreter::{Interpreter, value::ValueType}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::super::Value;

#[derive(Debug, Clone)]
pub struct Error {
	data: Box<dyn Value>,
}

impl Error {
	pub fn new(data: Box<dyn Value>) -> Box<dyn Value> {
		Self { data }.wrap()
	}
}

impl Value for Error {
	fn get_type(&self) -> ValueType { ValueType::Error }
	
	fn to_error(&self, _pos: SourcePos) -> Result<Box<dyn Value>> { self.data.clone().wrap() }
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn to_string(&self, interpreter: &mut Interpreter, pos: SourcePos) -> Result<String> {
		format!("{}: {}", ansi_term::Color::Red.paint("error"), self.data.to_string(interpreter, pos)?).wrap()
	}

	fn equ(&self, _other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> { false.wrap() }
}