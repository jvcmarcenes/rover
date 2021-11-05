
use std::{fmt::Debug, process};

use crate::source_pos::SourcePos;

pub type Result<T> = std::result::Result<T, Error>;

pub trait IntoOk {
	type Output;
	fn into_ok(self) -> Result<Self::Output>;
}

impl<T> IntoOk for T {
	type Output = T;

	fn into_ok(self) -> Result<Self::Output> {
		Ok(self)
	}
}

#[derive(Clone, Debug)]
pub struct Error {
	msg: String,
	pos: SourcePos,
}

impl Error {
	pub fn new(msg: String, pos: SourcePos) -> Error {
		Error { msg, pos }
	}

	pub fn throw_and_exit(self, path: &str, stage: &str) -> ! {
		self.throw_and_continue(path, stage);
		process::exit(0);
	}

	pub fn throw_and_continue(self, path: &str, stage: &str) {
		eprintln!("\n{} {}: {}\n",
			ansi_term::Color::Red.paint(format!("{} error", stage)),
			format!("[{}:{}:{}]", path, self.pos.lin, self.pos.col),
			self.msg
		);
		let data = std::fs::read_to_string(path).unwrap();
		let line = data.lines().skip(self.pos.lin as usize - 1).next().unwrap();
		eprintln!("  {} | {}", self.pos.lin, line);
		eprintln!("  {}   {}^",
			" ".repeat(self.pos.lin.to_string().len()),
			" ".repeat(self.pos.col as usize - 1)
		);
	}
}

impl<T> Into<Result<T>> for Error {
	fn into(self) -> Result<T> { Err(self) }
}
