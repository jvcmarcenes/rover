
use std::fmt::Display;

use crate::source_pos::SourcePos;

use self::{LiteralValue::*, ExprType::*};

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
	Str(String),
	Num(f64),
	Bool(bool),
}

#[derive(Clone, Debug)]
pub enum BinaryOperator {
	Add, Sub, Mul, Div, Mod,
	Equ, Neq, Lst, Lse, Grt, Gre,
	And, Or
}

#[derive(Clone, Debug)]
pub enum UnaryOperator {
	Not, Neg
}

#[derive(Clone, Debug)]
pub enum ExprType {
	Literal { value: LiteralValue },
	Binary { lhs: Box<Expression>, op: BinaryOperator, rhs: Box<Expression> },
	Unary { op: UnaryOperator, expr: Box<Expression> },
	Grouping { expr: Box<Expression> },
}

impl ExprType {
	pub fn to_expr(self, pos: SourcePos) -> Expression {
		Expression { typ: self, pos }
	}
}

impl Display for ExprType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Literal { value } => match value {
				Str(s) => write!(f, "{}", s),
				Num(n) => write!(f, "{}", n),
				Bool(b) => write!(f, "{}", b),
			},
			Binary { lhs, op, rhs } => write!(f, "{}, {:?}, {}", lhs, op, rhs),
			Unary { op, expr } => write!(f, "{:?}{}", op, expr),
			Grouping { expr } => write!(f, "({})", expr),
		}
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

impl Display for Expression {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {}", self.typ, self.pos)
	}
}
