
use std::{collections::HashMap, fs::OpenOptions, io::Write, path::{Path, PathBuf}};

use crate::{interpreter::{Interpreter, value::{Value, primitives::{bool::Bool, callable::{Callable, ValCallable}, error::Error, none::ValNone, object::Object, string::Str}}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

fn wipe(path: &PathBuf) -> Box<dyn Value> {
	#[derive(Debug)] struct WipeFile(PathBuf);

	impl Callable for WipeFile {
    fn arity(&self) -> usize { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			match OpenOptions::new().write(true).open(&self.0) {
				Ok(file) => {
					println!("wiping");
					if let Err(err) = file.set_len(0) {
						Error::new(Str::new(err.to_string()))
					} else {
						ValNone::new()
					}
				},
				Err(err) => Error::new(Str::new(err.to_string()))
			}.wrap()
    }
	}

	ValCallable::new(WipeFile(path.clone()).wrap())
}

fn writeline(path: &PathBuf) -> Box<dyn Value> {
	#[derive(Debug)] struct WriteLineFile(PathBuf);

	impl Callable for WriteLineFile {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let (v0, p0) = args[0].clone();
			let text = v0.to_string(interpreter, p0)?;
			match OpenOptions::new().write(true).open(&self.0) {
				Ok(mut file) => {
					let original = std::fs::read_to_string(&self.0).unwrap();
					if let Err(err) = writeln!(file, "{}{}", original, text) {
						Error::new(Str::new(err.to_string()))
					} else {
						ValNone::new()
					}
				},
				Err(err) => Error::new(Str::new(err.to_string())),
			}.wrap()
    }
	}

	ValCallable::new(WriteLineFile(path.clone()).wrap())
}

fn write(path: &PathBuf) -> Box<dyn Value> {
	#[derive(Debug)] struct WriteFile(PathBuf);

	impl Callable for WriteFile {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let (v0, p0) = args[0].clone();
			let text = v0.to_string(interpreter, p0)?;
			match OpenOptions::new().write(true).open(&self.0) {
				Ok(mut file) => {
					let original = std::fs::read_to_string(&self.0).unwrap();
					if let Err(err) = write!(file, "{}{}", original, text) {
						Error::new(Str::new(err.to_string()))
					} else {
						ValNone::new()
					}
				},
				Err(err) => Error::new(Str::new(err.to_string()))
			}.wrap()
    }
	}

	ValCallable::new(WriteFile(path.clone()).wrap())
}

fn read(path: &PathBuf) -> Box<dyn Value> {
	#[derive(Debug)] struct ReadFile(PathBuf);	

	impl Callable for ReadFile {
    fn arity(&self) -> usize { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			match std::fs::read_to_string(&self.0) {
				Ok(str) => Str::new(str),
				Err(err) => Error::new(Str::new(err.to_string())),
			}.wrap()
    }
	}

	ValCallable::new(ReadFile(path.clone()).wrap())
}

fn new_file(path: PathBuf) -> Box<dyn Value> {
	let mut map = HashMap::new();

	let mut path_str = path.canonicalize().unwrap().to_str().unwrap().to_owned();
	if let Some(str) = path_str.strip_prefix("\\\\?\\") { path_str = str.to_owned(); }

	let v = vec![
		("path", Str::new(path_str)),
		("read", read(&path)),
		("write", write(&path)),
		("writeline", writeline(&path)),
		("wipe", wipe(&path)),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), val.wrap());
	}

	Object::new(map)
}

pub fn open() -> Box<dyn Value> {
	#[derive(Debug)] struct Open;

	impl Callable for Open {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let (v0, p0) = args[0].clone();
			let path_str = v0.to_string(interpreter, p0)?;

			let mut path = interpreter.root_path.clone();
			path.push(path_str);

			if Path::exists(&path) {
				new_file(path.to_path_buf())
			} else {
				Error::new(Str::new("File not found".to_owned()))
			}.wrap()
    }
	}

	ValCallable::new(Open.wrap())
}

pub fn create() -> Box<dyn Value> {
	#[derive(Debug)] struct Create;

	impl Callable for Create {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let (v0, p0) = args[0].clone();
			let path_str = v0.to_string(interpreter, p0)?;

			let mut path = interpreter.root_path.clone();
			path.push(path_str);

			if let Err(err) = std::fs::File::create(&path) {
				Error::new(Str::new(err.to_string()))
			} else {
				new_file(path.to_path_buf())
			}.wrap()
    }
	}

	ValCallable::new(Create.wrap())
}

pub fn exists() -> Box<dyn Value> {
	#[derive(Debug)] struct Exists;

	impl Callable for Exists {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let (v0, p0) = args[0].clone();
			let path_str = v0.to_string(interpreter, p0)?;
			let mut path = interpreter.root_path.clone();
			path.push(path_str);
			Bool::new(path.exists()).wrap()
    }
	}

	ValCallable::new(Exists.wrap())
}

pub fn fs() -> Box<dyn Value> {
	let mut map = HashMap::new();

	let v = vec![
		("open", open()),
		("create", create()),
		("exists", exists()),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), val.wrap());
	}

	Object::new(map)
}
