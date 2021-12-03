
use std::collections::{HashMap, HashSet};

use crate::{interpreter::{Interpreter, value::{Value, macros::cast, primitives::{callable::{Callable, ValCallable}, error::Error, number::Number, object::Object, string::Str}}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

fn sin() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Sin;

	impl Callable for Sin {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			Number::new(n0.sin()).wrap()
    }
	}

	ValCallable::new(Sin.wrap())
}

fn cos() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Cos;
	
	impl Callable for Cos {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			Number::new(n0.cos()).wrap()
		}
	}
	
	ValCallable::new(Cos.wrap())
}

fn tan() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Tan;
	
	impl Callable for Tan {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			Number::new(n0.tan()).wrap()
		}
	}
	
	ValCallable::new(Tan.wrap())
}

fn pow() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Pow;
	
	impl Callable for Pow {
		fn arity(&self) -> usize { 2 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			let n1 = cast!(num args[1].0.clone());
			Number::new(n0.powf(n1)).wrap()
		}
	}
	
	ValCallable::new(Pow.wrap())
}

fn sqrt() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Sqrt;
	
	impl Callable for Sqrt {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			if n0 < 0.0 {
				Error::new(Str::from("sqrt of negative numbers is undefined"))
			} else {
				Number::new(n0.sqrt())
			}.wrap()
		}
	}
	
	ValCallable::new(Sqrt.wrap())
}

fn floor() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Floor;
	
	impl Callable for Floor {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			Number::new(n0.floor()).wrap()
		}
	}
	
	ValCallable::new(Floor.wrap())
}

fn ceil() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Ceil;
	
	impl Callable for Ceil {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			Number::new(n0.ceil()).wrap()
		}
	}
	
	ValCallable::new(Ceil.wrap())
}

fn round() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Round;
	
	impl Callable for Round {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			Number::new(n0.round()).wrap()
		}
	}
	
	ValCallable::new(Round.wrap())
}

fn abs() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Abs;
	
	impl Callable for Abs {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			Number::new(n0.abs()).wrap()
		}
	}
	
	ValCallable::new(Abs.wrap())
}

fn max() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Max;
	
	impl Callable for Max {
		fn arity(&self) -> usize { 2 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			let n1 = cast!(num args[1].0.clone());
			Number::new(n0.max(n1)).wrap()
		}
	}
	
	ValCallable::new(Max.wrap())
}

fn min() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Floor;
	
	impl Callable for Floor {
		fn arity(&self) -> usize { 2 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			let n1 = cast!(num args[1].0.clone());
			Number::new(n0.min(n1)).wrap()
		}
	}
	
	ValCallable::new(Floor.wrap())
}

fn clamp() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Clamp;
	
	impl Callable for Clamp {
		fn arity(&self) -> usize { 3 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			let n1 = cast!(num args[1].0.clone());
			let n2 = cast!(num args[2].0.clone());
			Number::new(n0.clamp(n1, n2)).wrap()
		}
	}
	
	ValCallable::new(Clamp.wrap())
}

fn frac() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Frac;
	
	impl Callable for Frac {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			Number::new(n0.fract()).wrap()
		}
	}
	
	ValCallable::new(Frac.wrap())
}

fn sign() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Sign;
	
	impl Callable for Sign {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			Number::new(n0.signum()).wrap()
		}
	}
	
	ValCallable::new(Sign.wrap())
}

fn lerp() -> Box<dyn Value> {
	#[derive(Clone, Debug)] struct Clamp;
	
	impl Callable for Clamp {
		fn arity(&self) -> usize { 3 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Box<dyn Value>, SourcePos)>) -> Result<Box<dyn Value>> {
			let n0 = cast!(num args[0].0.clone());
			let n1 = cast!(num args[1].0.clone());
			let n2 = cast!(num args[2].0.clone());
			let t = n0.clamp(0.0, 1.0);
			Number::new((1.0 - t) * n1 + t * n2).wrap()
		}
	}
	
	ValCallable::new(Clamp.wrap())
}

pub fn math() -> Box<dyn Value> {
	let mut map = HashMap::new();

	let v = vec![
		("sin", sin()),
		("cos", cos()),
		("tg", tan()),
		("pow", pow()),
		("sqrt", sqrt()),
		("floor", floor()),
		("ceil", ceil()),
		("round", round()),
		("abs", abs()),
		("max", max()),
		("min", min()),
		("clamp", clamp()),
		("frac", frac()),
		("sign", sign()),
		("lerp", lerp()),
		("pi", Number::new(std::f64::consts::PI)),
		("e", Number::new(std::f64::consts::E)),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), val.wrap());
	}

	Object::new(map, HashSet::new())
}
