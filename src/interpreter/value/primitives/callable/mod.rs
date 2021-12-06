
pub mod function;
pub mod nativefn;

use std::{cell::RefCell, fmt::{Debug, Display}, rc::Rc};

use crate::{interpreter::{Interpreter, value::ValueType}, utils::{result::*, source_pos::SourcePos, wrap::Wrap}};

use super::super::Value;

pub trait Callable : Debug {
	fn cloned(&self) -> Box<dyn Callable> { panic!(); }
	fn display(&self) -> String { "<function>".to_owned() }

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
		write!(f, "{}", self.display())
	}
}

impl <T : Callable + 'static> Wrap<Rc<RefCell<dyn Callable>>> for T {
	fn wrap(self) -> Rc<RefCell<(dyn Callable + 'static)>> { Rc::new(RefCell::new(self)) }
}

// impl <T : Callable + 'static> Wrap<Rc<RefCell<Box<dyn Callable>>>> for Box<T> {
// 	fn wrap(self) -> Rc<RefCell<Box<(dyn Callable + 'static)>>> { Rc::new(RefCell::new(self)) }
// }

impl <T : Callable + 'static> Wrap<Rc<RefCell<Box<dyn Callable>>>> for T {
	fn wrap(self) -> Rc<RefCell<Box<(dyn Callable + 'static)>>> { Rc::new(RefCell::new(Box::new(self))) }
}

impl Clone for Box<dyn Callable> {
	fn clone(&self) -> Self { self.cloned() }
}

#[derive(Debug, Clone)]
pub struct ValCallable {
	data: Rc<RefCell<Box<dyn Callable>>>
}

impl ValCallable {
	pub fn create(data: Rc<RefCell<Box<dyn Callable>>>) -> Self {
		Self { data }
	}
	
	pub fn new(data: Rc<RefCell<Box<dyn Callable>>>) -> Box<dyn Value> {
		Self::create(data).wrap()
	}
}

impl Value for ValCallable {
	fn get_type(&self) -> ValueType { ValueType::Callable }
	
	fn to_callable(&self, _pos: SourcePos) -> Result<Rc<RefCell<Box<(dyn Callable + 'static)>>>> { self.data.clone().wrap() }
	
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String> {
		self.data.borrow().to_string().wrap()
	}
	
	fn equ(&self, _other: Box<dyn Value>, _other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool> { false.wrap() }
}
