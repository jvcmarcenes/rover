
pub mod function;

use std::{cell::RefCell, fmt::{Debug, Display}, rc::Rc};

use crate::{interpreter::{Interpreter, value::ValueType}, utils::{result::*, source_pos::SourcePos, wrap::Wrap}};

use super::super::Value;

pub trait Callable : Debug {
	fn arity(&self) -> usize { 0 }
	
	fn check_arity(&self, args_in: usize, pos: SourcePos) -> Result<()> {
		if self.arity() == args_in {
			Ok(())
		} else {
			ErrorList::run(format!("Expected {} arguments, but got {}", self.arity(), args_in), pos).err()
		}
	}
	
	fn bind(&mut self, _binding: Box<dyn Value>) { }
	
	fn call(&mut self, pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>>;
}

impl Display for dyn Callable {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<function>")
	}
}

impl <T : Callable + 'static> Wrap<Rc<RefCell<dyn Callable>>> for T {
	fn wrap(self) -> Rc<RefCell<dyn Callable>> { Rc::new(RefCell::new(self)) }
}

#[derive(Debug, Clone)]
pub struct ValCallable {
	data: Rc<RefCell<dyn Callable>>
}

impl ValCallable {
	pub fn create(data: Rc<RefCell<dyn Callable>>) -> Self {
		Self { data }
	}

	pub fn new(data: Rc<RefCell<dyn Callable>>) -> Box<dyn Value> {
		Self::create(data).wrap()
	}
}

impl Value for ValCallable {
	fn get_type(&self) -> ValueType { ValueType::Callable }
	
	fn to_callable(&self, _pos: SourcePos) -> Result<Rc<RefCell<dyn Callable>>> { self.data.clone().wrap() }
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String> {
		"<function>".to_owned().wrap()
	}

	fn equ(&self, _other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> { false.wrap() }
}

impl <T : Callable + 'static> Wrap<ValCallable> for T {
	fn wrap(self) -> ValCallable {
		ValCallable::create(self.wrap())
	}
}
