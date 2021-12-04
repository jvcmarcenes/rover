
use std::collections::{HashMap, HashSet};

use crate::{interpreter::{Interpreter, get_index, globals::attributes::NatSelf, value::{Value, macros::{cast, castf}, primitives::{attribute::Attribute, bool::Bool, callable::{Callable, nativefn::NativeFn}, none::ValNone, number::Number, object::ObjectMap, vector::Vector}}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap, global_ids::global_id}, ast::identifier::Identifier};

pub const VECTOR_ATTR: &str = "Vector";

fn size() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Size(NatSelf);
	
	impl Callable for Size {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let vec_ref = self.0.clone().unwrap();
			Number::new(castf!(vec vec_ref.borrow()).borrow().len() as f64).wrap()
		}
	}
	
	NativeFn::create(Size(None).wrap())
}

fn get() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Get(NatSelf);
	
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
	
	NativeFn::create(Get(None).wrap())
}

fn push() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Push(NatSelf);
	
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
	
	NativeFn::create(Push(None).wrap())
}

fn pop() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Pop(NatSelf);
	
	impl Callable for Pop {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let vec_ref = self.0.clone().unwrap();
			let vec = castf!(vec vec_ref.borrow());
			let val = vec.borrow_mut().pop();
			val.unwrap_or(ValNone::new()).wrap()
		}
	}
	
	NativeFn::create(Pop(None).wrap())
}

fn contains() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Contains(NatSelf);
	
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
	
	NativeFn::create(Contains(None).wrap())
}

fn reverse() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Reverse(NatSelf);
	
	impl Callable for Reverse {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let vec_ref = self.0.clone().unwrap();
			let vec = castf!(vec vec_ref.borrow());
			let mut rev = vec.borrow().clone();
			rev.reverse();
			Vector::new(rev).wrap()
		}
	}
	
	NativeFn::create(Reverse(None).wrap())
}

pub fn vector() -> Box<dyn Value> {
	let mut methods = HashMap::new();
	
	let v = vec![
	("size", size()),
	("get", get()),
	("push", push()),
	("pop", pop()),
	("contains", contains()),
	("reverse", reverse()),
	];
	
	for (key, val) in v {
		methods.insert(key.to_owned(), val.wrap());
	}
	
	Attribute::new(Identifier { name: "vector".to_owned(), id: global_id(VECTOR_ATTR).wrap() }, methods, ObjectMap::new(), HashSet::new())
}