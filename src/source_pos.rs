
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
