
use std::fmt::{Debug, Display};

use super::source_pos::SourcePos;

macro_rules! append {
	(ret comp $str:expr, $pos:expr; to $errors:expr) => {{
		$errors.add_comp($str, $pos);
		return $errors.err()
	}};
	(ret run $str:expr, $pos:expr; to $errors:expr) => {
		$errors.add_run($str, $pos);
		return $errors.err()
	};
	(comp $str:expr, $pos:expr; to $errors:expr; dummy $or:expr) => {
		$errors.add_comp($str, $pos);
		$dummy
	};
	($expr:expr; to $errors:expr) => {
		match $expr {
			Ok(expr) => expr,
			Err(err) => {
				$errors.append(err);
				return $errors.err();
			}
		}
	};
	($expr:expr; to $errors:expr; dummy $or:expr) => {
		match $expr {
			Ok(expr) => expr,
			Err(err) => {
				$errors.append(err);
				$or
			}
		}
	};
	($expr:expr; to $errors:expr; or none) => {
		match $expr {
			Ok(expr) => Some(expr),
			Err(err) => {
				$errors.append(err);
				None
			}
		}
	};
	($expr:expr; to $errors:expr; with $sync:stmt; or none) => {
		match $expr {
			Ok(expr) => Some(expr),
			Err(err) => {
				$errors.append(err);
				$sync
				None
			}
		}
	};
	($expr:expr; to $errors:expr; with $sync:stmt) => {
		match $expr {
			Ok(expr) => expr,
			Err(err) => {
				$errors.append(err);
				$sync
				return $errors.err();
			}
		}
	};
}

macro_rules! throw {
	($errors:expr) => {
		if !$errors.is_empty() { return $errors.err() }
	};
}

pub(crate) use append;
pub(crate) use throw;

pub type Result<T> = std::result::Result<T, ErrorList>;

#[derive(Clone, Copy, Debug)]
pub enum Stage { System, Compile, Run }

impl Display for Stage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Stage::System => "system",
			Stage::Compile => "compile",
			Stage::Run => "runtime",
		})
	}
}

#[derive(Clone, Debug)]
struct Error {
	msg: String,
	pos: Option<SourcePos>,
	stage: Stage,
}

impl Error {
	
	fn new(msg: String, pos: Option<SourcePos>, stage: Stage) -> Error {
		Error { msg, pos, stage }
	}

	fn report(&self, path: &str) {
		eprintln!("{} {}: {}",
			ansi_term::Color::Red.bold().paint(format!("{} error", self.stage)),
			if let Some(pos) = self.pos { format!("[{}:{}:{}]", path, pos.lin, pos.col) } else { format!("[{}]", path) },
			self.msg
		);

		if let Some(pos) = self.pos {
			let data = std::fs::read_to_string(path).unwrap();
			let line = data.lines().skip(pos.lin as usize - 1).next().unwrap().replace("\t", " ");
	
			let bar_offset = " ".repeat(pos.lin.to_string().len());
	
			eprintln!(" {} |", bar_offset);
			eprintln!(" {} | {}", pos.lin, line);
			eprintln!(" {} | {}^",
				bar_offset,
				" ".repeat(pos.col as usize - 1),
			);
		}

		eprintln!();
	}

}

#[derive(Debug, Clone)]
pub struct ErrorList(Vec<Error>);

impl ErrorList {
	pub fn new() -> Self { Self(Vec::new()) }
	pub fn from(msg: String, pos: SourcePos, stage: Stage) -> Self { Self(vec![Error::new(msg, Some(pos), stage)]) }
	pub fn sys(msg: String) -> Self { Self(vec![Error::new(msg, None, Stage::System)]) }
	pub fn comp(msg: String, pos: SourcePos) -> Self { Self(vec![Error::new(msg, Some(pos), Stage::Compile)]) }
	pub fn run(msg: String, pos: SourcePos) -> Self { Self(vec![Error::new(msg, Some(pos), Stage::Run)]) }
	pub fn mod_comp(msg: String) -> Self { Self(vec![Error::new(msg, None, Stage::Compile)]) }
	pub fn mod_run(msg: String) -> Self { Self(vec![Error::new(msg, None, Stage::Run)]) }
	pub fn err<T>(self) -> Result<T> { Err(self) }
	pub fn is_empty(&self) -> bool { self.0.is_empty() }
	pub fn add(&mut self, msg: String, pos: SourcePos, stage: Stage) { self.0.push(Error::new(msg, Some(pos), stage)) }
	pub fn add_comp(&mut self, msg: String, pos: SourcePos) { self.0.push(Error::new(msg, Some(pos), Stage::Compile)) }
	pub fn add_mod_comp(&mut self, msg: String) { self.0.push(Error::new(msg, None, Stage::Compile)) }
	pub fn add_run(&mut self, msg: String, pos: SourcePos) { self.0.push(Error::new(msg, Some(pos), Stage::Run)) }
	pub fn append(&mut self, mut err: ErrorList) { self.0.append(&mut err.0) }
	pub fn try_append<T>(&mut self, res: Result<T>) { if let Err(err) = res { self.append(err) } }
	pub fn report(&self, path: &str) { self.0.iter().for_each(|err| err.report(path)) }
	pub fn if_empty<T>(self, ret: T) -> Result<T> { if self.is_empty() { Ok(ret) } else { self.err() } }
}

impl Display for ErrorList {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.0.as_slice() {
			[] => write!(f, "none"),
			[err] => write!(f, "{}", err.msg),
			_ => {
				for err in &self.0 {
					write!(f, "\n{}", err.msg)?;
				}
				write!(f, "\n")
			}
		}
	}
}
