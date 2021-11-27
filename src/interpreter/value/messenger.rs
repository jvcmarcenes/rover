
use crate::{interpreter::{Interpreter, Message}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::{Value, ValueType};

#[derive(Debug, Clone)]
pub struct Messenger(Message);

impl Messenger {
	pub fn new(data: Message) -> Box<dyn Value> {
		Self(data).wrap()
	}
}

impl Value for Messenger {
	fn get_type(&self) -> ValueType { ValueType::Messenger }
	
	fn to_message(&self) -> Message { self.0.clone() }
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String> {
		"<messenger>".to_owned().wrap()
	}

	fn equ(&self, _other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> { false.wrap() }
}