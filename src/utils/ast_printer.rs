
use std::fmt::Write;

use crate::ast::expression::{BinaryData, BinaryOperator, ExprVisitor, Expression, LiteralData, UnaryData, UnaryOperator};

use super::{result::Result, wrap::Wrap};

pub struct AstPrinter;

impl AstPrinter {
	pub fn print(&mut self, expr: Box<Expression>) -> Result<String> {
		expr.accept(self)
	}
}

impl ExprVisitor<String> for AstPrinter {

	fn literal(&mut self, data: LiteralData) -> Result<String> {
		match data {
			LiteralData::Str(s) => format!("{}", s).wrap(),
			LiteralData::Num(n) => format!("{}", n).wrap(),
			LiteralData::Bool(b) => format!("{}", b).wrap(),
		}
	}

	fn binary(&mut self, data: BinaryData) -> Result<String> {
		let mut s = String::new();
		let op = match data.op {
			BinaryOperator::Add => "+",
			BinaryOperator::Sub => "-",
			BinaryOperator::Mul => "*",
			BinaryOperator::Div => "/",
			BinaryOperator::Rem => "mod",
			BinaryOperator::Equ => "==",
			BinaryOperator::Neq => "!=",
			BinaryOperator::Lst => "<",
			BinaryOperator::Lse => "<=",
			BinaryOperator::Grt => ">",
			BinaryOperator::Gre => ">=",
			BinaryOperator::And => "and",
			BinaryOperator::Or => "or",
		};
		write!(s, "({} {} {})", op, self.print(data.lhs)?, self.print(data.rhs)?).unwrap();
		s.wrap()
	}

	fn unary(&mut self, data: UnaryData) -> Result<String> {
		let mut s = String::new();
		let op = match data.op {
			UnaryOperator::Neg => "!",
			UnaryOperator::Not => "-",
		};
		write!(s, "({} {})", op, self.print(data.expr)?).unwrap();
		s.wrap()
	}

	fn grouping(&mut self, data: Box<Expression>) -> Result<String> {
		format!("({})", self.print(data)?).wrap()
	}
}
