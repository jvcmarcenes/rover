
pub mod value;
pub mod environment;
pub mod globals;

use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use crate::{ast::{Identifier, expression::*, statement::*}, interpreter::value::macros::{pass_msg, unwrap_msg}, utils::{result::{Result, ErrorList}, source_pos::SourcePos, wrap::Wrap}};

use self::{environment::{Environment, ValueMap}, value::{Value, function::{Function, SELF}}};

fn get_index(mut n: f64, len: usize, pos: SourcePos) -> Result<usize> {
	if n < 0.0 { n += len as f64; }
	if n < 0.0 || n >= len as f64 { 
		ErrorList::run("Index out of bounds".to_owned(), pos).err()
	} else {
		(n as usize).wrap()
	}
}

#[derive(Clone, Debug)]
pub enum Message {
	None,
	Break,
	Continue,
	Return(Value),
	Eval(Value),
}

pub struct Interpreter {
	env: Environment,
	pub location: PathBuf,
}

impl Interpreter {

	pub fn new(globals: ValueMap, location: PathBuf) -> Self {
		Self {
			env: Environment::new(globals),
			location,
		}
	}

	pub fn interpret(&mut self, statements: &Block) -> Result<()> {
		for stmt in statements.clone() { stmt.accept(self)?; }
		Ok(())
	}

