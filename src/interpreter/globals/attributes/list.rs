
use std::collections::{HashMap, HashSet};

use crate::{interpreter::{Interpreter, get_index, globals::attributes::NatSelf, value::{Value, macros::{cast, castf}, primitives::{attribute::Attribute, bool::Bool, callable::{Callable, nativefn::NativeFn}, none::ValNone, number::Number, object::ObjectMap, list::List}, ValueType}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap, global_ids::global_id}, ast::identifier::Identifier};

pub const LIST_ATTR: &str = "List";

fn size() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Size(NatSelf);
	
	impl Callable for Size {
		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap() }
		
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let list_ref = self.0.clone().unwrap();
			Number::new(castf!(vec list_ref.borrow()).borrow().len() as f64).wrap()
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
			let list_ref = self.0.clone().unwrap();
			let list = castf!(vec list_ref.borrow());
			let list = list.borrow();
			let i = match get_index(n0, list.len(), p0) {
				Ok(n) => n,
				Err(_) => return ValNone::new().wrap(),
			};
			match list.get(i) {
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
			let list_ref = self.0.clone().unwrap();
			let list = castf!(vec list_ref.borrow());
			list.borrow_mut().push(v0);
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
			let list_ref = self.0.clone().unwrap();
			let list = castf!(vec list_ref.borrow());
			let val = list.borrow_mut().pop();
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
			let list_ref = self.0.clone().unwrap();
			let list = castf!(vec list_ref.borrow());
			for val in list.borrow().iter() {
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
			let list_ref = self.0.clone().unwrap();
			let list = castf!(vec list_ref.borrow());
			let mut rev = list.borrow().clone();
			rev.reverse();
			List::new(rev).wrap()
		}
	}
	
	NativeFn::create(Reverse(None).wrap())
}

fn iter() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Iter(NatSelf);
	
	impl Callable for Iter {
		fn arity(&self) -> usize { 1 }

		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap(); }

		fn call(&mut self, pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let c0 = castf!(fun args[0].0.clone());
			let list_ref = self.0.clone().unwrap();
			let list = castf!(vec list_ref.borrow());
			for val in list.borrow().iter().cloned() {
				c0.borrow_mut().call(pos, interpreter, vec![(val, pos)])?;
			}
			ValNone::new().wrap()
		}
	}
	
	NativeFn::create(Iter(None).wrap())
}

fn map() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Map(NatSelf);
	
	impl Callable for Map {
		fn arity(&self) -> usize { 1 }

		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap(); }

		fn call(&mut self, pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let c0 = castf!(fun args[0].0.clone());
			let list_ref = self.0.clone().unwrap();
			let list = castf!(vec list_ref.borrow());
			let mut mapped = Vec::new();
			for val in list.borrow().iter().cloned() {
				mapped.push({
					let ret = c0.borrow_mut().call(pos, interpreter, vec![(val, pos)])?;
					match ret.get_type() {
						ValueType::Error => return ret.wrap(),
						_ => ret
					}
				});
			}
			List::new(mapped).wrap()
		}
	}
	
	NativeFn::create(Map(None).wrap())
}

fn filter() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Filter(NatSelf);
	
	impl Callable for Filter {
		fn arity(&self) -> usize { 1 }

		fn bind(&mut self, binding: Box<dyn Value>) { self.0 = binding.wrap(); }

		fn call(&mut self, pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let c0 = castf!(fun args[0].0.clone());
			let list_ref = self.0.clone().unwrap();
			let list = castf!(vec list_ref.borrow());
			let mut filtered = Vec::new();
			for val in list.borrow().iter().cloned() {
				let ret = c0.borrow_mut().call(pos, interpreter, vec![(val.clone(), pos)])?;
				match ret.get_type() {
					ValueType::Error => return ret.wrap(),
					_ if ret.is_truthy() => filtered.push(val),
					_ => (),
				}
			}
			List::new(filtered).wrap()
		}
	}
	
	NativeFn::create(Filter(None).wrap())
}

pub fn list() -> Box<dyn Value> {
	let mut methods = HashMap::new();
	
	let v = vec![
	("push", push()),
	("pop", pop()),
	("iter", iter()),
	("map", map()),
	("filter", filter()),
	("size", size()),
	("get", get()),
	("contains", contains()),
	("reverse", reverse()),
	];
	
	for (key, val) in v {
		methods.insert(key.to_owned(), val.wrap());
	}
	
	Attribute::new(Identifier { name: "list".to_owned(), id: global_id(LIST_ATTR).wrap() }, methods, ObjectMap::new(), HashSet::new())
}