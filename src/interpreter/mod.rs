
pub mod value;
pub mod environment;
pub mod globals;

use std::{cell::RefCell, rc::Rc};

use crate::{ast::{expression::*, statement::*}, utils::{result::{Result, ErrorList}, source_pos::SourcePos, wrap::Wrap}};

use self::{Message::*, environment::{Environment, ValueMap}, globals::globals, value::{Value, function::Function}};

pub enum Message {
	None,
	Break,
	Continue,
	Return(Value)
}

type MessageResult = Result<Message>;

pub struct Interpreter {
	env: Environment,
}

impl Interpreter {

	pub fn new() -> Self {
		Self {
			env: globals()
		}
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
			LiteralData::None => Value::None,
			LiteralData::Str(s) => Value::Str(s),
			LiteralData::Num(n) => Value::Num(n),
			LiteralData::Bool(b) => Value::Bool(b),
			LiteralData::Template(exprs) => {
				let mut values = Vec::new();
				for expr in exprs { values.push(expr.accept(self)?) }
				Value::Str(values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(""))
			}
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
			BinaryOperator::Div => {
				let rhs = rhs.to_num(r_pos)?;
				if rhs == 0.0 {
					return ErrorList::new("Cannot divide by zero".to_owned(), r_pos).err()
				} else {
					Value::Num(lhs.to_num(l_pos)? / rhs)
				}
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
			UnaryOperator::Not => Value::Bool(!val.is_truthy()).wrap(),
		}
	}

	fn logic(&mut self, data: LogicData, _pos: SourcePos) -> Result<Value> {
		let left = data.lhs.accept(self)?.is_truthy();
		Value::Bool(match data.op {
			LogicOperator::And => if left { data.rhs.accept(self)?.is_truthy() } else { false },
			LogicOperator::Or => if left { true } else { data.rhs.accept(self)?.is_truthy() }
		}).wrap()
	}

	fn grouping(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<Value> {
		data.accept(self)
	}

	fn variable(&mut self, data: String, pos: SourcePos) -> Result<Value> {
		self.env.get(&data, pos)
	}

	fn call(&mut self, data: CallData, pos: SourcePos) -> Result<Value> {
		let calee_pos = data.calee.pos;
		let calee = data.calee.accept(self)?;
		let mut args = Vec::new();
		for arg in data.args {
			args.push(arg.accept(self)?);
		}
		// We need a pointer to the callable because we need multiple mutable borrows of a shared reference
		// We need a shared reference (Rc<RefCell<dyn Callable>>) to be able to handle closures and interior mutability
		// We need multiple mutable borrows to handle recursive function calls
		// This should not cause any issues since the function won't drop itself or it's environment!
		unsafe {
			let function = calee.to_callable(calee_pos)?.as_ptr();
			let arity = function.as_ref().unwrap().arity();
			if arity != args.len() as u8 {
				return ErrorList::new(format!("Expected {} arguments, but got {}", arity, args.len()), pos).err();
			}
			let ret = function.as_mut().unwrap().call(calee_pos, self, args);
			ret
		}
	}

	fn lambda(&mut self, data: LambdaData, _pos: SourcePos) -> Result<Value> {
		let func = Function::new(self.env.clone(), data.params, data.body);
		Value::Callable(Rc::new(RefCell::new(func))).wrap()
	}

}

impl StmtVisitor<Message> for Interpreter {

	fn expr(&mut self, expr: Box<Expression>, _pos: SourcePos) -> Result<Message> {
		expr.accept(self)?;
		None.wrap()
	}

	fn declaration(&mut self, data: DeclarationData, pos: SourcePos) -> MessageResult {
		let val = data.expr.accept(self)?;
		self.env.define(&data.name, val, pos)?;
		None.wrap()
	}

	fn assignment(&mut self, data: AssignData, pos: SourcePos) -> MessageResult {
		let val = data.expr.accept(self)?;
		self.env.assign(&data.name, val, pos)?;
		None.wrap()
	}

	fn if_stmt(&mut self, data: IfData, _pos: SourcePos) -> MessageResult {
		if data.cond.accept(self)?.is_truthy() {
			self.execute_block(data.then_block, ValueMap::new())
		} else {
			self.execute_block(data.else_block, ValueMap::new())
		}
	}

	fn loop_stmt(&mut self, block: Block, _pos: SourcePos) -> MessageResult {
		loop {
			match self.execute_block(block.clone(), ValueMap::new())? {
				None | Continue => continue,
				Break => return None.wrap(),
				msg => return msg.wrap(),
			}
		}
	}

	fn break_stmt(&mut self, _pos: SourcePos) -> MessageResult {
		Ok(Break)
	}

	fn continue_stmt(&mut self, _pos: SourcePos) -> Result<Message> {
		Ok(Continue)
	}

	fn return_stmt(&mut self, expr: Box<Expression>, _pos: SourcePos) -> Result<Message> {
		let val = expr.accept(self)?;
		Return(val).wrap()
	}

}
