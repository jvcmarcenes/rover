use crate::utils::{wrap::Wrap, result::Result, source_pos::SourcePos};

use super::Value;


#[derive(Clone, Debug)]
pub struct Str {
	data: String,
}

impl Str {
	pub fn new(data: String) -> Self {
		Self { data }
	}
	
	pub fn create(data: String) -> Box<dyn Value> {
		Self::new(data).wrap()
	}
}

impl Value for Str {
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	fn display(&self) -> Result<String> { self.data.clone().wrap() }
	
	fn is_string(&self) -> bool { true }
	fn as_string(&self, _pos: SourcePos) -> Result<Str> { self.clone().wrap() }
	
	fn add(&self, other: Box<dyn Value>, _spos: SourcePos, _opos: SourcePos, _pos: SourcePos) -> Result<Box<dyn Value>> { Str::create(format!("{}{}", self.data, other.display()?)).wrap() }
	fn equ(&self, other: Box<dyn Value>, _spos: SourcePos, opos: SourcePos, _pos: SourcePos) -> Result<bool> { (other.is_string() && self.data == other.as_string(opos)?.data).wrap() }
}
