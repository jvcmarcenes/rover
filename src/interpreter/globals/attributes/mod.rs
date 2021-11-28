
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{interpreter::{Interpreter, value::{Value, macros::castf, primitives::{attribute::Attribute, bool::Bool, callable::{Callable, ValCallable}, error::Error, number::Number, string::Str}}}, resolver::IdentifierData, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

use super::Globals;

pub const DEFAULT_ATTR: &[(&str, usize)] = &[
	("string", 1)
];

fn is_num() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct IsNum(Option<Rc<RefCell<Box<dyn Value>>>>);
	
	impl Callable for IsNum {
		fn arity(&self) -> usize { 0 }

		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = Some(binding.wrap()) }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let str_ref = self.0.clone().unwrap();
			Bool::new(castf!(str str_ref.borrow()).parse::<f64>().is_ok()).wrap()
		}
	}

	ValCallable::new(IsNum(None).wrap())
}

fn to_num() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct ToNum(Option<Rc<RefCell<Box<dyn Value>>>>);
	
	impl Callable for ToNum {
		fn arity(&self) -> usize { 0 }

		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = Some(binding.wrap()) }

		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let str_ref = self.0.clone().unwrap();
			match castf!(str str_ref.borrow()).parse::<f64>() {
				Ok(n) => Number::new(n),
				Err(_) => Error::new(Str::from("Cannot convert to number"))
			}.wrap()
		}
	}

	ValCallable::new(ToNum(None).wrap())
}

fn string() -> Box<dyn Value> {
	let mut methods = HashMap::new();
	
	let v = vec![
		("is_num", is_num()),
		("to_num", to_num()),
	];

	for (key, val) in v {
		methods.insert(key.to_owned(), val);
	}
	
	Attribute::new("string".to_owned(), methods)
}

pub(super) fn register_default_attr(globals: &mut Globals) {
	
	let v = vec![
		("string", string()),
	];

	let mut i = 1;
	for (key, val) in v {
		globals.ids.insert(key.to_owned(), IdentifierData::new(i, true));
		globals.values.insert(i, val);
		i += 1;
	}
	
}
