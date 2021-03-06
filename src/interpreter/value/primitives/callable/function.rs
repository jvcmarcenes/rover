
use crate::{ast::{identifier::Identifier, Block}, interpreter::{Interpreter, Message, environment::Environment, value::{Value, primitives::none::ValNone, messenger::Messenger}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::Callable;

pub const SELF: usize = 0;

#[derive(Debug, Clone)]
pub struct Function {
	pub name: Option<String>,
	pub env: Environment,
	pub params: Vec<Identifier>,
	pub body: Block,
}

impl Function {
	pub fn new(name: Option<String>, env: Environment, params: Vec<Identifier>, body: Block) -> Self {
		Self { name, env, params, body }
	}
}

impl Callable for Function {
	fn cloned(&self) -> Box<dyn Callable> {
		Box::new(Function::new(self.name.clone(), self.env.cloned(), self.params.clone(), self.body.clone()))
	}

	fn display(&self) -> String {
		if let Some(ref name) = self.name {
			format!("<function {}>", name)	
		} else {
			"<lambda>".to_owned()
		}
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
			Message::Halt => Messenger::new(Message::Halt),
			_ => ValNone.wrap()
		};
		
		interpreter.env.pop();
		
		// self.env = interpreter.env.clone();
		interpreter.env = prev;
		
		ret.wrap()
	}
}
