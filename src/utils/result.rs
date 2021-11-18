
use std::fmt::{Debug, Display};

use super::{source_pos::SourcePos};

pub type Result<T> = std::result::Result<T, ErrorList>;

#[derive(Clone, Debug)]
pub enum Stage { Compile, Run }

impl Display for Stage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Stage::Compile => "compile",
			Stage::Run => "run",
		})
	}
}

#[derive(Clone, Debug)]
struct Error {
	msg: String,
	pos: SourcePos,
	stage: Stage,
}

impl Error {
	
	fn new(msg: String, pos: SourcePos, stage: Stage) -> Error {
		Error { msg, pos, stage }
	}

	fn report(&self, path: &str) {
		eprintln!("{} {}: {}",
			ansi_term::Color::Red.bold().paint(format!("{} error", self.stage)),
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

	fn report_repl(&self, line: &str) {
		eprintln!("{}: {}",
			ansi_term::Color::Red.bold().paint(format!("{} error", self.stage)),
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
	pub fn new(msg: String, pos: SourcePos, stage: Stage) -> Self { Self(vec![Error::new(msg, pos, stage)]) }
	pub fn comp(msg: String, pos: SourcePos) -> Self { Self(vec![Error::new(msg, pos, Stage::Compile)]) }
	pub fn run(msg: String, pos: SourcePos) -> Self { Self(vec![Error::new(msg, pos, Stage::Run)]) }
	pub fn err<T>(self) -> Result<T> { Err(self) }
	pub fn is_empty(&self) -> bool { self.0.is_empty() }
	pub fn add(&mut self, msg: String, pos: SourcePos, stage: Stage) { self.0.push(Error::new(msg, pos, stage)) }
	pub fn add_comp(&mut self, msg: String, pos: SourcePos) { self.0.push(Error::new(msg, pos, Stage::Compile)) }
	pub fn add_run(&mut self, msg: String, pos: SourcePos) { self.0.push(Error::new(msg, pos, Stage::Run)) }
	pub fn append(&mut self, mut err: ErrorList) { self.0.append(&mut err.0) }
	pub fn report(&self, path: &str) { self.0.iter().for_each(|err| err.report(path)) }
	pub fn report_repl(&self, path: &str) { self.0.iter().for_each(|err| err.report_repl(path)) }
}
