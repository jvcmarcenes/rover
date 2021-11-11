
pub mod value;
pub mod environment;

use text_io::try_read;

use crate::{ast::{expression::*, statement::*}, utils::{result::{Result, ErrorList}, source_pos::SourcePos, wrap::Wrap}};

use self::{Message::*, environment::{Environment, ValueMap}, value::Value};

enum Message {
	None,
	Break,
	Continue,
}

type MessageResult = Result<Message>;

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

	pub fn interpret(&mut self, statements: Block) -> Result<()> {
		for stmt in statements { stmt.accept(self)?; }
		Ok(())
	}

	fn execute_block(&mut self, block: Block, map: ValueMap) -> MessageResult {
		self.env.push(map);
		for stmt in block {
			match stmt.accept(self)? {
				None => continue,
				msg => {
					self.env.pop();
					return msg.wrap();
				}
			}
		}
		self.env.pop();
		Ok(None)
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
				_ => Value::Num(lhs.to_num(l_pos)? + rhs.to_num(r_pos)?),
			}
			BinaryOperator::Sub => Value::Num(lhs.to_num(l_pos)? - rhs.to_num(r_pos)?),
			BinaryOperator::Mul => Value::Num(lhs.to_num(l_pos)? * rhs.to_num(r_pos)?),
			BinaryOperator::Div => if rhs.to_num(r_pos)? == 0.0 {
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

	fn logic(&mut self, data: LogicData, _pos: SourcePos) -> Result<Value> {
		let left = is_truthy(&data.lhs.accept(self)?);
		Value::Bool(match data.op {
			LogicOperator::And => if left { is_truthy(&data.rhs.accept(self)?) } else { false },
			LogicOperator::Or => if left { true } else { is_truthy(&data.rhs.accept(self)?) }
		}).wrap()
	}

	fn grouping(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<Value> {
		data.accept(self)
	}

	fn variable(&mut self, data: String, pos: SourcePos) -> Result<Value> {
		self.env.get(&data, pos)
	}

	fn read(&mut self, pos: SourcePos) -> Result<Value> {
		let in_res: std::result::Result<String, text_io::Error> = try_read!("{}\r\n");
		match in_res {
			Ok(str) => Value::Str(str).wrap(),
			Err(_) => ErrorList::new("Invalid console input".to_owned(), pos).err(),
		}
	}

	fn readnum(&mut self, pos: SourcePos) -> Result<Value> {
		let in_res: std::result::Result<f64, text_io::Error> = try_read!("{}\r\n");
		match in_res {
			Ok(n) => Value::Num(n).wrap(),
			Err(_) => ErrorList::new("Invalid console input, expected a number".to_owned(), pos).err(),
		}
	}
}

impl StmtVisitor<Message> for Interpreter {

	fn writeline(&mut self, data: Box<Expression>, _pos: SourcePos) -> MessageResult {
		let val = data.accept(self)?;
		println!("{}", val);
		None.wrap()
	}

	fn declaration(&mut self, data: DeclarationData, _pos: SourcePos) -> MessageResult {
		let val = data.expr.accept(self)?;
		self.env.define(data.name, val);
		None.wrap()
	}

	fn assignment(&mut self, data: AssignData, pos: SourcePos) -> MessageResult {
		let val = data.expr.accept(self)?;
		self.env.assign(data.name, val, pos)?;
		None.wrap()
	}

	fn if_stmt(&mut self, data: IfData, _pos: SourcePos) -> MessageResult {
		if is_truthy(&data.cond.accept(self)?) {
			self.execute_block(data.then_block, ValueMap::new())
		} else {
			self.execute_block(data.else_block, ValueMap::new())
		}
	}

	fn loop_stmt(&mut self, block: Block, _pos: SourcePos) -> MessageResult {
		loop {
			match self.execute_block(block.clone(), ValueMap::new())? {
				None | Continue => continue,
				Break => return None.wrap()
			}
		}
	}

	fn break_stmt(&mut self, _pos: SourcePos) -> MessageResult {
		Ok(Break)
	}

	fn continue_stmt(&mut self, _pos: SourcePos) -> Result<Message> {
		Ok(Continue)
	}

}
