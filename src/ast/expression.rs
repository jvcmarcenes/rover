
use crate::utils::{result::*, source_pos::*};

use self::ExprType::*;

#[derive(Clone)]
pub enum BinaryOperator {
	Add, Sub, Mul, Div, Rem,
	Equ, Neq, Lst, Lse, Grt, Gre,
}

#[derive(Clone)]
pub enum UnaryOperator { Not, Neg }
#[derive(Clone)]
pub enum LogicOperator { And, Or }

#[derive(Clone)]
pub enum LiteralData {
	Str(String),
	Num(f64),
	Bool(bool),
	None,
}

#[derive(Clone)]
pub struct BinaryData { pub lhs: Box<Expression>, pub op: BinaryOperator, pub rhs: Box<Expression> }
#[derive(Clone)]
pub struct UnaryData { pub op: UnaryOperator, pub expr: Box<Expression> }
#[derive(Clone)]
pub struct LogicData { pub lhs: Box<Expression>, pub op: LogicOperator, pub rhs: Box<Expression> }

#[derive(Clone)]
pub enum ExprType {
	Literal(LiteralData),
	Binary(BinaryData),
	Unary(UnaryData),
	Logic(LogicData),
	Grouping(Box<Expression>),
	Variable(String),
	Read, ReadNum
}

impl ExprType {
	pub fn to_expr(self, pos: SourcePos) -> Expression {
		Expression { typ: self, pos }
	}
}

#[derive(Clone)]
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
			Read => visitor.read(self.pos),
			ReadNum => visitor.readnum(self.pos),
		}
	}
}

pub trait ExprVisitor<T> {
	fn literal(&mut self, data: LiteralData, pos: SourcePos) -> Result<T>;
	fn binary(&mut self, data: BinaryData, pos: SourcePos) -> Result<T>;
	fn unary(&mut self, data: UnaryData, pos: SourcePos) -> Result<T>;
	fn logic(&mut self, data: LogicData, pos: SourcePos) -> Result<T>;
	fn grouping(&mut self, data: Box<Expression>, pos: SourcePos) -> Result<T>;
	fn variable(&mut self, data: String, pos: SourcePos) -> Result<T>;
	fn read(&mut self, pos: SourcePos) -> Result<T>;
	fn readnum(&mut self, pos: SourcePos) -> Result<T>;
}