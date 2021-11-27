
use std::{collections::HashMap, fs::OpenOptions, io::Write, path::{Path, PathBuf}};

use crate::{interpreter::{Interpreter, value::{Value, callable::Callable}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

fn wipe(path: &PathBuf) -> Value {
	#[derive(Debug)] struct WipeFile(PathBuf);

	impl Callable for WipeFile {
    fn arity(&self) -> usize { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Value, SourcePos)>) -> Result<Value> {
			match OpenOptions::new().write(true).open(&self.0) {
				Ok(file) => {
					println!("wiping");
					if let Err(err) = file.set_len(0) {
						Value::Error(Value::Str(err.to_string()).wrap())
					} else {
						Value::None
					}
				},
				Err(err) => Value::Error(Value::Str(err.to_string()).wrap())
			}.wrap()
    }
	}

	Value::Callable(WipeFile(path.clone()).wrap())
}

fn writeline(path: &PathBuf) -> Value {
	#[derive(Debug)] struct WriteLineFile(PathBuf);

	impl Callable for WriteLineFile {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			let text = v0.to_string(interpreter, p0)?;
			match OpenOptions::new().write(true).open(&self.0) {
				Ok(mut file) => {
					let original = std::fs::read_to_string(&self.0).unwrap();
					if let Err(err) = writeln!(file, "{}{}", original, text) {
						Value::Error(Value::Str(err.to_string()).wrap())
					} else {
						Value::None
					}
				},
				Err(err) => Value::Error(Value::Str(err.to_string()).wrap()),
			}.wrap()
    }
	}

	Value::Callable(WriteLineFile(path.clone()).wrap())
}

fn write(path: &PathBuf) -> Value {
	#[derive(Debug)] struct WriteFile(PathBuf);

	impl Callable for WriteFile {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			let text = v0.to_string(interpreter, p0)?;
			match OpenOptions::new().write(true).open(&self.0) {
				Ok(mut file) => {
					let original = std::fs::read_to_string(&self.0).unwrap();
					if let Err(err) = write!(file, "{}{}", original, text) {
						Value::Error(Value::Str(err.to_string()).wrap())
					} else {
						Value::None
					}
				},
				Err(err) => Value::Error(Value::Str(err.to_string()).wrap())
			}.wrap()
    }
	}

	Value::Callable(WriteFile(path.clone()).wrap())
}

fn read(path: &PathBuf) -> Value {
	#[derive(Debug)] struct ReadFile(PathBuf);	

	impl Callable for ReadFile {
    fn arity(&self) -> usize { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Value, SourcePos)>) -> Result<Value> {
			match std::fs::read_to_string(&self.0) {
				Ok(str) => Value::Str(str),
				Err(err) => Value::Error(Value::Str(err.to_string()).wrap()),
			}.wrap()
    }
	}

	Value::Callable(ReadFile(path.clone()).wrap())
}

fn new_file(path: PathBuf) -> Value {
	let mut map = HashMap::new();

	let mut path_str = path.canonicalize().unwrap().to_str().unwrap().to_owned();
	if let Some(str) = path_str.strip_prefix("\\\\?\\") { path_str = str.to_owned(); }

	let v = vec![
		("path", Value::Str(path_str)),
		("read", read(&path)),
		("write", write(&path)),
		("writeline", writeline(&path)),
		("wipe", wipe(&path)),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), val.wrap());
	}

	Value::Object(map)
}

pub fn open() -> Value {
	#[derive(Debug)] struct Open;

	impl Callable for Open {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			let path_str = v0.to_string(interpreter, p0)?;

			let mut path = interpreter.root_path.clone();
			path.push(path_str);

			if Path::exists(&path) {
				new_file(path.to_path_buf())
			} else {
				Value::Error(Value::Str("File not found".to_owned()).wrap())
			}.wrap()
    }
	}

	Value::Callable(Open.wrap())
}

pub fn create() -> Value {
	#[derive(Debug)] struct Create;

	impl Callable for Create {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			let path_str = v0.to_string(interpreter, p0)?;

			let mut path = interpreter.root_path.clone();
			path.push(path_str);

			if let Err(err) = std::fs::File::create(&path) {
				Value::Error(Value::Str(err.to_string()).wrap())
			} else {
				new_file(path.to_path_buf())
			}.wrap()
    }
	}

	Value::Callable(Create.wrap())
}

pub fn exists() -> Value {
	#[derive(Debug)] struct Exists;

	impl Callable for Exists {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			let path_str = v0.to_string(interpreter, p0)?;
			let mut path = interpreter.root_path.clone();
			path.push(path_str);
			Value::Bool(path.exists()).wrap()
    }
	}

	Value::Callable(Exists.wrap())
}

pub fn fs() -> Value {
	let mut map = HashMap::new();

	let v = vec![
		("open", open()),
		("create", create()),
		("exists", exists()),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), val.wrap());
	}

	Value::Object(map)
}
