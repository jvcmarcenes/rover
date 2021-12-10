
use crate::utils::{wrap::Wrap, source_pos::SourcePos, result::Result};

use super::Value;

#[derive(Clone, Debug)]
pub struct Number {
	pub data: f64,
}

impl Number {
	pub fn new(data: f64) -> Self {
		Self { data }
	}
	
	pub fn create(data: f64) -> Box<dyn Value> {
		Self::new(data).wrap()
	}
}

impl Value for Number {
	fn cloned(&self) -> Box<dyn Value> { self.clone().wrap() }
	fn display(&self) -> Result<String> { self.data.to_string().wrap() }

	fn is_num(&self) -> bool { true }
	fn as_num(&self, _pos: SourcePos) -> Result<Number> { self.clone().wrap() }

	fn add(&self, other: Box<dyn Value>, _spos: SourcePos, opos: SourcePos, _pos: SourcePos) -> Result<Box<dyn Value>> { Number::create(self.data + other.as_num(opos)?.data).wrap() }
	fn sub(&self, other: Box<dyn Value>, _spos: SourcePos, opos: SourcePos, _pos: SourcePos) -> Result<Box<dyn Value>> { Number::create(self.data - other.as_num(opos)?.data).wrap() }
	fn mul(&self, other: Box<dyn Value>, _spos: SourcePos, opos: SourcePos, _pos: SourcePos) -> Result<Box<dyn Value>> { Number::create(self.data * other.as_num(opos)?.data).wrap() }
	fn div(&self, other: Box<dyn Value>, _spos: SourcePos, opos: SourcePos, _pos: SourcePos) -> Result<Box<dyn Value>> { Number::create(self.data / other.as_num(opos)?.data).wrap() }
	fn rem(&self, other: Box<dyn Value>, _spos: SourcePos, opos: SourcePos, _pos: SourcePos) -> Result<Box<dyn Value>> { Number::create(self.data % other.as_num(opos)?.data).wrap() }
	fn equ(&self, other: Box<dyn Value>, _spos: SourcePos, opos: SourcePos, _pos: SourcePos) -> Result<bool> { (other.is_num() && self.data == other.as_num(opos)?.data).wrap() }
	fn cmp(&self, other: Box<dyn Value>, _spos: SourcePos, opos: SourcePos, _pos: SourcePos) -> Result<i8> { let r = self.data - other.as_num(opos)?.data; if r < 0.0 { -1 } else if r > 0.0 { 1 } else { 0 }.wrap() }
}
