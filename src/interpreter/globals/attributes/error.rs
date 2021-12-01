
use std::collections::HashMap;

use crate::{interpreter::{value::{Value, primitives::{attribute::Attribute, object::ObjectMap, callable::{Callable, ValCallable}}, macros::castf}, globals::attributes::NatSelf, Interpreter}, ast::identifier::Identifier, utils::{result::Result, wrap::Wrap, source_pos::SourcePos}};

use super::ERROR_ATTR;

fn get() -> Box<dyn Value> {
	#[derive(Debug)] struct Get(NatSelf);
	
	impl Callable for Get {
		fn arity(&self) -> usize { 0 }
		
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let data = self.0.clone().unwrap();
			castf!(err data.borrow()).wrap()
		}
	}
	
	ValCallable::new(Get(None).wrap())
}

pub fn error() -> Box<dyn Value> {
	let mut methods = HashMap::new();
	
	let v = vec![
	("get", get()),
	];
	
	for (key, val) in v {
		methods.insert(key.to_owned(), val);
	}
	
	Attribute::new(Identifier { name: "error".to_owned(), id: ERROR_ATTR.wrap() }, ObjectMap::new(), methods)
}