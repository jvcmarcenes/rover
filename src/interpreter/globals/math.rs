
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

pub fn math() -> Value {
	let mut map = HashMap::new();

	let v = vec![
		("sin", sin()),
		("cos", cos()),
		("tg", tan()),
		("pow", pow()),
		("sqrt", sqrt()),
		("pi", Value::Num(3.141592653589793238462643383279502884197139699)),
	];

	for (key, val) in v {
		map.insert(key.to_owned(), new_rcref(val));
	}

	Value::Object(map)
	
}