
pub mod value;
pub mod environment;
pub mod globals;

use std::{cell::RefCell, rc::Rc};

use crate::{ast::{expression::*, statement::*}, utils::{result::{Result, ErrorList}, source_pos::SourcePos, wrap::Wrap}};

use self::{Message::*, environment::{Environment, ValueMap}, value::{Value, function::Function}};

fn get_index(mut n: f64, len: usize, pos: SourcePos) -> Result<usize> {
	if n < 0.0 { n += len as f64; }
	if n < 0.0 || n >= len as f64 { 
		ErrorList::run("Index out of bounds".to_owned(), pos).err()
	} else {
		(n as usize).wrap()
	}
}

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
			env: Environment::new(),
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
			LiteralData::List(exprs) => {
				let mut values = Vec::new();
				for expr in exprs { values.push(expr.accept(self)?) }
				Value::List(values)
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
					return ErrorList::run("Cannot divide by zero".to_owned(), r_pos).err()
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

	fn lambda(&mut self, data: LambdaData, _pos: SourcePos) -> Result<Value> {
		let func = Function::new(self.env.clone(), data.params, data.body);
		Value::Callable(Rc::new(RefCell::new(func))).wrap()
	}

	fn call(&mut self, data: CallData, pos: SourcePos) -> Result<Value> {
		let calee_pos = data.calee.pos;
		let bound = match data.calee.typ {
			ExprType::Variable(_) | ExprType::Index(_) => true,
			_ => false,
		};
		let calee = data.calee.accept(self)?;
		let mut args = Vec::new();
		for arg in data.args {
			let arg_pos = arg.pos;
			args.push((arg.accept(self)?, arg_pos));
		}
		// We need a pointer to the callable because we need multiple mutable borrows of a shared reference
		// We need a shared reference (Rc<RefCell<dyn Callable>>) to be able to handle closures and interior mutability
		// We need multiple mutable borrows to handle recursive function calls
		// This should not cause any issues since the function won't drop itself or it's environment!
		// HOWEVER, we can only do this if the value is bound in the environment, if the calee is a lambda this would cause a segfault
		if bound {
			unsafe {
				let function = calee.to_callable(calee_pos)?.as_ptr();
				function.as_ref().unwrap().check_arity(args.len(), pos)?;
				let ret = function.as_mut().unwrap().call(calee_pos, self, args);
				ret
			}
		} else {
			let function = calee.to_callable(calee_pos)?;
			function.borrow().check_arity(args.len(), pos)?;
			let ret = function.borrow_mut().call(calee_pos, self, args);
			ret
		}
	}

	fn index(&mut self, data: IndexData, _pos: SourcePos) -> Result<Value> {
		let (head_pos, index_pos) = (data.head.pos, data.index.pos);
		let list = match data.head.accept(self)? {
			Value::List(list) => list,
			Value::Str(str) => str.chars().map(|c| Value::Str(c.to_string())).collect(),
			val => return ErrorList::run(format!("Cannot index {}", val.get_type()), head_pos).err()
		};
		let index = data.index.accept(self)?.to_num(index_pos)?;
		let index = get_index(index, list.len(), index_pos)?;
		Ok(list[index].clone())
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
		let mut head = data.head;
		let mut val = data.expr.accept(self)?;
		loop {
			match head.typ {
				ExprType::Variable(name) => {
					self.env.assign(&name, val, pos)?;
					return Message::None.wrap();
				},
				ExprType::Index(IndexData { head: ihead, index }) => {
					let h_pos = ihead.pos;
					head = ihead.clone();
					let mut list = ihead.accept(self)?.to_list(h_pos)?;
					let i_pos = index.pos;
					let index = index.accept(self)?.to_num(i_pos)?;
					let index = get_index(index, list.len() + 1, i_pos)?;
					if index < list.len() { list.remove(index); }
					list.insert(index, val);
					val = Value::List(list);
				}
				_ => return ErrorList::run("Invalid assignment target".to_owned(), head.pos).err()
			}
		}
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
