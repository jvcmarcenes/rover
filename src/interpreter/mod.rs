
use crate::{ast::expression::{BinaryData, BinaryOperator, ExprVisitor, Expression, LiteralData, UnaryData, UnaryOperator}, utils::{result::{Result, Error}, wrap::Wrap}};

use self::value::Value;

pub mod value;

pub struct Interpreter;

fn is_truthy(val: &Value) -> bool {
	match val {
		Value::None => false,
		Value::Bool(b) => *b,
		_ => true,
	}
}

impl Interpreter {
	pub fn evaluate(&mut self, expr: Box<Expression>) -> Result<Value> {
		expr.accept(self)
	}
}

impl ExprVisitor<Value> for Interpreter {

	fn literal(&mut self, data: LiteralData) -> Result<Value> {
		let value = match data {
			LiteralData::Str(s) => Value::Str(s),
			LiteralData::Num(n) => Value::Num(n),
			LiteralData::Bool(b) => Value::Bool(b),
		};
		value.wrap()
	}

	fn binary(&mut self, data: BinaryData) -> Result<Value> {
		let pos = data.rhs.pos;
		let lhs = self.evaluate(data.lhs)?;
		let rhs = self.evaluate(data.rhs)?;
		let value = match data.op {
			BinaryOperator::Add => match (&lhs, &rhs) {
				(Value::Str(_), _) | (_, Value::Str(_)) => Value::Str(format!("{}{}", lhs, rhs)),
				(Value::Num(l), Value::Num(r)) => Value::Num(l + r),
				_ => return Error::new("Invalid operation for types".to_owned(), pos).into()
			}
			BinaryOperator::Sub => Value::Num(lhs.to_num(pos)? - rhs.to_num(pos)?),
			BinaryOperator::Mul => Value::Num(lhs.to_num(pos)? * rhs.to_num(pos)?),
			BinaryOperator::Div => Value::Num(lhs.to_num(pos)? / rhs.to_num(pos)?),
			BinaryOperator::Rem => Value::Num(lhs.to_num(pos)? % rhs.to_num(pos)?),
			BinaryOperator::Lst => Value::Bool(lhs.to_num(pos)? < rhs.to_num(pos)?),
			BinaryOperator::Lse => Value::Bool(lhs.to_num(pos)? <= rhs.to_num(pos)?),
			BinaryOperator::Grt => Value::Bool(lhs.to_num(pos)? > rhs.to_num(pos)?),
			BinaryOperator::Gre => Value::Bool(lhs.to_num(pos)? >= rhs.to_num(pos)?),
			BinaryOperator::Equ => Value::Bool(lhs.to_num(pos)? == rhs.to_num(pos)?),
			BinaryOperator::Neq => Value::Bool(lhs.to_num(pos)? != rhs.to_num(pos)?),
			BinaryOperator::And => todo!(),
			BinaryOperator::Or => todo!(),
		};
		value.wrap()
	}

	fn unary(&mut self, data: UnaryData) -> Result<Value> {
		let pos = data.expr.pos;
		let val = self.evaluate(data.expr)?;
		match data.op {
			UnaryOperator::Neg => Value::Num(-val.to_num(pos)?).wrap(),
			UnaryOperator::Not => Value::Bool(!is_truthy(&val)).wrap(),
		}
	}

	fn grouping(&mut self, data: Box<Expression>) -> Result<Value> {
		self.evaluate(data)
	}
}
