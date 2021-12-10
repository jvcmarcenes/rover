
pub mod value;
pub mod globals;

use std::{collections::{HashMap, HashSet}, path::PathBuf};

use crate::{environment::Environment, ast::{identifier::Identifier, expression::*, statement::*}, interpreter::value::{ValueType, macros::{castf, pass_msg, unwrap_msg}, messenger::Messenger, primitives::{bool::Bool, error::Error, none::ValNone, number::Number, object::Object, string::Str, list::List}}, utils::{result::{Result, ErrorList}, source_pos::SourcePos, wrap::Wrap}};

use self::{value::{Value, primitives::{callable::{ValCallable, function::{Function, SELF}}, attribute::Attribute}}, globals::init_globals};

pub fn get_index(mut n: f64, len: usize, pos: SourcePos) -> Result<usize> {
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
	Return(Box<dyn Value>),
	Eval(Box<dyn Value>),
}

pub struct Interpreter {
	env: Environment<Box<dyn Value>>,
	pub root_path: PathBuf,
}

impl Interpreter {
	
	pub fn new(root_path: PathBuf) -> Self {
		Self {
			env: Environment::new(init_globals()),
			root_path,
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

impl ExprVisitor<Box<dyn Value>> for Interpreter {
	
	fn literal(&mut self, data: LiteralData, _pos: SourcePos) -> Result<Box<dyn Value>> {
		match data {
			LiteralData::None => ValNone.wrap(),
			LiteralData::Str(s) => Str::new(s).wrap(),
			LiteralData::Num(n) => Number::new(n).wrap(),
			LiteralData::Bool(b) => Bool::new(b).wrap(),
			LiteralData::Template(exprs) => {
				let mut values = Vec::new();
				for expr in exprs { values.push((expr.pos, pass_msg!(expr.accept(self)?))); }
				let mut strs = Vec::new();
				for (pos, val) in values { strs.push(val.to_string(self, pos)?); }
				Str::new(strs.join("")).wrap()
			},
			LiteralData::List(exprs) => {
				let mut values = Vec::new();
				for expr in exprs { values.push(expr.accept(self)?) }
				List::new(values).wrap()
			},
			LiteralData::Object(map, attrs) => {
				let mut value_map = HashMap::new();
				for (key, expr) in map {
					value_map.insert(key, expr.accept(self)?.wrap());
				}
				
				let attributes = attrs.iter().map(|i| i.get_id()).collect::<HashSet<_>>();
				Object::new(value_map, attributes).wrap()
			}
			LiteralData::Error(expr) => Error::new(pass_msg!(expr.accept(self)?)).wrap(),
		}
	}
	
	fn binary(&mut self, data: BinaryData, pos: SourcePos) -> Result<Box<dyn Value>> {
		let (l_pos, r_pos) = (data.lhs.pos, data.rhs.pos);
		let lhs = pass_msg!(data.lhs.accept(self)?);
		let rhs = pass_msg!(data.rhs.accept(self)?);
		match data.op {
			BinaryOperator::Add => lhs.add(rhs, r_pos, self, pos)?.wrap(),
			BinaryOperator::Sub => lhs.sub(rhs, r_pos, self, pos)?.wrap(),
			BinaryOperator::Mul => lhs.mul(rhs, r_pos, self, pos)?.wrap(),
			BinaryOperator::Div => lhs.div(rhs, r_pos, self, pos)?.wrap(),
			BinaryOperator::Rem => Number::new(lhs.to_num(l_pos)? % rhs.to_num(r_pos)?).wrap(),
			BinaryOperator::Lst => Bool::new(lhs.to_num(l_pos)? < rhs.to_num(r_pos)?).wrap(),
			BinaryOperator::Lse => Bool::new(lhs.to_num(l_pos)? <= rhs.to_num(r_pos)?).wrap(),
			BinaryOperator::Grt => Bool::new(lhs.to_num(l_pos)? > rhs.to_num(r_pos)?).wrap(),
			BinaryOperator::Gre => Bool::new(lhs.to_num(l_pos)? >= rhs.to_num(r_pos)?).wrap(),
			BinaryOperator::Equ => Bool::new(lhs.equals(rhs, r_pos, self, pos)?).wrap(),
			BinaryOperator::Neq => Bool::new(!lhs.equals(rhs, r_pos, self, pos)?).wrap(),
			BinaryOperator::Typ => Bool::new(lhs.has_attr(rhs.to_attr(r_pos)?.get_id(), self)).wrap(),
		}
	}
	
	fn unary(&mut self, data: UnaryData, _pos: SourcePos) -> Result<Box<dyn Value>> {
		let pos = data.expr.pos;
		let val = pass_msg!(data.expr.accept(self)?);
		match data.op {
			UnaryOperator::Pos => Number::new(val.to_num(pos)?).wrap(),
			UnaryOperator::Neg => Number::new(-val.to_num(pos)?).wrap(),
			UnaryOperator::Not => Bool::new(!val.is_truthy()).wrap(),
		}
	}
	
	fn logic(&mut self, data: LogicData, _pos: SourcePos) -> Result<Box<dyn Value>> {
		let left = pass_msg!(data.lhs.accept(self)?).is_truthy();
		Bool::new(match data.op {
			LogicOperator::And => if left { pass_msg!(data.rhs.accept(self)?).is_truthy() } else { false },
			LogicOperator::Or => if left { true } else { pass_msg!(data.rhs.accept(self)?).is_truthy() }
		}).wrap()
	}
	
	fn grouping(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<Box<dyn Value>> {
		data.accept(self)
	}
	
	fn variable(&mut self, data: Identifier, _pos: SourcePos) -> Result<Box<dyn Value>> {
		self.env.get(data.get_id()).wrap()
	}
	
	fn lambda(&mut self, data: LambdaData, _pos: SourcePos) -> Result<Box<dyn Value>> {
		let func = Function::new(self.env.clone(), data.params, data.body);
		ValCallable::new(func.wrap()).wrap()
	}
	
	fn call(&mut self, data: CallData, pos: SourcePos) -> Result<Box<dyn Value>> {
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
	
	fn index(&mut self, data: IndexData, _pos: SourcePos) -> Result<Box<dyn Value>> {
		let (head_pos, index_pos) = (data.head.pos, data.index.pos);
		let head_val = pass_msg!(data.head.accept(self)?);
		let list = match head_val.get_type() {
			ValueType::Vector => head_val.to_list(head_pos)?.borrow().clone(),
			ValueType::Str => head_val.to_str(head_pos)?.chars().map(|c| Str::new(c.to_string())).collect(),
			typ => return ErrorList::run(format!("Cannot index {}", typ), head_pos).err()
		};
		let index = pass_msg!(data.index.accept(self)?).to_num(index_pos)?;
		let index = get_index(index, list.len(), index_pos)?;
		list[index].clone().wrap()
	}
	
	fn field(&mut self, data: FieldData, pos: SourcePos) -> Result<Box<dyn Value>> {
		let head = pass_msg!(data.head.accept(self)?);
		let field = head.get_field(&data.field, self, pos)?;
		if field.borrow().get_type() == ValueType::Callable && head.get_type() != ValueType::Attribute {
			let method = field.borrow().to_callable(pos)?;
			let mut bound_method = method.borrow().cloned();
			bound_method.bind(head);
			return ValCallable::new(bound_method.wrap()).wrap()
		};
		field.clone().borrow().clone().wrap()
	}
	
	fn self_ref(&mut self, pos: SourcePos) -> Result<Box<dyn Value>> {
		if self.env.has(SELF) {
			self.env.get(SELF).wrap()
		} else {
			ErrorList::run("Unbound self".to_owned(), pos).err()
		}
	}
	
	fn do_expr(&mut self, block: Block, _pos: SourcePos) -> Result<Box<dyn Value>> {
		match self.execute_block(block)? {
			Message::None => ValNone::new(),
			Message::Eval(val) => pass_msg!(val),
			msg => Messenger::new(msg),
		}.wrap()
	}
	
	fn bind_expr(&mut self, data: BindData, _pos: SourcePos) -> Result<Box<dyn Value>> {
		let head = data.expr.accept(self)?;
		let method_pos = data.method.pos;
		let method = data.method.accept(self)?.to_callable(method_pos)?;
		let mut bound_method = method.borrow().cloned();
		bound_method.bind(head);
		ValCallable::new(bound_method.wrap()).wrap()
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
		// self.env.define(data.name.get_id(), ValNone.wrap()) // <- this could be a solution, assign none to the symbol, and after evaluating the r-value we re-assign it
		let val = unwrap_msg!(data.expr.accept(self)?);
		self.env.define(data.name.get_id(), val);
		Message::None.wrap()
	}
	
	fn attr_declaration(&mut self, data: AttrDeclarationData, _pos: SourcePos) -> Result<Message> {
		let mut methods = HashMap::new();
		for method in data.methods {
			let func = Function::new(self.env.clone(), method.params, method.body);
			methods.insert(method.name, ValCallable::new(func.wrap()).wrap());
		}
		
		let mut fields = HashMap::new();
		for (key, expr) in data.fields {
			fields.insert(key, expr.accept(self)?.wrap());
		}
		
		let attrs = data.attributes.iter().map(|i| i.get_id()).collect();
		
		self.env.define(data.name.get_id(), Attribute::new(data.name, methods, fields, attrs));
		Message::None.wrap()
	}
	
	fn assignment(&mut self, data: AssignData, _pos: SourcePos) -> Result<Message> {
		let val = unwrap_msg!(data.expr.accept(self)?);
		loop {
			match data.head.typ {
				ExprType::Variable(name) => {
					self.env.assign(name.get_id(), val);
					return Message::None.wrap();
				},
				ExprType::Index(IndexData { head: ihead, index }) => {
					let h_pos = ihead.pos;
					let head = ihead.accept(self)?;
					let list = if let ValueType::Vector = head.get_type() {
						castf!(vec head.clone())
					} else {
						return ErrorList::run("Invalid assignment target".to_owned(), h_pos).err()
					};
					let i_pos = index.pos;
					let index = unwrap_msg!(index.accept(self)?).to_num(i_pos)?;
					let index = get_index(index, list.borrow().len(), i_pos)?;
					list.borrow_mut().remove(index);
					list.borrow_mut().insert(index, val);
					return Message::None.wrap();
				},
				ExprType::FieldGet(FieldData { head: fhead, field }) => {
					let h_pos = fhead.pos;
					let map = fhead.accept(self)?.to_obj(h_pos)?;
					if let Some(cur) = map.get(&field) {
						*cur.borrow_mut() = val;
					} else {
						return ErrorList::run(format!("Property {} is undefined for object", field), h_pos).err();
					}
					return Message::None.wrap();
				},
				_ => return ErrorList::run("Invalid assignment target".to_owned(), data.head.pos).err()
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
