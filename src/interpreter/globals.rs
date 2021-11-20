
use std::{cell::RefCell, collections::HashMap, io::Write, process, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use text_io::try_read;

use crate::{interpreter::{Interpreter, value::callable::Callable}, resolver::IdentifierData, utils::{result::*, source_pos::SourcePos, wrap::Wrap}};

use super::value::Value;

fn clock() -> Value {
	#[derive(Debug, Clone)] struct Clock;

	impl Callable for Clock {
    fn arity(&self) -> usize { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
			Value::Num(now).wrap()
    }
	}

	Value::Callable(Rc::new(RefCell::new(Clock)))
}

fn write() -> Value {
	#[derive(Debug, Clone)] struct Write;
	
	impl Callable for Write {
		fn arity(&self) -> usize { 1 }

		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			print!("{}", args[0].0);
			let _ = std::io::stdout().flush();
			Value::None.wrap()
		}
	}

	Value::Callable(Rc::new(RefCell::new(Write)))
}

fn writeline() -> Value {
	#[derive(Debug, Clone)] struct Writeline;
	
	impl Callable for Writeline {
		fn arity(&self) -> usize { 1 }

		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			println!("{}", args[0].0);
			Value::None.wrap()
		}
	}

	Value::Callable(Rc::new(RefCell::new(Writeline)))
}

fn read() -> Value {
	#[derive(Debug, Clone)] struct Read;

	impl Callable for Read {
		fn arity(&self) -> usize { 0 }
		fn call(&mut self, pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let in_res: std::result::Result<String, text_io::Error> = try_read!("{}\r\n");
			match in_res {
				Ok(str) => Value::Str(str).wrap(),
				Err(_) => ErrorList::run("Invalid console input".to_owned(), pos).err(),
			}
		}
	}

	Value::Callable(Rc::new(RefCell::new(Read)))
}

fn random() -> Value {
	#[derive(Debug, Clone)] struct Random;

	impl Callable for Random {
		fn arity(&self) -> usize { 0 }
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Value, SourcePos)>) -> Result<Value> {
			Value::Num(rand::random()).wrap()
		}
	}

	Value::Callable(Rc::new(RefCell::new(Random)))
}

fn size() -> Value {
	#[derive(Debug, Clone)] struct Size;

	impl Callable for Size {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let arg = &args[0];
			match &arg.0 {
				Value::List(list) => Value::Num(list.len() as f64).wrap(),
				Value::Str(str) => Value::Num(str.len() as f64).wrap(),
				val => ErrorList::run(format!("Invalid argument type '{}'", val.get_type()), arg.1).err()
			}
    }
	}

	Value::Callable(Rc::new(RefCell::new(Size)))
}

fn is_num() -> Value {
	#[derive(Debug, Clone)] struct IsNum;

	impl Callable for IsNum {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let arg = &args[0];
			match &arg.0 {
				Value::Str(str) => Value::Bool(str.parse::<f64>().is_ok()).wrap(),
				val => ErrorList::run(format!("Invalid argument type '{}'", val.get_type()), arg.1).err(),
			}
		}
	}

	Value::Callable(Rc::new(RefCell::new(IsNum)))
}

fn to_num() -> Value {
	#[derive(Debug, Clone)] struct ToNum;

	impl Callable for ToNum {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let arg = &args[0];
			match &arg.0 {
				Value::Str(str) => {
					match str.parse::<f64>() {
						Ok(n) => Value::Num(n).wrap(),
						Err(_) => ErrorList::run("Could not convert to number".to_owned(), arg.1).err(),
					}
				},
				val => ErrorList::run(format!("Invalid argument type '{}'", val.get_type()), arg.1).err(),
			}
		}
	}

	Value::Callable(Rc::new(RefCell::new(ToNum)))
}

fn exit() -> Value {
	#[derive(Clone, Debug)] struct Exit;

	impl Callable for Exit {
		fn arity(&self) -> usize { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Value, SourcePos)>) -> Result<Value> {
			process::exit(0)
    }
	}

	Value::Callable(Rc::new(RefCell::new(Exit)))
}

fn sleep() -> Value {
	#[derive(Clone, Debug)] struct Exit;

	impl Callable for Exit {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone();
			let d = std::time::Duration::from_secs_f64(val.to_num(pos)?);
			std::thread::sleep(d);
			Value::None.wrap()
    }
	}

	Value::Callable(Rc::new(RefCell::new(Exit)))
}

fn new_list() -> Value {
	#[derive(Clone, Debug)] struct NewList;

	impl Callable for NewList {
    fn arity(&self) -> usize { 2 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (len, pos) = args[0].clone();
			let len = len.to_num(pos)?;
			if len < 0.0 { return ErrorList::run("list size cannot be negative".to_owned(), pos).err() }
			let def = args[1].clone().0;
			let mut vec = Vec::new();
			for _ in 0..(len as i32) { vec.push(def.clone()) }
			Value::List(vec).wrap()
    }
	}

	Value::Callable(Rc::new(RefCell::new(NewList)))
}

fn _typeof() -> Value {
	#[derive(Clone, Debug)] struct TypeOf;

	impl Callable for TypeOf {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			Value::Str(args[0].clone().0.get_type()).wrap()
    }
	}

	Value::Callable(Rc::new(RefCell::new(TypeOf)))
}

#[derive(Clone, Debug)]
pub struct Globals {
	pub ids: HashMap<String, IdentifierData>,
	pub values: HashMap<usize, Value>,
}

impl Globals {

	pub fn new() -> Self {
		let mut globals = Self {
			ids: HashMap::new(),
			values: HashMap::new(),
		};

		let v = vec![
			("clock", clock()),
			("write", write()),
			("writeline", writeline()),
			("read", read()),
			("random", random()),
			("size", size()),
			("pi", Value::Num(3.141592653589793238462643383279502884197139699)),
			("is_num", is_num()),
			("to_num", to_num()),
			("exit", exit()),
			("sleep", sleep()),
			("new_list", new_list()),
			("typeof", _typeof()),
		];

		let mut i = 1;
		for (key, val) in v {
			globals.ids.insert(key.to_owned(), IdentifierData::new(i, true));
			globals.values.insert(i, val);
			i += 1;
		}
 
		globals
	}

}
