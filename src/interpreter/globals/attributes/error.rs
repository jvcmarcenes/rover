
use std::collections::{HashMap, HashSet};

use crate::{interpreter::{value::{Value, primitives::{attribute::Attribute, object::ObjectMap, callable::{Callable, nativefn::NativeFn}}, macros::castf}, globals::attributes::NatSelf, Interpreter}, ast::identifier::Identifier, utils::{result::Result, wrap::Wrap, source_pos::SourcePos, global_ids::global_id}};

pub const ERROR_ATTR: &str = "Error";

fn get() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Get(NatSelf);
	
	impl Callable for Get {
		fn arity(&self) -> usize { 0 }
		
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let data = self.0.clone().unwrap();
			castf!(err data.borrow()).wrap()
		}
	}
	
	NativeFn::create(Get(None).wrap())
}

pub fn error() -> Box<dyn Value> {
	let mut methods = HashMap::new();
	
	let v = vec![
	("get", get()),
	];
	
	for (key, val) in v {
		methods.insert(key.to_owned(), val.wrap());
	}
	
	Attribute::new(Identifier { name: "error".to_owned(), id: global_id(ERROR_ATTR).wrap() }, methods, ObjectMap::new(), HashSet::new())
}