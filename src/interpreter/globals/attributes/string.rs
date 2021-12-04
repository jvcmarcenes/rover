
use std::collections::{HashMap, HashSet};

use crate::{interpreter::{Interpreter, get_index, globals::attributes::NatSelf, value::{Value, macros::{cast, castf}, primitives::{attribute::Attribute, bool::Bool, callable::{Callable, nativefn::NativeFn}, error::Error, none::ValNone, number::Number, string::Str, object::ObjectMap}}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap, global_ids::global_id}, ast::identifier::Identifier};

pub const STRING_ATTR: &str = "String";

fn is_num() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct IsNum(NatSelf);
	
	impl Callable for IsNum {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let str_ref = self.0.clone().unwrap();
			Bool::new(castf!(str str_ref.borrow()).parse::<f64>().is_ok()).wrap()
		}
	}
	
	NativeFn::create(IsNum(None).wrap())
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
	
	NativeFn::create(ToNum(None).wrap())
}

fn size() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Size(NatSelf);
	
	impl Callable for Size {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let str_ref = self.0.clone().unwrap();
			Number::new(castf!(str str_ref.borrow()).len() as f64).wrap()
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
	
	NativeFn::create(Get(None).wrap())
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
		methods.insert(key.to_owned(), val.wrap());
	}
	
	Attribute::new(Identifier { name: "string".to_owned(), id: global_id(STRING_ATTR).wrap() }, methods, ObjectMap::new(), HashSet::new())
}