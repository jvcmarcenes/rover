
use std::collections::HashMap;

use crate::{interpreter::{Interpreter, get_index, globals::attributes::NatSelf, value::{Value, macros::{cast, castf}, primitives::{attribute::Attribute, bool::Bool, callable::{Callable, ValCallable}, none::ValNone, number::Number}}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

fn size() -> Box<dyn Value> {
	#[derive(Debug)] struct Size(NatSelf);
	
	impl Callable for Size {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let vec_ref = self.0.clone().unwrap();
			Number::new(castf!(vec vec_ref.borrow()).borrow().len() as f64).wrap()
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
			let vec_ref = self.0.clone().unwrap();
			let vec = castf!(vec vec_ref.borrow());
			let vec = vec.borrow();
			let i = match get_index(n0, vec.len(), p0) {
				Ok(n) => n,
				Err(_) => return ValNone::new().wrap(),
			};
			match vec.get(i) {
				Some(val) => val.clone(),
				None => ValNone::new(),
			}.wrap()
		}
	}
	
	ValCallable::new(Get(None).wrap())
}

fn push() -> Box<dyn Value> {
	#[derive(Debug)] struct Push(NatSelf);
	
	impl Callable for Push {
		fn arity(&self) -> usize { 1 }
		
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let v0 = args[0].0.clone();
			let vec_ref = self.0.clone().unwrap();
			let vec = castf!(vec vec_ref.borrow());
			vec.borrow_mut().push(v0);
			ValNone::new().wrap()
		}
	}
	
	ValCallable::new(Push(None).wrap())
}

fn pop() -> Box<dyn Value> {
	#[derive(Debug)] struct Pop(NatSelf);
	
	impl Callable for Pop {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let vec_ref = self.0.clone().unwrap();
			let vec = castf!(vec vec_ref.borrow());
			let val = vec.borrow_mut().pop();
			val.unwrap_or(ValNone::new()).wrap()
		}
	}
	
	ValCallable::new(Pop(None).wrap())
}

fn contains() -> Box<dyn Value> {
	#[derive(Debug)] struct Contains(NatSelf);
	
	impl Callable for Contains {
		fn arity(&self) -> usize { 1 }
		
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let (v0, p0) = args[0].clone();
			let vec_ref = self.0.clone().unwrap();
			let vec = castf!(vec vec_ref.borrow());
			for val in vec.borrow().iter() {
				if val.equals(v0.clone(), p0, interpreter, pos)? {
					return Bool::new(true).wrap()
				}
			}
			Bool::new(false).wrap()
		}
	}
	
	ValCallable::new(Contains(None).wrap())
}

pub fn vector() -> Box<dyn Value> {
	let mut methods = HashMap::new();
	
	let v = vec![
	("size", size()),
	("get", get()),
	("push", push()),
	("pop", pop()),
	("contains", contains()),
	];
	
	for (key, val) in v {
		methods.insert(key.to_owned(), val);
	}
	
	Attribute::new("vector".to_owned(), methods)
}