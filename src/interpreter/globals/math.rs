
use std::collections::HashMap;

use crate::{interpreter::{Interpreter, value::{Value, callable::Callable}}, utils::{new_rcref, result::Result, source_pos::SourcePos, wrap::Wrap}};

fn sin() -> Value {
	#[derive(Clone, Debug)] struct Sin;

	impl Callable for Sin {
    fn arity(&self) -> usize { 1 }

    fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone();
			Value::Num(val.to_num(pos)?.sin()).wrap()
    }
	}

	Value::Callable(new_rcref(Sin))
}

fn cos() -> Value {
	#[derive(Clone, Debug)] struct Cos;
	
	impl Callable for Cos {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone();
			Value::Num(val.to_num(pos)?.cos()).wrap()
		}
	}
	
	Value::Callable(new_rcref(Cos))
}

fn tan() -> Value {
	#[derive(Clone, Debug)] struct Tan;
	
	impl Callable for Tan {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone();
			Value::Num(val.to_num(pos)?.tan()).wrap()
		}
	}
	
	Value::Callable(new_rcref(Tan))
}

fn pow() -> Value {
	#[derive(Clone, Debug)] struct Pow;
	
	impl Callable for Pow {
		fn arity(&self) -> usize { 2 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (base, base_pos) = args[0].clone();
			let (exp, exp_pos) = args[1].clone();
			Value::Num(base.to_num(base_pos)?.powf(exp.to_num(exp_pos)?)).wrap()
		}
	}
	
	Value::Callable(new_rcref(Pow))
}

fn sqrt() -> Value {
	#[derive(Clone, Debug)] struct Sqrt;
	
	impl Callable for Sqrt {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone();
			Value::Num(val.to_num(pos)?.sqrt()).wrap()
		}
	}
	
	Value::Callable(new_rcref(Sqrt))
}

fn floor() -> Value {
	#[derive(Clone, Debug)] struct Floor;
	
	impl Callable for Floor {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone();
			Value::Num(val.to_num(pos)?.floor()).wrap()
		}
	}
	
	Value::Callable(new_rcref(Floor))
}

fn ceil() -> Value {
	#[derive(Clone, Debug)] struct Ceil;
	
	impl Callable for Ceil {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone();
			Value::Num(val.to_num(pos)?.ceil()).wrap()
		}
	}
	
	Value::Callable(new_rcref(Ceil))
}

fn round() -> Value {
	#[derive(Clone, Debug)] struct Round;
	
	impl Callable for Round {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone();
			Value::Num(val.to_num(pos)?.round()).wrap()
		}
	}
	
	Value::Callable(new_rcref(Round))
}

fn abs() -> Value {
	#[derive(Clone, Debug)] struct Abs;
	
	impl Callable for Abs {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone();
			Value::Num(val.to_num(pos)?.abs()).wrap()
		}
	}
	
	Value::Callable(new_rcref(Abs))
}

fn max() -> Value {
	#[derive(Clone, Debug)] struct Max;
	
	impl Callable for Max {
		fn arity(&self) -> usize { 2 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			let (v1, p1) = args[1].clone();
			Value::Num(v0.to_num(p0)?.max(v1.to_num(p1)?)).wrap()
		}
	}
	
	Value::Callable(new_rcref(Max))
}

fn min() -> Value {
	#[derive(Clone, Debug)] struct Floor;
	
	impl Callable for Floor {
		fn arity(&self) -> usize { 2 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			let (v1, p1) = args[1].clone();
			Value::Num(v0.to_num(p0)?.min(v1.to_num(p1)?)).wrap()
		}
	}
	
	Value::Callable(new_rcref(Floor))
}

fn clamp() -> Value {
	#[derive(Clone, Debug)] struct Clamp;
	
	impl Callable for Clamp {
		fn arity(&self) -> usize { 3 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			let (v1, p1) = args[1].clone();
			let (v2, p2) = args[2].clone();
			Value::Num(v0.to_num(p0)?.clamp(v1.to_num(p1)?, v2.to_num(p2)?)).wrap()
		}
	}
	
	Value::Callable(new_rcref(Clamp))
}

fn frac() -> Value {
	#[derive(Clone, Debug)] struct Frac;
	
	impl Callable for Frac {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (val, pos) = args[0].clone();
			Value::Num(val.to_num(pos)?.fract()).wrap()
		}
	}
	
	Value::Callable(new_rcref(Frac))
}

fn sign() -> Value {
	#[derive(Clone, Debug)] struct Sign;
	
	impl Callable for Sign {
		fn arity(&self) -> usize { 1 }
	
		fn call(&mut self, _pos: SourcePos, _interpreter: &mut Interpreter, args: Vec<(Value, SourcePos)>) -> Result<Value> {
			let (v0, p0) = args[0].clone();
			Value::Num(v0.to_num(p0)?.signum()).wrap()
		}
	}
	
	Value::Callable(new_rcref(Sign))
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
		("sign", sign()),
		("frac", frac()),
		("pi", Value::Num(std::f64::consts::PI)),
		("e", Value::Num(std::f64::consts::E)),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), new_rcref(val));
	}

	Value::Object(map)
}