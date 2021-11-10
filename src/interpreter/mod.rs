
pub mod value;
pub mod environment;

use crate::{ast::{expression::*, statement::*}, utils::{result::{Result, ErrorList}, source_pos::SourcePos, wrap::Wrap}};

use self::{environment::Environment, value::Value};

pub struct Interpreter {
	env: Environment,
}

fn is_truthy(val: &Value) -> bool {
	match val {
		Value::None => false,
		Value::Bool(b) => *b,
		_ => true,
	}
}

impl Interpreter {

	pub fn new() -> Self {
		Self { env: Environment::new() }
	}

	pub fn interpret(&mut self, statements: Vec<Statement>) -> Result<()> {
		for stmt in statements { stmt.accept(self)?; }
		Ok(())
	}

	pub fn execute_block(&mut self, statements: Vec<Statement>, env: Environment) -> Result<()> {
		let prev = self.env.clone();
		self.env = env;
		for stmt in statements {
			if let err @ Err(_) = stmt.accept(self) {
				self.env = prev;
				return err
			}
		}
		self.env = prev;
		Ok(())
	}

}

impl ExprVisitor<Value> for Interpreter {

	fn literal(&mut self, data: LiteralData, _pos: SourcePos) -> Result<Value> {
		let value = match data {
			LiteralData::Str(s) => Value::Str(s),
			LiteralData::Num(n) => Value::Num(n),
			LiteralData::Bool(b) => Value::Bool(b),
			LiteralData::None => Value::None,
		};
		value.wrap()
	}

	fn binary(&mut self, data: BinaryData, _pos: SourcePos) -> Result<Value> {
		let (l_pos, r_pos) = (data.lhs.pos, data.rhs.pos);
		let lhs = data.lhs.accept(self)?;
		let rhs = data.rhs.accept(self)?;
		let value = match data.op {
			BinaryOperator::Add => match (&lhs, &rhs) {
				(Value::Str(_), _) | (_, Value::Str(_)) => Value::Str(format!("{}{}", lhs, rhs)),
				(Value::Num(l), Value::Num(r)) => Value::Num(l + r),
				_ => return ErrorList::new("Invalid operation for types".to_owned(), l_pos).err()
			}
			BinaryOperator::Sub => Value::Num(lhs.to_num(l_pos)? - rhs.to_num(r_pos)?),
			BinaryOperator::Mul => Value::Num(lhs.to_num(l_pos)? * rhs.to_num(r_pos)?),
			BinaryOperator::Div => if rhs.to_num(l_pos)? == 0.0 {
				return ErrorList::new("Cannot divide by zero".to_owned(), r_pos).err()
			} else {
				Value::Num(lhs.to_num(l_pos)? / rhs.to_num(r_pos)?)
			},
			BinaryOperator::Rem => Value::Num(lhs.to_num(l_pos)? % rhs.to_num(r_pos)?),
			BinaryOperator::Lst => Value::Bool(lhs.to_num(l_pos)? < rhs.to_num(r_pos)?),
			BinaryOperator::Lse => Value::Bool(lhs.to_num(l_pos)? <= rhs.to_num(r_pos)?),
			BinaryOperator::Grt => Value::Bool(lhs.to_num(l_pos)? > rhs.to_num(r_pos)?),
			BinaryOperator::Gre => Value::Bool(lhs.to_num(l_pos)? >= rhs.to_num(r_pos)?),
			BinaryOperator::Equ => Value::Bool(lhs.to_num(l_pos)? == rhs.to_num(r_pos)?),
			BinaryOperator::Neq => Value::Bool(lhs.to_num(l_pos)? != rhs.to_num(r_pos)?),
			BinaryOperator::And => todo!(),
			BinaryOperator::Or => todo!(),
		};
		value.wrap()
	}

	fn unary(&mut self, data: UnaryData, _pos: SourcePos) -> Result<Value> {
		let pos = data.expr.pos;
		let val = data.expr.accept(self)?;
		match data.op {
			UnaryOperator::Neg => Value::Num(-val.to_num(pos)?).wrap(),
			UnaryOperator::Not => Value::Bool(!is_truthy(&val)).wrap(),
		}
	}

	fn grouping(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<Value> {
		data.accept(self)
	}

	fn variable(&mut self, data: String, pos: SourcePos) -> Result<Value> {
		self.env.get(&data, pos)
	}

}

impl StmtVisitor<()> for Interpreter {

	fn writeline(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<()> {
		let val = data.accept(self)?;
		println!("{}", val);
		Ok(())
	}

	fn declaration(&mut self, data: DeclarationData, _pos: SourcePos) -> Result<()> {
		let val = data.expr.accept(self)?;
		self.env.define(data.name, val);
		Ok(())
	}

	fn assignment(&mut self, data: AssignData, pos: SourcePos) -> Result<()> {
		let val = data.expr.accept(self)?;
		self.env.assign(data.name, val, pos)?;
		Ok(())
	}

	fn block(&mut self, data: Vec<Statement>, _pos: SourcePos) -> Result<()> {
		self.execute_block(data, Environment::with_parent(self.env.clone()))?;
		Ok(())
	}

}
