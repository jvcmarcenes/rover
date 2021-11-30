
use std::collections::HashMap;

use crate::{interpreter::{Interpreter, get_index, globals::attributes::NatSelf, value::{Value, macros::{cast, castf}, primitives::{attribute::Attribute, bool::Bool, callable::{Callable, ValCallable}, error::Error, none::ValNone, number::Number, string::Str, object::ObjectMap}}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

fn is_num() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct IsNum(NatSelf);
	
	impl Callable for IsNum {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let str_ref = self.0.clone().unwrap();
			Bool::new(castf!(str str_ref.borrow()).parse::<f64>().is_ok()).wrap()
		}
	}
	
	ValCallable::new(IsNum(None).wrap())
}

fn to_num() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct ToNum(NatSelf);
	
	impl Callable for ToNum {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
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

fn size() -> Box<dyn Value> {
	#[derive(Debug)] struct Size(NatSelf);
	
	impl Callable for Size {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let str_ref = self.0.clone().unwrap();
			Number::new(castf!(str str_ref.borrow()).len() as f64).wrap()
		}
	}
	
	ValCallable::new(Size(None).wrap())
}

fn get() -> Box<dyn Value> {
	#[derive(Debug)] struct Get(NatSelf);
	
	impl Callable for Get {
		fn arity(&self) -> usize { 1 }
		
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			let p0 = args[0].1.clone();
			let str_ref = self.0.clone().unwrap();
			let str = castf!(str str_ref.borrow());
			let i = match get_index(n0, str.len(), p0) {
				Ok(n) => n,
				Err(_) => return ValNone::new().wrap(),
			};
			match str.chars().skip(i).next() {
				Some(c) => Str::new(c.to_string()),
				None => ValNone::new(),
			}.wrap()
		}
	}
	
	ValCallable::new(Get(None).wrap())
}

pub fn string() -> Box<dyn Value> {
	let mut methods = HashMap::new();
	
	let v = vec![
	("is_num", is_num()),
	("to_num", to_num()),
	("size", size()),
	("get", get()),
	];
	
	for (key, val) in v {
		methods.insert(key.to_owned(), val);
	}
	
	Attribute::new("string".to_owned(), ObjectMap::new(), methods)
}