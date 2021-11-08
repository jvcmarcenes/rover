
use crate::utils::{result::*, source_pos::*};

use self::ExprType::*;


#[derive(Clone, Debug)]
pub enum BinaryOperator {
	Add, Sub, Mul, Div, Rem,
	Equ, Neq, Lst, Lse, Grt, Gre,
	And, Or
}

#[derive(Clone, Debug)]
pub enum UnaryOperator {
	Not, Neg
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralData {
	Str(String),
	Num(f64),
	Bool(bool),
}

#[derive(Clone, Debug)]
pub struct BinaryData { pub lhs: Box<Expression>, pub op: BinaryOperator, pub rhs: Box<Expression> }

#[derive(Clone, Debug)]
pub struct UnaryData { pub op: UnaryOperator, pub expr: Box<Expression> }

#[derive(Clone, Debug)]
pub enum ExprType {
	Literal(LiteralData),
	Binary(BinaryData),
	Unary(UnaryData),
	Grouping(Box<Expression>),
}

impl ExprType {
	pub fn to_expr(self, pos: SourcePos) -> Expression {
		Expression { typ: self, pos }
	}
}

#[derive(Clone, Debug)]
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
			Literal(value) => visitor.literal(value),
			Binary(data) => visitor.binary(data),
			Unary(data) => visitor.unary(data),
			Grouping(data) => visitor.grouping(data),
		}
	}
}

pub trait ExprVisitor<T> {
	fn literal(&mut self, data: LiteralData) -> Result<T>;
	fn binary(&mut self, data: BinaryData) -> Result<T>;
	fn unary(&mut self, data: UnaryData) -> Result<T>;
	fn grouping(&mut self, data: Box<Expression>) -> Result<T>;
}