
use crate::{environment::Environment, ast::{identifier::Identifier, statement::Block}, interpreter::{Interpreter, Message, value::{Value, primitives::none::ValNone}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::Callable;

pub const SELF: usize = 0;

#[derive(Debug, Clone)]
pub struct Function {
	pub env: Environment<Box<dyn Value>>,
	pub params: Vec<Identifier>,
	pub body: Block,
}

impl Function {
	pub fn new(env: Environment<Box<dyn Value>>, params: Vec<Identifier>, body: Block) -> Self {
		Self { env, params, body }
	}
}

impl Callable for Function {
	fn cloned(&self) -> Box<dyn Callable> {
		Box::new(Function::new(self.env.cloned(), self.params.clone(), self.body.clone()))
	}

	fn arity(&self) -> usize {
		self.params.len()
	}
	
	fn bind(&mut self, binding: Box<dyn Value>) {
		self.env.define(SELF, binding);
	}
	
	fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
		
		let prev = interpreter.env.clone();
		interpreter.env = self.env.clone();
		
		interpreter.env.push_new();
		
		for (iden, (val, _)) in self.params.iter().zip(args.iter()) {
			interpreter.env.define(iden.get_id(), val.clone())
		}
		
		let ret = match interpreter.execute_block(self.body.clone())? {
			Message::Return(val) => val,
			_ => ValNone.wrap()
		};
		
		interpreter.env.pop();
		
		// self.env = interpreter.env.clone();
		interpreter.env = prev;
		
		ret.wrap()
		
	}
}
