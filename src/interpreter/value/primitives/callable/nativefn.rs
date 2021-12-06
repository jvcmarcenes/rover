
use std::{rc::Rc, cell::RefCell};

use crate::{utils::{result::Result, source_pos::SourcePos, wrap::Wrap}, interpreter::{Interpreter, value::Value}};

use super::{Callable, ValCallable};

#[derive(Debug, Clone)]
pub struct NativeFn {
	function: Rc<RefCell<Box<dyn Callable>>>
}

impl NativeFn {
	pub fn new(function: Rc<RefCell<Box<dyn Callable>>>) -> Self {
		NativeFn { function }
	}

	pub fn create(function: Rc<RefCell<Box<dyn Callable>>>) -> Box<dyn Value> {
		ValCallable::new(Self::new(function).wrap())
	}
}

impl Callable for NativeFn {
	fn cloned(&self) -> Box<dyn Callable> { Box::new(self.clone()) }
	fn display(&self) -> String { "<native function>".to_owned() }
	
	fn arity(&self) -> usize { self.function.borrow().arity() }

	fn check_arity(&self, args_in: usize, pos: SourcePos) -> Result<()> { self.function.borrow().check_arity(args_in, pos) }

	fn bind(&mut self, binding: Box<dyn Value>) { self.function.borrow_mut().bind(binding) }

	fn call(&mut self, pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
		self.function.borrow_mut().call(pos, interpreter, args)
	}
}
