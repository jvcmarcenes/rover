
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::utils::{result::*, source_pos::*};

use self::ExprType::*;

use super::{Identifier, statement::Block};

#[derive(Debug, Clone)]
pub enum BinaryOperator {
	Add, Sub, Mul, Div, Rem,
	Equ, Neq, Lst, Lse, Grt, Gre,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
	Not, Neg
}

#[derive(Debug, Clone)]
pub enum LogicOperator {
	And, Or
}

#[derive(Debug, Clone)]
pub enum LiteralData {
	None,
	Str(String),
	Num(f64),
	Bool(bool),
	Template(Vec<Expression>),
	List(Vec<Expression>),
	Object(HashMap<String, Expression>),
}

#[derive(Debug, Clone)]
pub struct BinaryData { pub lhs: Box<Expression>, pub op: BinaryOperator, pub rhs: Box<Expression> }
#[derive(Debug, Clone)]
pub struct UnaryData { pub op: UnaryOperator, pub expr: Box<Expression> }
#[derive(Debug, Clone)]
pub struct LogicData { pub lhs: Box<Expression>, pub op: LogicOperator, pub rhs: Box<Expression> }
#[derive(Debug, Clone)]
pub struct LambdaData { pub self_ref: Option<SourcePos>, pub params: Vec<Identifier>, pub body: Block }
#[derive(Debug, Clone)]
pub struct CallData { pub calee: Box<Expression>, pub args: Vec<Expression> }
#[derive(Debug, Clone)]
pub struct IndexData { pub head: Box<Expression>, pub index: Box<Expression> }
#[derive(Debug, Clone)]
pub struct FieldData { pub head: Box<Expression>, pub field: String }

#[derive(Debug, Clone)]
pub enum ExprType {
	Literal(LiteralData),
	Binary(BinaryData),
	Unary(UnaryData),
	Logic(LogicData),
	Grouping(Box<Expression>),
	Variable(Identifier),
	Lambda(LambdaData),
	Call(CallData),
	Index(IndexData),
	FieldGet(FieldData),
	SelfRef(Rc<RefCell<usize>>),
}

impl ExprType {
	pub fn to_expr(self, pos: SourcePos) -> Expression {
		Expression { typ: self, pos }
	}
}

#[derive(Debug, Clone)]
pub struct Expression {
	pub typ: ExprType,
	pub pos: SourcePos
}

impl Expression {
	pub fn new(typ: ExprType, pos: SourcePos) -> Self {
		Self { typ, pos }
	}
}

impl Expression {
	pub fn accept<T>(self, visitor: &mut dyn ExprVisitor<T>) -> Result<T> {
		match self.typ {
			Literal(value) => visitor.literal(value, self.pos),
			Binary(data) => visitor.binary(data, self.pos),
			Unary(data) => visitor.unary(data, self.pos),
			Logic(data) => visitor.logic(data, self.pos),
			Grouping(data) => visitor.grouping(data, self.pos),
			Variable(name) => visitor.variable(name, self.pos),
			Lambda(data) => visitor.lambda(data, self.pos),
			Call(data) => visitor.call(data, self.pos),
			Index(data) => visitor.index(data, self.pos),
			FieldGet(data) => visitor.field(data, self.pos),
			SelfRef(data) => visitor.self_ref(data, self.pos),
		}
	}
}

pub trait ExprVisitor<T> {
	fn literal(&mut self, data: LiteralData, pos: SourcePos) -> Result<T>;
	fn binary(&mut self, data: BinaryData, pos: SourcePos) -> Result<T>;
	fn unary(&mut self, data: UnaryData, pos: SourcePos) -> Result<T>;
	fn logic(&mut self, data: LogicData, pos: SourcePos) -> Result<T>;
	fn grouping(&mut self, data: Box<Expression>, pos: SourcePos) -> Result<T>;
	fn variable(&mut self, data: Identifier, pos: SourcePos) -> Result<T>;
	fn lambda(&mut self, data: LambdaData, pos: SourcePos) -> Result<T>;
	fn call(&mut self, data: CallData, pos: SourcePos) -> Result<T>;
	fn index(&mut self, data: IndexData, pos: SourcePos) -> Result<T>;
	fn field(&mut self, data: FieldData, pos: SourcePos) -> Result<T>;
	fn self_ref(&mut self, data: Rc<RefCell<usize>>, pos: SourcePos) -> Result<T>;
}