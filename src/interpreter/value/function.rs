
use crate::{ast::{identifier::Identifier, statement::Block}, interpreter::{Interpreter, Message, environment::Environment}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::{Value, callable::Callable};

pub const SELF: usize = 0;

#[derive(Debug, Clone)]
pub struct Function {
	pub env: Environment,
	pub params: Vec<Identifier>,
	pub body: Block,
}

impl Function {
	pub fn new(env: Environment, params: Vec<Identifier>, body: Block) -> Self {
		Self { env, params, body }
	}
}

impl Callable for Function {
	fn arity(&self) -> usize {
		self.params.len()
	}

	fn bind(&mut self, binding: Value) {
		self.env.define(SELF, binding);
	}

	fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
		
		let prev = interpreter.env.clone();
		interpreter.env = self.env.clone();

		interpreter.env.push_new();

		for (iden, (val, _)) in self.params.iter().zip(args.iter()) {
			interpreter.env.define(iden.get_id(), val.clone())
		}

		let ret = match interpreter.execute_block(self.body.clone())? {
			Message::Return(val) => val,
			_ => Value::None
		};

		interpreter.env.pop();
		
		// self.env = interpreter.env.clone();
		interpreter.env = prev;

		ret.wrap()

	}
}
