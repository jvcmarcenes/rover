
use crate::{ast::{Identifier, statement::Block}, interpreter::{Interpreter, Message, environment::{Environment}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::{Value, callable::Callable};


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

	fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {

		for (iden, (val, _)) in self.params.iter().zip(args.iter()) {
			interpreter.env.assign(iden.get_id(), val.clone())
		}

		let prev = interpreter.env.clone();
		interpreter.env = self.env.clone();

		let ret = match interpreter.execute_block(self.body.clone())? {
			Message::Return(val) => val,
			_ => Value::None
		};

		self.env = interpreter.env.clone();
		interpreter.env = prev;

		ret.wrap()

	}
}
