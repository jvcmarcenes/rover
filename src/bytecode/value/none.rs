use crate::utils::{wrap::Wrap, source_pos::SourcePos, result::Result};

use super::Value;


#[derive(Clone, Debug)]
pub struct ValNone;

impl ValNone {
	pub fn create() -> Box<dyn Value> { Self.wrap() }
}

impl Value for ValNone {
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	fn display(&self) -> Result<String> { "none".to_string().wrap() }

	fn is_none(&self) -> bool { true }

	fn truthy(&self) -> bool { false }
	fn equ(&self, other: Box<dyn Value>, _spos: SourcePos, _opos: SourcePos, pos: SourcePos) -> Result<bool> { other.is_none().wrap() }
}
