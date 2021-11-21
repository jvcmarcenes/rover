
use std::fmt::{Debug, Display};

use crate::{interpreter::Interpreter, utils::{result::*, source_pos::SourcePos}};

use super::Value;

pub trait Callable : Debug {
	fn arity(&self) -> usize;
	fn check_arity(&self, args_in: usize, pos: SourcePos) -> Result<()> {
		if self.arity() != args_in {
			return ErrorList::run(format!("Expected {} arguments, but got {}", self.arity(), args_in), pos).err();
		} else {
			Ok(())
		}
	}
	fn call(&mut self, pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value>;
}

impl Display for dyn Callable {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<function>")
	}
}
