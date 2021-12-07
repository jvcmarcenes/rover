
use crate::utils::{wrap::Wrap, source_pos::SourcePos, result::Result};

use super::Value;

#[derive(Clone, Debug)]
pub struct Bool {
	pub data: bool,
}

impl Bool {
	pub fn new(data: bool) -> Self {
		Self { data }
	}
	
	pub fn create(data: bool) -> Box<dyn Value> {
		Self::new(data).wrap()
	}
}

impl Value for Bool {
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	fn display(&self) -> String { self.data.to_string() }

	fn is_bool(&self) -> bool { true }
	fn as_bool(&self, pos: SourcePos) -> Result<Bool> { self.clone().wrap() }

	fn truthy(&self) -> bool { self.data }
	fn equ(&self, other: Box<dyn Value>, _spos: SourcePos, _opos: SourcePos, pos: SourcePos) -> Result<bool> { (self.data == other.truthy()).wrap() }
}
