
use crate::{ast::expression::{BinaryData, BinaryOperator::{self, *}, ExprType::{self, *}, Expression, LiteralData, UnaryData, UnaryOperator::{self, *}}, lexer::token::{Keyword::*, LiteralType, Symbol::*, Token, TokenType::{*, self}}, utils::{result::{Error, Result}, wrap::Wrap}};

use super::Parser;

pub type ExprResult = Result<Expression>;

fn bin_operation_for_token(token: &Token) -> BinaryOperator {
	match token.typ {
		Symbol(Plus) => Add,
		Symbol(Minus) => Sub,
		Symbol(Star) => Mul,
		Symbol(Slash) => Div,
		Keyword(Mod) => Rem,
		Symbol(OpenAng) => Lst,
		Symbol(OpenAngEquals) => Lse,
		Symbol(CloseAng) => Grt,
		Symbol(CloseAngEquals) => Gre,
		Symbol(DoubleEquals) => Equ,
		Symbol(ExclamEquals) => Neq,
		// Keyword(And) => And
		// Keyword(Or) => Or
		_ => panic!("This function should only be called when we know it will match"),
	}
}

fn un_operator_for_token(token: &Token) -> UnaryOperator {
	match token.typ {
		Symbol(Exclam) => Not,
		Symbol(Minus) => Neg,
		_ => panic!("This function should only be called when we know it will match"),
	}
}

impl Parser {

	pub fn expression(&mut self) -> ExprResult {
		self.equality()
	}

	fn binary<F : FnMut(&mut Self) -> ExprResult>(&mut self, mut operand: F, operators: &[TokenType]) -> ExprResult {
		let mut left = operand(self)?;
		loop {
			if let Some(token) = self.optional_any(operators) {
				let op = bin_operation_for_token(&token);
				let right = operand(self)?;
				left = Binary(BinaryData { lhs: Box::new(left), op, rhs: Box::new(right) }).to_expr(token.pos);
			} else {
				return left.wrap();
			}
		}
	}

	fn equality(&mut self) -> ExprResult {
		self.binary(|parser| parser.comparison(), &[Symbol(DoubleEquals), Symbol(ExclamEquals)])
	}
	
	fn comparison(&mut self) -> ExprResult {
		self.binary(|parser| parser.term(), &[Symbol(CloseAng), Symbol(CloseAngEquals), Symbol(OpenAng), Symbol(OpenAngEquals)])
	}
	
	fn term(&mut self) -> ExprResult {
		self.binary(|parser| parser.factor(), &[Symbol(Plus), Symbol(Minus), Keyword(Mod)])
	}
	
	fn factor(&mut self) -> ExprResult {
		self.binary(|parser| parser.unary(), &[Symbol(Star), Symbol(Slash)])
	}

	fn unary(&mut self) -> ExprResult {
		if let Some(token) = self.optional_any(&[Symbol(Exclam), Symbol(Minus)]) {
			let op = un_operator_for_token(&token);
			let expr = self.unary()?;
			return Unary(UnaryData { op, expr: Box::new(expr) }).to_expr(token.pos).wrap();
		} else {
			return self.primary();
		}
	}

	fn primary(&mut self) -> ExprResult {
		let token = self.next();
		let expr_typ = match token.typ {
			Keyword(False) => ExprType::Literal(LiteralData::Bool(false)),
			Keyword(True) => ExprType::Literal(LiteralData::Bool(true)),
			TokenType::Literal(lit) => match lit {
				LiteralType::Num(n) => ExprType::Literal(LiteralData::Num(n)),
				LiteralType::Str(s) => ExprType::Literal(LiteralData::Str(s)),
			}
			Symbol(OpenPar) => {
				let expr = self.expression()?;
				self.expect(Symbol(ClosePar))?;
				Grouping(Box::new(expr))
			}
			_ => return Error::new(format!("Expected expression, found {}", token), token.pos).into()
		};
		expr_typ.to_expr(token.pos).wrap()
	}

}
