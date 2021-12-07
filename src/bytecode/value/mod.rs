
pub mod number;
pub mod bool;
pub mod none;

use std::fmt::{Debug, Display};

use crate::utils::{wrap::Wrap, result::{Result, ErrorList}, source_pos::SourcePos};

use self::{number::Number, bool::Bool};

pub trait Value : Debug {

	fn cloned(&self) -> Box<dyn Value>;
	fn display(&self) -> String;

	fn is_none(&self) -> bool { false }
	fn is_num(&self) -> bool { false }
	fn as_num(&self, pos: SourcePos) -> Result<Number> { ErrorList::run("Could not convert to number".to_owned(), pos).err() }
	fn is_bool(&self) -> bool { false }
	fn as_bool(&self, pos: SourcePos) -> Result<Bool> { ErrorList::run("Could not convert to bool".to_owned(), pos).err() }

	fn truthy(&self) -> bool { true }
	fn sub(&self, _other: Box<dyn Value>, _spos: SourcePos, _opos: SourcePos, pos: SourcePos) -> Result<Box<dyn Value>> { ErrorList::run("Operation add is not defined".to_owned(), pos).err() }
	fn add(&self, _other: Box<dyn Value>, _spos: SourcePos, _opos: SourcePos, pos: SourcePos) -> Result<Box<dyn Value>> { ErrorList::run("Operation sub is not defined".to_owned(), pos).err() }
	fn mul(&self, _other: Box<dyn Value>, _spos: SourcePos, _opos: SourcePos, pos: SourcePos) -> Result<Box<dyn Value>> { ErrorList::run("Operation mul is not defined".to_owned(), pos).err() }
	fn div(&self, _other: Box<dyn Value>, _spos: SourcePos, _opos: SourcePos, pos: SourcePos) -> Result<Box<dyn Value>> { ErrorList::run("Operation div is not defined".to_owned(), pos).err() }
	fn rem(&self, _other: Box<dyn Value>, _spos: SourcePos, _opos: SourcePos, pos: SourcePos) -> Result<Box<dyn Value>> { ErrorList::run("Operation mod is not defined".to_owned(), pos).err() }
	fn equ(&self, _other: Box<dyn Value>, _spos: SourcePos, _opos: SourcePos, pos: SourcePos) -> Result<bool> { ErrorList::run("Operation equals is not defined".to_owned(), pos).err() }
	fn cmp(&self, _other: Box<dyn Value>, _spos: SourcePos, _opos: SourcePos, pos: SourcePos) -> Result<i8> { ErrorList::run("Operation compare is not defined".to_owned(), pos).err() }

}

impl Clone for Box<dyn Value> {
	fn clone(&self) -> Self { self.cloned() }
}

impl Display for Box<dyn Value> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.display())
	}
}

impl <T: 'static + Value> Wrap<Box<dyn Value>> for T {
	fn wrap(self) -> Box<dyn Value> { Box::new(self) }
}
