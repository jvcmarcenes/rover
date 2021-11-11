
use crate::{ast::statement::Block, interpreter::{Interpreter, Message, environment::ValueMap}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::{Value, callable::Callable};


#[derive(Debug, Clone)]
pub struct Function {
	pub params: Vec<String>,
	pub body: Block,
}

impl Function {
	pub fn new(params: Vec<String>, body: Block) -> Self {
		Self { params, body }
	}
}

impl Callable for Function {
	fn arity(&self) -> u8 {
		self.params.len() as u8
	}

	fn call(&self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value> {
		let mut map = ValueMap::new();

		for (iden, val) in self.params.iter().zip(args.iter()) {
			map.insert(iden.clone(), val.clone());
		}

		match interpreter.execute_block(self.body.clone(), map)? {
			Message::Return(val) => val.wrap(),
			_ => Value::None.wrap()
		}
	}
}
