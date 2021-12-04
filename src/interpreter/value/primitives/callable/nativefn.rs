
use std::{rc::Rc, cell::RefCell};

use crate::{utils::{result::Result, source_pos::SourcePos}, interpreter::{Interpreter, value::Value}};

use super::{Callable, ValCallable};

#[derive(Debug, Clone)]
pub struct NativeFn {
	function: Rc<RefCell<dyn Callable>>
}

impl NativeFn {
	pub fn new(function: Rc<RefCell<dyn Callable>>) -> Self {
		NativeFn { function }
	}

	pub fn create(function: Rc<RefCell<dyn Callable>>) -> Box<dyn Value> {
		ValCallable::new(Rc::new(RefCell::new(Box::new(Self::new(function)))))
	}
}

impl Callable for NativeFn {
	fn cloned(&self) -> Box<dyn Callable> { self.function.borrow().cloned() }
	fn display(&self) -> String { "<native function>".to_owned() }
	
	fn arity(&self) -> usize { self.function.borrow().arity() }

	fn check_arity(&self, args_in: usize, pos: SourcePos) -> Result<()> { self.function.borrow().check_arity(args_in, pos) }

	fn bind(&mut self, binding: Box<dyn Value>) { self.function.borrow_mut().bind(binding) }

	fn call(&mut self, pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
		self.function.borrow_mut().call(pos, interpreter, args)
	}
}