	fn execute_block(&mut self, block: Block) -> Result<Message> {
		self.env.push_new();

		let mut last_eval = Message::None;

		for stmt in block {
			match stmt.accept(self)? {
				Message::None => continue,
				Message::Eval(val) => last_eval = Message::Eval(unwrap_msg!(val)),
				msg => {
					self.env.pop();
					return msg.wrap();
				}
			}
		}
		self.env.pop();
		last_eval.wrap()
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
				for expr in exprs { values.push((expr.pos, pass_msg!(expr.accept(self)?))); }
				let mut strs = Vec::new();
				for (pos, val) in values { strs.push(val.to_string(self, pos)?); }
				Value::Str(strs.join(""))
			},
			LiteralData::List(exprs) => {
				let mut values = Vec::new();
				for expr in exprs { values.push(expr.accept(self)?) }
				Value::List(values)
			},
			LiteralData::Object(map) => {
				let mut value_map = HashMap::new();
				for (key, expr) in map {
					value_map.insert(key, expr.accept(self)?.wrap());
				}
				Value::Object(value_map)
			}
			LiteralData::Error(expr) => Value::Error(pass_msg!(expr.accept(self)?).wrap()),
		};
		value.wrap()
	}

	fn binary(&mut self, data: BinaryData, pos: SourcePos) -> Result<Value> {
		let (l_pos, r_pos) = (data.lhs.pos, data.rhs.pos);
		let lhs = pass_msg!(data.lhs.accept(self)?);
		let rhs = pass_msg!(data.rhs.accept(self)?);
		let value = match data.op {
			BinaryOperator::Add => lhs.add(&rhs, r_pos, self, pos)?,
			BinaryOperator::Sub => lhs.sub(&rhs, r_pos, self, pos)?,
			BinaryOperator::Mul => lhs.mul(&rhs, r_pos, self, pos)?,
			BinaryOperator::Div => lhs.div(&rhs, r_pos, self, pos)?,
			BinaryOperator::Rem => Value::Num(lhs.to_num(l_pos)? % rhs.to_num(r_pos)?),
			BinaryOperator::Lst => Value::Bool(lhs.to_num(l_pos)? < rhs.to_num(r_pos)?),
			BinaryOperator::Lse => Value::Bool(lhs.to_num(l_pos)? <= rhs.to_num(r_pos)?),
			BinaryOperator::Grt => Value::Bool(lhs.to_num(l_pos)? > rhs.to_num(r_pos)?),
			BinaryOperator::Gre => Value::Bool(lhs.to_num(l_pos)? >= rhs.to_num(r_pos)?),
			BinaryOperator::Equ => Value::Bool(lhs.equals(&rhs, r_pos, self, pos)?),
			BinaryOperator::Neq => Value::Bool(!lhs.equals(&rhs, r_pos, self, pos)?),
		};
		value.wrap()
	}

	fn unary(&mut self, data: UnaryData, _pos: SourcePos) -> Result<Value> {
		let pos = data.expr.pos;
		let val = pass_msg!(data.expr.accept(self)?);
		match data.op {
			UnaryOperator::Neg => Value::Num(-val.to_num(pos)?).wrap(),
			UnaryOperator::Not => Value::Bool(!val.is_truthy()).wrap(),
		}
	}

	fn logic(&mut self, data: LogicData, _pos: SourcePos) -> Result<Value> {
		let left = pass_msg!(data.lhs.accept(self)?).is_truthy();
		Value::Bool(match data.op {
			LogicOperator::And => if left { pass_msg!(data.rhs.accept(self)?).is_truthy() } else { false },
			LogicOperator::Or => if left { true } else { pass_msg!(data.rhs.accept(self)?).is_truthy() }
		}).wrap()
	}

	fn grouping(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<Value> {
		data.accept(self)
	}

	fn variable(&mut self, data: Identifier, _pos: SourcePos) -> Result<Value> {
		self.env.get(data.get_id()).wrap()
	}

	fn lambda(&mut self, data: LambdaData, _pos: SourcePos) -> Result<Value> {
		let func = Function::new(self.env.clone(), data.params, data.body);
		Value::Callable(Rc::new(RefCell::new(func))).wrap()
	}

	fn call(&mut self, data: CallData, pos: SourcePos) -> Result<Value> {
		let calee_pos = data.calee.pos;
		let bound = match data.calee.typ {
			ExprType::Variable(_) | ExprType::Index(_) | ExprType::FieldGet(_) => true,
			_ => false,
		};
		let calee = pass_msg!(data.calee.accept(self)?);
		let mut args = Vec::new();
		for arg in data.args {
			let arg_pos = arg.pos;
			args.push((pass_msg!(arg.accept(self)?), arg_pos));
		}
		// We need a pointer to the callable because we need multiple mutable borrows of a shared reference
		// We need a shared reference (Rc<RefCell<dyn Callable>>) to be able to handle closures and interior mutability
		// We need multiple mutable borrows to handle recursive function calls
		// This should not cause any issues since the function won't drop itself or it's environment!
			// Actually... a function can reference the name it is bound too, and therefore can mutate it, causing it to be dropped
			// A solution that could get rid of the 'unsafe' code (and solve this) could be:
			// instead of mutating the function reference, we clone it, mutate it's local environemnt, and then assign it to itself after the call is done
		// HOWEVER, we can only do this if the value is bound in the environment, if the calee is a lambda this would cause a segfault
		// Additionaly, if a function isn't bound, it can't call itself recursively, so we don't need multiple mutable borrows either way
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
		let list = match pass_msg!(data.head.accept(self)?) {
			Value::List(list) => list,
			Value::Str(str) => str.chars().map(|c| Value::Str(c.to_string())).collect(),
			val => return ErrorList::run(format!("Cannot index {}", val.get_type()), head_pos).err()
		};
		let index = pass_msg!(data.index.accept(self)?).to_num(index_pos)?;
		let index = get_index(index, list.len(), index_pos)?;
		list[index].clone().wrap()
	}

	fn field(&mut self, data: FieldData, pos: SourcePos) -> Result<Value> {
		let head = pass_msg!(data.head.accept(self)?);
		let field = head.get_field(&data.field, pos)?;
		if let Value::Callable(callable) = field.borrow().clone() {
			callable.borrow_mut().bind(head);
		};
		field.clone().borrow().clone().wrap()
	}

	fn self_ref(&mut self, _pos: SourcePos) -> Result<Value> {
		self.env.get(SELF).wrap()
	}

	fn do_expr(&mut self, block: Block, _pos: SourcePos) -> Result<Value> {
		match self.execute_block(block)? {
			Message::None => Value::None,
			Message::Eval(val) => pass_msg!(val),
			// Message::Error(_) => todo!(),
			msg => Value::Messenger(Box::new(msg))
			// should break and continue bubble from do expressions?
			// Message::Break => todo()!
			// Message::Continue => todo()!
		}.wrap()
	}

}

