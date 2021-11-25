
mod math;
mod fs;

use std::{cell::RefCell, collections::HashMap, io::Write, process, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use ansi_term::Color;
use rand::{SeedableRng, prelude::StdRng};
use text_io::try_read;

use crate::{interpreter::{Interpreter, globals::{fs::fs, math::math}, value::callable::Callable}, resolver::IdentifierData, utils::{result::*, source_pos::SourcePos, wrap::Wrap}};

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

		fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone(); 
			print!("{}", val.to_string(interpreter, pos)?);
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

		fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone(); 
			println!("{}", val.to_string(interpreter, pos)?);
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
		
		fn check_arity(&self, args_in: usize, pos: SourcePos) -> Result<()> {
			if (0..=1).contains(&args_in) {
				Ok(())
			} else {
				ErrorList::run(format!("Expected 0 or 1 arguments, but got {}", args_in), pos).err()
			}
		}

		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {

			let rng = if args.len() == 1 {
				let (val, pos) = args[0].clone();
				StdRng::seed_from_u64(val.to_num(pos)? as u64)
			} else {
				StdRng::seed_from_u64(rand::random())
			};

			#[derive(Clone, Debug)] struct Rng(StdRng);

			impl Callable for Rng {
				fn arity(&self) -> usize { 0 }
				fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Value, SourcePos)>) -> Result<Value> {
					Value::Num(rand::Rng::gen(&mut self.0)).wrap()
				}
			}

			Value::Callable(Rng(rng).wrap()).wrap()
		}
	}

	Value::Callable(Random.wrap())
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

fn _char() -> Value {
	let mut map = HashMap::new();

	#[derive(Debug)] struct CharFromCode;

	impl Callable for CharFromCode {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			match char::from_u32(v0.to_num(p0)? as u32) {
				Some(c) => Value::Str(String::from(c)).wrap(),
				None => ErrorList::run("Invalid char code".to_owned(), p0).err(),
			}
    }
	}

	let v = vec![
		("new_line", Value::Str("\n".to_owned())),
		("carriage_return", Value::Str("\r".to_owned())),
		("tab", Value::Str("\t".to_owned())),
		("null", Value::Str("\0".to_owned())),
		("from_code", Value::Callable(CharFromCode.wrap())),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), val.wrap());
	}

	Value::Object(map)
}

fn paint() -> Value {
	#[derive(Debug)] struct Paint(Color);

	impl Callable for Paint {
		fn arity(&self) -> usize { 1 }

		fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			Value::Str(self.0.paint(v0.to_string(interpreter, p0)?).to_string()).wrap()
    }
	}

	#[derive(Debug)] struct RGBPaint;

	impl Callable for RGBPaint {
		fn arity(&self) -> usize { 3 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			let (v1, p1) = args[1].clone();
			let (v2, p2) = args[2].clone();
			Value::Callable(Paint(Color::RGB(v0.to_num(p0)? as u8, v1.to_num(p1)? as u8, v2.to_num(p2)? as u8)).wrap()).wrap()
    }
	}

	let mut map = HashMap::new();

	let v = vec![
		("red", Value::Callable(Paint(Color::Red).wrap())),
		("blue", Value::Callable(Paint(Color::Blue).wrap())),
		("green", Value::Callable(Paint(Color::Green).wrap())),
		("yellow", Value::Callable(Paint(Color::Yellow).wrap())),
		("cyan", Value::Callable(Paint(Color::Cyan).wrap())),
		("purple", Value::Callable(Paint(Color::Purple).wrap())),
		("white", Value::Callable(Paint(Color::White).wrap())),
		("black", Value::Callable(Paint(Color::Black).wrap())),
		("rgb", Value::Callable(RGBPaint.wrap())),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), val.wrap());
	}

	Value::Object(map)
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
			("is_num", is_num()),
			("to_num", to_num()),
			("exit", exit()),
			("sleep", sleep()),
			("new_list", new_list()),
			("typeof", _typeof()),
			("math", math()),
			("fs", fs()),
			("char", _char()),
			("paint", paint()),
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
