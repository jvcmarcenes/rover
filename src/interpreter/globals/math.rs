
use std::collections::HashMap;

use crate::{interpreter::{Interpreter, value::{Value, callable::Callable, macros::cast}}, utils::{result::Result, source_pos::SourcePos, wrap::Wrap}};

fn sin() -> Value {
	#[derive(Clone, Debug)] struct Sin;

	impl Callable for Sin {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			Value::Num(n0.sin()).wrap()
    }
	}

	Value::Callable(Sin.wrap())
}

fn cos() -> Value {
	#[derive(Clone, Debug)] struct Cos;
	
	impl Callable for Cos {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			Value::Num(n0.cos()).wrap()
		}
	}
	
	Value::Callable(Cos.wrap())
}

fn tan() -> Value {
	#[derive(Clone, Debug)] struct Tan;
	
	impl Callable for Tan {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			Value::Num(n0.tan()).wrap()
		}
	}
	
	Value::Callable(Tan.wrap())
}

fn pow() -> Value {
	#[derive(Clone, Debug)] struct Pow;
	
	impl Callable for Pow {
		fn arity(&self) -> usize { 2 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			let n1 = cast!(num args[1].0);
			Value::Num(n0.powf(n1)).wrap()
		}
	}
	
	Value::Callable(Pow.wrap())
}

fn sqrt() -> Value {
	#[derive(Clone, Debug)] struct Sqrt;
	
	impl Callable for Sqrt {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			if n0 < 0.0 {
				Value::Error(Value::Str("sqrt of negative numbers is undefined".to_owned()).wrap())
			} else {
				Value::Num(n0.sqrt())
			}.wrap()
		}
	}
	
	Value::Callable(Sqrt.wrap())
}

fn floor() -> Value {
	#[derive(Clone, Debug)] struct Floor;
	
	impl Callable for Floor {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			Value::Num(n0.floor()).wrap()
		}
	}
	
	Value::Callable(Floor.wrap())
}

fn ceil() -> Value {
	#[derive(Clone, Debug)] struct Ceil;
	
	impl Callable for Ceil {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			Value::Num(n0.ceil()).wrap()
		}
	}
	
	Value::Callable(Ceil.wrap())
}

fn round() -> Value {
	#[derive(Clone, Debug)] struct Round;
	
	impl Callable for Round {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			Value::Num(n0.round()).wrap()
		}
	}
	
	Value::Callable(Round.wrap())
}

fn abs() -> Value {
	#[derive(Clone, Debug)] struct Abs;
	
	impl Callable for Abs {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			Value::Num(n0.abs()).wrap()
		}
	}
	
	Value::Callable(Abs.wrap())
}

fn max() -> Value {
	#[derive(Clone, Debug)] struct Max;
	
	impl Callable for Max {
		fn arity(&self) -> usize { 2 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			let n1 = cast!(num args[1].0);
			Value::Num(n0.max(n1)).wrap()
		}
	}
	
	Value::Callable(Max.wrap())
}

fn min() -> Value {
	#[derive(Clone, Debug)] struct Floor;
	
	impl Callable for Floor {
		fn arity(&self) -> usize { 2 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			let n1 = cast!(num args[1].0);
			Value::Num(n0.min(n1)).wrap()
		}
	}
	
	Value::Callable(Floor.wrap())
}

fn clamp() -> Value {
	#[derive(Clone, Debug)] struct Clamp;
	
	impl Callable for Clamp {
		fn arity(&self) -> usize { 3 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			let n1 = cast!(num args[1].0);
			let n2 = cast!(num args[2].0);
			Value::Num(n0.clamp(n1, n2)).wrap()
		}
	}
	
	Value::Callable(Clamp.wrap())
}

fn frac() -> Value {
	#[derive(Clone, Debug)] struct Frac;
	
	impl Callable for Frac {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			Value::Num(n0.fract()).wrap()
		}
	}
	
	Value::Callable(Frac.wrap())
}

fn sign() -> Value {
	#[derive(Clone, Debug)] struct Sign;
	
	impl Callable for Sign {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			Value::Num(n0.signum()).wrap()
		}
	}
	
	Value::Callable(Sign.wrap())
}

fn lerp() -> Value {
	#[derive(Clone, Debug)] struct Clamp;
	
	impl Callable for Clamp {
		fn arity(&self) -> usize { 3 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let n0 = cast!(num args[0].0);
			let n1 = cast!(num args[1].0);
			let n2 = cast!(num args[2].0);
			let t = n0.clamp(0.0, 1.0);
			Value::Num((1.0 - t) * n1 + t * n2).wrap()
		}
	}
	
	Value::Callable(Clamp.wrap())
}

pub fn math() -> Value {
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
		("pi", Value::Num(std::f64::consts::PI)),
		("e", Value::Num(std::f64::consts::E)),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), val.wrap());
	}

	Value::Object(map)
}
