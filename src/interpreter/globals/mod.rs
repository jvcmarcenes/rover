
mod math;
mod fs;
pub mod attributes;

use std::{cell::RefCell, collections::{HashMap, HashSet}, io::Write, process, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use ansi_term::Color;
use rand::{SeedableRng, prelude::StdRng};
use text_io::try_read;

use crate::{interpreter::{Interpreter, globals::{attributes::register_default_attr, fs::fs, math::math}, value::{ValueType, macros::{cast, castf}, primitives::{bool::Bool, callable::{Callable, ValCallable}, error::Error, none::ValNone, number::Number, object::Object, string::Str, vector::Vector}}}, resolver::IdentifierData, utils::{result::*, source_pos::SourcePos, wrap::Wrap}};

use super::value::Value;

fn clock() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct Clock;

	impl Callable for Clock {
    fn arity(&self) -> usize { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
			Number::new(now).wrap()
    }
	}

	ValCallable::new(Clock.wrap())
}

fn write() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct Write;
	
	impl Callable for Write {
		fn arity(&self) -> usize { 1 }

		fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let (v0, p0) = args[0].clone(); 
			print!("{}", v0.to_string(interpreter, p0)?);
			let _ = std::io::stdout().flush();
			ValNone.wrap()
		}
	}

	ValCallable::new(Write.wrap())
}

fn writeline() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct Writeline;
	
	impl Callable for Writeline {
		fn arity(&self) -> usize { 1 }

		fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let (v0, p0) = args[0].clone(); 
			println!("{}", v0.to_string(interpreter, p0)?);
			ValNone.wrap()
		}
	}

	ValCallable::new(Writeline.wrap())
}

fn debug() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct Debug;
	
	impl Callable for Debug {
		fn arity(&self) -> usize { 1 }

		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let v0 = args[0].0.clone(); 
			println!("{:?}", v0);
			ValNone.wrap()
		}
	}

	ValCallable::new(Debug.wrap())
}

fn read() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct Read;

	impl Callable for Read {
		fn arity(&self) -> usize { 0 }
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let in_res: std::result::Result<String, text_io::Error> = try_read!("{}\r\n");
			match in_res {
				Ok(str) => Str::new(str),
				Err(_) => Error::new(Str::from("Invalid console input")),
			}.wrap()
		}
	}

	ValCallable::new(Read.wrap())
}

fn random() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct Random;

	impl Callable for Random {
		
		fn check_arity(&self, args_in: usize, pos: SourcePos) -> Result<()> {
			if (0..=1).contains(&args_in) {
				Ok(())
			} else {
				ErrorList::run(format!("Expected 0 or 1 arguments, but got {}", args_in), pos).err()
			}
		}

		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {

			let rng = if args.len() == 1 {
				let n0 = cast!(num args[0].0.clone());
				StdRng::seed_from_u64(n0 as u64)
			} else {
				StdRng::seed_from_u64(rand::random())
			};

			#[derive(Clone, Debug)] struct Rng(StdRng);

			impl Callable for Rng {
				fn arity(&self) -> usize { 0 }
				fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
					Number::new(rand::Rng::gen(&mut self.0)).wrap()
				}
			}

			ValCallable::new(Rng(rng).wrap()).wrap()
		}
	}

	ValCallable::new(Random.wrap())
}

fn is_num() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct IsNum;

	impl Callable for IsNum {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let v0 = args[0].0.clone();
			match v0.get_type() {
				ValueType::Str => Bool::new(castf!(str v0).parse::<f64>().is_ok()),
				_ => Error::new(Str::new(format!("Invalid argument type '{}'", v0.get_type()))),
			}.wrap()
		}
	}

	ValCallable::new(IsNum.wrap())
}

fn to_num() -> Box<dyn Value> {
	#[derive(Debug, Clone)] struct ToNum;

	impl Callable for ToNum {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let v0 = args[0].0.clone();
			match v0.get_type() {
				ValueType::Str => {
					match castf!(str v0).parse::<f64>() {
						Ok(n) => Number::new(n),
						Err(_) => Error::new(Str::from("Cannot convert to number"))
					}
				},
				_ => Error::new(Str::new(format!("Invalid argument type '{}'", v0.get_type()))),
			}.wrap()
		}
	}

	ValCallable::new(ToNum.wrap())
}

fn exit() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Exit;

	impl Callable for Exit {
		fn arity(&self) -> usize { 0 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, _args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			process::exit(0)
    }
	}

	ValCallable::new(Exit.wrap())
}

