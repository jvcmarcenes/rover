
use std::{cell::RefCell, io::Write, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use text_io::try_read;

use crate::{interpreter::{Interpreter, value::callable::Callable}, utils::{result::*, source_pos::SourcePos, wrap::Wrap}};

use super::{environment::{Environment, ValueMap}, value::Value};

fn clock() -> Value {
	#[derive(Debug, Clone)] struct Clock;

	impl Callable for Clock {
    fn arity(&self) -> u8 { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<Value>) -> Result<Value> {
			let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
			Value::Num(now).wrap()
    }
	}

	Value::Callable(Rc::new(RefCell::new(Clock)))
}

fn write() -> Value {
	#[derive(Debug, Clone)] struct Write;
	
	impl Callable for Write {
		fn arity(&self) -> u8 { 1 }

		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value> {
			print!("{}", args[0]);
			let _ = std::io::stdout().flush();
			Value::None.wrap()
		}
	}

	Value::Callable(Rc::new(RefCell::new(Write)))
}

fn writeline() -> Value {
	#[derive(Debug, Clone)] struct Writeline;
	
	impl Callable for Writeline {
		fn arity(&self) -> u8 { 1 }

		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value> {
			println!("{}", args[0]);
			Value::None.wrap()
		}
	}

	Value::Callable(Rc::new(RefCell::new(Writeline)))
}

fn read() -> Value {
	#[derive(Debug, Clone)] struct Read;

	impl Callable for Read {
		fn arity(&self) -> u8 { 0 }
		fn call(&mut self, pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<Value>) -> Result<Value> {
			let in_res: std::result::Result<String, text_io::Error> = try_read!("{}\r\n");
			match in_res {
				Ok(str) => Value::Str(str).wrap(),
				Err(_) => ErrorList::new("Invalid console input".to_owned(), pos).err(),
			}
		}
	}

	Value::Callable(Rc::new(RefCell::new(Read)))
}

fn readnum() -> Value {
	#[derive(Debug, Clone)] struct ReadNum;

	impl Callable for ReadNum {
		fn arity(&self) -> u8 { 0 }
		fn call(&mut self, pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<Value>) -> Result<Value> {
			let in_res: std::result::Result<f64, text_io::Error> = try_read!();
			match in_res {
				Ok(n) => Value::Num(n).wrap(),
				Err(_) => ErrorList::new("Invalid console input".to_owned(), pos).err(),
			}
		}
	}

	Value::Callable(Rc::new(RefCell::new(ReadNum)))
}

fn random() -> Value {
	#[derive(Debug, Clone)] struct Random;

	impl Callable for Random {
		fn arity(&self) -> u8 { 0 }
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<Value>) -> Result<Value> {
			Value::Num(rand::random()).wrap()
		}
	}

	Value::Callable(Rc::new(RefCell::new(Random)))
}

fn size() -> Value {
	#[derive(Debug, Clone)] struct Size;

	impl Callable for Size {
		fn arity(&self) -> u8 { 1 }

    fn call(&mut self, pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value> {
			match &args[0] {
				Value::List(list) => Value::Num(list.len() as f64).wrap(),
				_ => ErrorList::new("Invalid argument type".to_owned(), pos).err()
			}
    }
	}

	Value::Callable(Rc::new(RefCell::new(Size)))
}

pub(super) fn globals() -> Environment {
	let mut env = Environment::new();

	env.definef("clock", clock());
	env.definef("write", write());
	env.definef("writeline", writeline());
	env.definef("read", read());
	env.definef("readnum", readnum());
	env.definef("random", random());
	env.definef("size", size());
	env.definef("pi", Value::Num(3.141592653589793238462643383279502884197139699));

	env.push(ValueMap::new());
	env
}
