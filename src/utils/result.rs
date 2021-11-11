
use std::fmt::Debug;

use super::{source_pos::SourcePos};

pub type Result<T> = std::result::Result<T, ErrorList>;

#[derive(Clone, Debug)]
struct Error {
	msg: String,
	pos: SourcePos,
}

impl Error {
	
	fn new(msg: String, pos: SourcePos) -> Error {
		Error { msg, pos }
	}

	fn report(&self, path: &str, stage: &str) {
		eprintln!("{} {}: {}",
			ansi_term::Color::Red.bold().paint(format!("{} error", stage)),
			format!("[{}:{}:{}]", path, self.pos.lin, self.pos.col),
			self.msg
		);

		let data = std::fs::read_to_string(path).unwrap();
		let line = data.lines().skip(self.pos.lin as usize - 1).next().unwrap().replace("\t", " ");

		let bar_offset = " ".repeat(self.pos.lin.to_string().len());

		eprintln!(" {} |", bar_offset);
		eprintln!(" {} | {}", self.pos.lin, line);
		eprintln!(" {} | {}^",
			bar_offset,
			" ".repeat(self.pos.col as usize - 1),
		);
		eprintln!();
	}

	fn report_repl(&self, line: &str, stage: &str) {
		eprintln!("{}: {}",
			ansi_term::Color::Red.bold().paint(format!("{} error", stage)),
			self.msg
		);
		eprintln!(" |");
		eprintln!(" | {}", line.trim_end_matches(|c| c == '\n' || c == '\r').replace("\t", " "));
		eprintln!(" | {}^", " ".repeat(self.pos.col as usize - 1));
		eprintln!();
	}
	
}

#[derive(Debug, Clone)]
pub struct ErrorList(Vec<Error>);

impl ErrorList {
	pub fn empty() -> Self { Self(Vec::new()) }
	pub fn new(msg: String, pos: SourcePos) -> Self { Self(vec![Error::new(msg, pos)]) }
	pub fn err<T>(self) -> Result<T> { Err(self) }
	pub fn is_empty(&self) -> bool { self.0.is_empty() }
	pub fn add(&mut self, msg: String, pos: SourcePos) { self.0.push(Error::new(msg, pos)) }
	pub fn append(&mut self, mut err: ErrorList) { self.0.append(&mut err.0) }
	pub fn report(&self, path: &str, stage: &str) { self.0.iter().for_each(|err| err.report(path, stage)) }
	pub fn report_repl(&self, path: &str, stage: &str) { self.0.iter().for_each(|err| err.report_repl(path, stage)) }
}
