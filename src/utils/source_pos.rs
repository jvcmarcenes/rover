
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SourcePos {
	pub lin: u32,
	pub col: u32,
}

impl SourcePos {
	pub fn new(lin: u32, col: u32) -> Self {
		Self { lin, col }
	}
}

impl Display for SourcePos {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "@({}, {})", self.lin, self.col)
	}
}
