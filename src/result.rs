
use std::fmt::Debug;

use crate::{source_pos::SourcePos, utils::Wrap};

pub type Result<T> = std::result::Result<T, Error>;

impl<T> Wrap<Result<Option<T>>> for T {
	fn wrap(self) -> Result<Option<T>> { Ok(Some(self)) }
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

	pub fn report(self, path: &str, stage: &str) {
		eprintln!("{} {}: {}",
			ansi_term::Color::Red.bold().paint(format!("{} error", stage)),
			format!("[{}:{}:{}]", path, self.pos.lin, self.pos.col),
			self.msg
		);

		let data = std::fs::read_to_string(path).unwrap();
		let line = data.lines().skip(self.pos.lin as usize - 1).next().unwrap();

		let bar_offset = " ".repeat(self.pos.lin.to_string().len());

		eprintln!(" {} |", bar_offset);
		eprintln!(" {} | {}", self.pos.lin, line);
		eprintln!(" {} | {}^",
			bar_offset,
			" ".repeat(self.pos.col as usize - 1),
		);
		eprintln!();
	}

	#[allow(dead_code)]
	pub fn print_lines(self, path: &str, stage: &str) {
		eprintln!("\n{} {}: {}\n",
			ansi_term::Color::Red.bold().paint(format!("{} error", stage)),
			format!("[{}:{}:{}]", path, self.pos.lin, self.pos.col),
			self.msg
		);
		let data = std::fs::read_to_string(path).unwrap();

		if self.pos.lin > 1 {
			let prev_line = data.lines().skip(self.pos.lin as usize - 2).next().unwrap();
			eprintln!(" {} | {}", self.pos.lin - 1, prev_line);
		}

		let line = data.lines().skip(self.pos.lin as usize - 1).next().unwrap();
		eprintln!(" {} | {}", self.pos.lin, line);
		eprintln!(" {}   {}^",
			" ".repeat(self.pos.lin.to_string().len()),
			" ".repeat(self.pos.col as usize - 1)
		);
	
		if let Some(next_line) = data.lines().skip(self.pos.lin as usize).next() {
			eprintln!(" {} | {}\n", self.pos.lin + 1, next_line);
		}
	}
	
}

impl<T> Into<Result<T>> for Error {
	fn into(self) -> Result<T> { Err(self) }
}
