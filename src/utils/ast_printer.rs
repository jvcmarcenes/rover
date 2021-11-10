
use std::fmt::Write;

use crate::ast::{expression::*, statement::*};

use super::{result::Result, source_pos::SourcePos, wrap::Wrap};

pub struct AstPrinter;

impl ExprVisitor<String> for AstPrinter {

	fn literal(&mut self, data: LiteralData, _pos: SourcePos) -> Result<String> {
		match data {
			LiteralData::Str(s) => format!("{}", s).wrap(),
			LiteralData::Num(n) => format!("{}", n).wrap(),
			LiteralData::Bool(b) => format!("{}", b).wrap(),
			LiteralData::None => format!("").wrap(),
		}
	}

	fn binary(&mut self, data: BinaryData, _pos: SourcePos) -> Result<String> {
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
		write!(s, "({} {} {})", op, data.lhs.accept(self)?, data.rhs.accept(self)?).unwrap();
		s.wrap()
	}

	fn unary(&mut self, data: UnaryData, _pos: SourcePos) -> Result<String> {
		let mut s = String::new();
		let op = match data.op {
			UnaryOperator::Neg => "!",
			UnaryOperator::Not => "-",
		};
		write!(s, "({} {})", op, data.expr.accept(self)?).unwrap();
		s.wrap()
	}

	fn grouping(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<String> {
		format!("({})", data.accept(self)?).wrap()
	}

	fn variable(&mut self, data: String, _pos: SourcePos) -> Result<String> {
		data.wrap()
	}

}

impl StmtVisitor<String> for AstPrinter {
	
	fn expr(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<String> {
		format!("({})", data.accept(self)?).wrap()
	}

	fn writeline(&mut self, data: Box<Expression>, _pos: SourcePos) -> Result<String> {
		format!("(writeline {})", data.accept(self)?).wrap()
	}

	fn declaration(&mut self, data: DeclarationData, _pos: SourcePos) -> Result<String> {
		format!("(decl {} {})", data.name, data.expr.accept(self)?).wrap()
	}

	fn assignment(&mut self, data: AssignData, _pos: SourcePos) -> Result<String> {
		format!("(assign {} {})", data.name, data.expr.accept(self)?).wrap()
	}

}
