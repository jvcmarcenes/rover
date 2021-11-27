
mod math;
mod fs;

use std::{cell::RefCell, collections::HashMap, io::Write, process, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use ansi_term::Color;
use rand::{SeedableRng, prelude::StdRng};
use text_io::try_read;

use crate::{interpreter::{Interpreter, globals::{fs::fs, math::math}, value::{callable::Callable, macros::cast}}, resolver::IdentifierData, utils::{result::*, source_pos::SourcePos, wrap::Wrap}};

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

	Value::Callable(Clock.wrap())
}

fn write() -> Value {
	#[derive(Debug, Clone)] struct Write;
	
	impl Callable for Write {
		fn arity(&self) -> usize { 1 }

		fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone(); 
			print!("{}", v0.to_string(interpreter, p0)?);
			let _ = std::io::stdout().flush();
			Value::None.wrap()
		}
	}

	Value::Callable(Write.wrap())
}

fn writeline() -> Value {
	#[derive(Debug, Clone)] struct Writeline;
	
	impl Callable for Writeline {
		fn arity(&self) -> usize { 1 }

		fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone(); 
			println!("{}", v0.to_string(interpreter, p0)?);
			Value::None.wrap()
		}
	}

	Value::Callable(Writeline.wrap())
}

fn debug() -> Value {
	#[derive(Debug, Clone)] struct Debug;
	
	impl Callable for Debug {
		fn arity(&self) -> usize { 1 }

		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let v0 = args[0].0.clone(); 
			println!("{:?}", v0);
			Value::None.wrap()
		}
	}

	Value::Callable(Debug.wrap())
}

fn read() -> Value {
	#[derive(Debug, Clone)] struct Read;

	impl Callable for Read {
		fn arity(&self) -> usize { 0 }
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let in_res: std::result::Result<String, text_io::Error> = try_read!("{}\r\n");
			match in_res {
				Ok(str) => Value::Str(str),
				Err(_) => Value::Error(Value::Str("Invalid console input".to_owned()).wrap()),
			}.wrap()
		}
	}

	Value::Callable(Read.wrap())
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
				let n0 = cast!(num args[0].0);
				StdRng::seed_from_u64(n0 as u64)
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
			let v0 = args[0].0.clone();
			match v0 {
				Value::List(list) => Value::Num(list.len() as f64),
				Value::Str(str) => Value::Num(str.len() as f64),
				_ => Value::Error(Value::Str(format!("Invalid argument type '{}'", v0.get_type())).wrap())
			}.wrap()
    }
	}

	Value::Callable(Size.wrap())
}

fn is_num() -> Value {
	#[derive(Debug, Clone)] struct IsNum;

	impl Callable for IsNum {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let v0 = args[0].0.clone();
			match v0 {
				Value::Str(str) => Value::Bool(str.parse::<f64>().is_ok()),
				_ => Value::Error(Value::Str(format!("Invalid argument type '{}'", v0.get_type())).wrap()),
			}.wrap()
		}
	}

	Value::Callable(IsNum.wrap())
}

fn to_num() -> Value {
	#[derive(Debug, Clone)] struct ToNum;

	impl Callable for ToNum {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let v0 = args[0].0.clone();
			if let Value::Str(str) = v0 {
				match str.parse::<f64>() {
					Ok(n) => Value::Num(n),
					Err(_) => Value::Error(Value::Str("Could't convert to number".to_owned()).wrap()),
				}
			} else {
    		Value::Error(Value::Str(format!("Invalid argument type '{}'", v0.get_type())).wrap())
			}.wrap()
		}
	}

	Value::Callable(ToNum.wrap())
}

fn exit() -> Value {
	#[derive(Clone, Debug)] struct Exit;

	impl Callable for Exit {
		fn arity(&self) -> usize { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Value, SourcePos)>) -> Result<Value> {
			process::exit(0)
    }
	}

	Value::Callable(Exit.wrap())
}

fn abort() -> Value {
	#[derive(Clone, Debug)] struct Exit;

	impl Callable for Exit {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			let str = if let Value::Error(val) = v0 {
				val.to_string(interpreter, p0)?
			} else {
				v0.to_string(interpreter, p0)?
			};
			eprintln!("{}: {}", Color::Red.paint("error"), str);
			process::exit(0)
    }
	}

	Value::Callable(Rc::new(RefCell::new(Exit)))
}

fn sleep() -> Value {
	#[derive(Clone, Debug)] struct Sleep;

	impl Callable for Sleep {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			let d = std::time::Duration::from_secs_f64(n0);
			std::thread::sleep(d);
			Value::None.wrap()
    }
	}

	Value::Callable(Sleep.wrap())
}

fn range() -> Value {
	#[derive(Clone, Debug)] struct Range;

	impl Callable for Range {
    fn arity(&self) -> usize { 2 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			let n1 = cast!(num args[1].0);
			let mut vec = Vec::new();
			for i in (n0 as i32)..(n1 as i32) { vec.push(Value::Num(i as f64)) }
			Value::List(vec).wrap()
    }
	}

	Value::Callable(Range.wrap())
}

fn _typeof() -> Value {
	#[derive(Clone, Debug)] struct TypeOf;

	impl Callable for TypeOf {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			Value::Str(args[0].clone().0.get_type()).wrap()
    }
	}

	Value::Callable(TypeOf.wrap())
}

fn _char() -> Value {
	let mut map = HashMap::new();

	#[derive(Debug)] struct CharFromCode;

	impl Callable for CharFromCode {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			match char::from_u32(n0 as u32) {
				Some(c) => Value::Str(String::from(c)).wrap(),
				None => Value::Error(Value::Str("Invalid char code".to_owned()).wrap()).wrap(),
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
			let n0 = cast!(num args[0].0);
			let n1 = cast!(num args[1].0);
			let n2 = cast!(num args[2].0);
			Value::Callable(Paint(Color::RGB(n0 as u8, n1 as u8, n2 as u8)).wrap()).wrap()
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

fn is_err() -> Value {
	#[derive(Debug)] struct IsErr;

	impl Callable for IsErr {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let v0 = args[0].0.clone();
			Value::Bool(v0.is_error()).wrap()
    }
	}

	Value::Callable(IsErr.wrap())
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
			// io
			("write", write()),
			("writeline", writeline()),
			("debug", debug()),
			("read", read()),
			
			// to be moved to attributes
			("size", size()),
			("is_num", is_num()),
			("to_num", to_num()),
			
			// system / process		
			("exit", exit()),
			("abort", abort()),
			
			// thread		
			("sleep", sleep()),
			
			// other
			("clock", clock()),
			("range", range()),
			("is_err", is_err()),
			("typeof", _typeof()),
			("random", random()),
			("char", _char()),
			("paint", paint()),
			
			// std lib	
			("math", math()),
			("fs", fs()),
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