impl StmtVisitor<Message> for Interpreter {

	fn expr(&mut self, expr: Box<Expression>, _pos: SourcePos) -> Result<Message> {
		let val = unwrap_msg!(expr.accept(self)?);
		Message::Eval(val).wrap()
	}

	fn declaration(&mut self, data: DeclarationData, _pos: SourcePos) -> Result<Message> {
		// this crashes with objects that try to 'statically' access the variable they're being declared to
		// the resolver allows it (and it should), but here the name is only defined after the r-value is evaluated
		let val = unwrap_msg!(data.expr.accept(self)?);
		self.env.define(data.name.get_id(), val);
		Message::None.wrap()
	}

	fn assignment(&mut self, data: AssignData, _pos: SourcePos) -> Result<Message> {
		let mut head = data.head;
		let mut val = unwrap_msg!(data.expr.accept(self)?);
		loop {
			match head.typ {
				ExprType::SelfRef => {
					self.env.assign(SELF, val);
					return Message::None.wrap();
				},
				ExprType::Variable(name) => {
					self.env.assign(name.get_id(), val);
					return Message::None.wrap();
				},
				ExprType::Index(IndexData { head: ihead, index }) => {
					let h_pos = ihead.pos;
					head = ihead.clone();
					let mut list = ihead.accept(self)?.to_list(h_pos)?;
					let i_pos = index.pos;
					let index = unwrap_msg!(index.accept(self)?).to_num(i_pos)?;
					let index = get_index(index, list.len() + 1, i_pos)?;
					if index < list.len() { list.remove(index); }
					list.insert(index, val);
					val = Value::List(list);
				},
				ExprType::FieldGet(FieldData { head: fhead, field }) => {
					let h_pos = fhead.pos;
					head = fhead.clone();
					let map = fhead.accept(self)?.to_obj(h_pos)?;
					if let Some(cur) = map.get(&field) {
						*cur.borrow_mut() = val;
					} else {
						return ErrorList::run(format!("Property {} is undefined for object", field), h_pos).err();
					}
					val = Value::Object(map);
				},
				_ => return ErrorList::run("Invalid assignment target".to_owned(), head.pos).err()
			}
		}
	}

	fn if_stmt(&mut self, data: IfData, _pos: SourcePos) -> Result<Message> {
		if unwrap_msg!(data.cond.accept(self)?).is_truthy() {
			self.execute_block(data.then_block)
		} else {
			self.execute_block(data.else_block)
		}
	}

	fn loop_stmt(&mut self, block: Block, _pos: SourcePos) -> Result<Message> {
		loop {
			match self.execute_block(block.clone())? {
				Message::None | Message::Continue | Message::Eval(_) => continue,
				Message::Break => return Message::None.wrap(),
				msg => return msg.wrap(),
			}
		}
	}

	fn break_stmt(&mut self, _pos: SourcePos) -> Result<Message> {
		Message::Break.wrap()
	}

	fn continue_stmt(&mut self, _pos: SourcePos) -> Result<Message> {
		Message::Continue.wrap()
	}

	fn return_stmt(&mut self, expr: Box<Expression>, _pos: SourcePos) -> Result<Message> {
		let val = unwrap_msg!(expr.accept(self)?);
		Message::Return(val).wrap()
	}

	fn scoped_stmt(&mut self, block: Block, _pos: SourcePos) -> Result<Message> {
		self.execute_block(block)
	}

}
