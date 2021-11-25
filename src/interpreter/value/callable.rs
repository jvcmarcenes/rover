
use std::{cell::RefCell, fmt::{Debug, Display}, rc::Rc};

use crate::{interpreter::Interpreter, utils::{result::*, source_pos::SourcePos, wrap::Wrap}};

use super::Value;

pub trait Callable : Debug {
	fn arity(&self) -> usize { 0 }

	fn check_arity(&self, args_in: usize, pos: SourcePos) -> Result<()> {
		if self.arity() == args_in {
			Ok(())
		} else {
			ErrorList::run(format!("Expected {} arguments, but got {}", self.arity(), args_in), pos).err()
		}
	}

	fn bind(&mut self, _binding: Value) { }
	
	fn call(&mut self, pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value>;
}

impl Display for dyn Callable {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<function>")
	}
}

impl <T : Callable + 'static> Wrap<Rc<RefCell<dyn Callable>>> for T {
	fn wrap(self) -> Rc<RefCell<dyn Callable>> { Rc::new(RefCell::new(self)) }
}