fn abort() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Exit;

	impl Callable for Exit {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let (v0, p0) = args[0].clone();
			let str = match v0.get_type() {
				ValueType::Error => castf!(err v0).to_string(interpreter, p0)?,
				_ => v0.to_string(interpreter, p0)?,
			};
			eprintln!("{}: {}", Color::Red.paint("error"), str);
			process::exit(0)
    }
	}

	ValCallable::new(Rc::new(RefCell::new(Exit)))
}

fn sleep() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Sleep;

	impl Callable for Sleep {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			let d = std::time::Duration::from_secs_f64(n0);
			std::thread::sleep(d);
			ValNone.wrap()
    }
	}

	ValCallable::new(Sleep.wrap())
}

fn range() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Range;

	impl Callable for Range {
    fn arity(&self) -> usize { 2 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			let n1 = cast!(num args[1].0.clone());
			let mut vec = Vec::new();
			for i in (n0 as i32)..(n1 as i32) { vec.push(Number::new(i as f64)) }
			Vector::new(vec).wrap()
    }
	}

	ValCallable::new(Range.wrap())
}

fn _typeof() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct TypeOf;

	impl Callable for TypeOf {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			Str::new(args[0].0.clone().get_type().to_string()).wrap()
    }
	}

	ValCallable::new(TypeOf.wrap())
}

fn _char() -> Box<dyn Value> {
	let mut map = HashMap::new();

	#[derive(Debug)] struct CharFromCode;

	impl Callable for CharFromCode {
		fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			match char::from_u32(n0 as u32) {
				Some(c) => Str::new(String::from(c)),
				None => Error::new(Str::from("Invalid char code")),
			}.wrap()
    }
	}

	let v = vec![
		("new_line", Str::from("\n")),
		("carriage_return", Str::from("\r")),
		("tab", Str::from("\t")),
		("null", Str::from("\0")),
		("from_code", ValCallable::new(CharFromCode.wrap())),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), val.wrap());
	}

	Object::new(map, HashSet::new())
}

fn paint() -> Box<dyn Value> {
	#[derive(Debug)] struct Paint(Color);

	impl Callable for Paint {
		fn arity(&self) -> usize { 1 }

		fn call(&mut self, _pos: SourcePos, interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let (v0, p0) = args[0].clone();
			Str::new(self.0.paint(v0.to_string(interpreter, p0)?).to_string()).wrap()
    }
	}

	#[derive(Debug)] struct RGBPaint;

	impl Callable for RGBPaint {
		fn arity(&self) -> usize { 3 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			let n1 = cast!(num args[1].0.clone());
			let n2 = cast!(num args[2].0.clone());
			ValCallable::new(Paint(Color::RGB(n0 as u8, n1 as u8, n2 as u8)).wrap()).wrap()
    }
	}

	let mut map = HashMap::new();

	let v = vec![
		("red", ValCallable::new(Paint(Color::Red).wrap())),
		("blue", ValCallable::new(Paint(Color::Blue).wrap())),
		("green", ValCallable::new(Paint(Color::Green).wrap())),
		("yellow", ValCallable::new(Paint(Color::Yellow).wrap())),
		("cyan", ValCallable::new(Paint(Color::Cyan).wrap())),
		("purple", ValCallable::new(Paint(Color::Purple).wrap())),
		("white", ValCallable::new(Paint(Color::White).wrap())),
		("black", ValCallable::new(Paint(Color::Black).wrap())),
		("rgb", ValCallable::new(RGBPaint.wrap())),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), val.wrap());
	}

	Object::new(map, HashSet::new())
}

fn is_err() -> Box<dyn Value> {
	#[derive(Debug)] struct IsErr;

	impl Callable for IsErr {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let v0 = args[0].0.clone();
			Bool::new(v0.to_error(SourcePos::new(0, 0)).is_ok()).wrap()
    }
	}

	ValCallable::new(IsErr.wrap())
}

#[derive(Clone, Debug)]
pub struct Globals {
	pub ids: HashMap<String, IdentifierData>,
	pub values: HashMap<usize, Box<dyn Value>>,
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
	
			// system / process		
			("exit", exit()),
			("abort", abort()),
			
			// thread		
			("sleep", sleep()),
			
			// other
			("clock", clock()),
			("range", range()),
			("typeof", _typeof()),
			("random", random()),
			("char", _char()),
			("paint", paint()),
			
			// std lib	
			("math", math()),
			("fs", fs()),
		];

		let attrs = register_default_attr(&mut globals);

		let mut i = 1 + attrs;
		for (key, val) in v {
			globals.ids.insert(key.to_owned(), IdentifierData::new(i, true));
			globals.values.insert(i, val);
			i += 1;
		}
 
		globals
	}

}
