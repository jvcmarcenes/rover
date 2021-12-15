
use crate::utils::{wrap::Wrap, result::Result, source_pos::SourcePos};

use super::Value;

#[derive(Clone, Debug)]
pub struct Function {
	pub ptr: usize,
	// pub arity: u8,
	pub params: Vec<usize>,
}

impl Function {
	pub fn new(ptr: usize, params: Vec<usize>) -> Self {
		Self { ptr, params }
	}

	pub fn create(ptr: usize, params: Vec<usize>) -> Box<dyn Value> {
		Self::new(ptr, params).wrap()
	}
}

impl Value for Function {
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	fn display(&self) -> Result<String> { "<function>".to_owned().wrap() }

	fn is_function(&self) -> bool { true }
	fn as_function(&self, _pos: SourcePos) -> Result<Function> { self.clone().wrap() }
}
