
use std::{cell::RefCell, io::Write, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use text_io::try_read;

use crate::{interpreter::{Interpreter, value::callable::{Callable, NativeCallable}}, utils::{result::*, source_pos::SourcePos, wrap::Wrap}};

use super::{environment::Environment, value::Value};

fn clock() -> Value {
	#[derive(Debug, Clone)] struct Clock;

	impl Callable for Clock {
    fn arity(&self) -> u8 { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<Value>) -> Result<Value> {
			let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
			Value::Num(now).wrap()
    }
	}

	impl NativeCallable for Clock { }

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

	impl NativeCallable for Write { }

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

	impl NativeCallable for Writeline { }

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

	impl NativeCallable for Read { }

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

	impl NativeCallable for ReadNum { }

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

	impl NativeCallable for Random { }

	Value::Callable(Rc::new(RefCell::new(Random)))
}

pub(super) fn globals() -> Environment {
	let mut env = Environment::new();

	env.define("clock", clock());
	env.define("write", write());
	env.define("writeline", writeline());
	env.define("read", read());
	env.define("readnum", readnum());
	env.define("random", random());
	env.define("pi", Value::Num(3.141592653589793238462643383279502884197139699));

	env
}
