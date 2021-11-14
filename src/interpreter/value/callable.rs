
use std::fmt::{Debug, Display};

use dyn_clone::DynClone;

use crate::{interpreter::Interpreter, utils::{result::*, source_pos::SourcePos}};

use super::Value;

pub trait Callable : Debug + DynClone {
	fn arity(&self) -> u8;
	fn call(&mut self, pos: SourcePos, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value>;
}

dyn_clone::clone_trait_object!(Callable);

impl Display for dyn Callable {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<function>")
	}
}
