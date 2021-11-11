
use crate::{ast::expression::{BinaryData, BinaryOperator::{self, *}, CallData, ExprType::{self, *}, Expression, LambdaData, LiteralData, LogicData, LogicOperator, UnaryData, UnaryOperator::{self, *}}, lexer::token::{Keyword::*, LiteralType, Symbol::*, Token, TokenType::{*, self}}, utils::{result::{ErrorList, Result}, wrap::Wrap}};

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

fn lg_operator_for_token(token: &Token) -> LogicOperator {
	match token.typ {
		Keyword(And) => LogicOperator::And,
		Keyword(Or) => LogicOperator::Or,
		_ => panic!("This function should only be called when we know it will match"),
	}
}

impl Parser {

	pub fn expression_or_none(&mut self) -> ExprResult {
		let peek = self.peek();
		match peek.typ {
			EOL | EOF => ExprType::Literal(LiteralData::None).to_expr(peek.pos).wrap(),
			_ => self.expression()
		}
	}

	pub fn expression(&mut self) -> ExprResult {
		self.logic()
	}

	fn logic(&mut self) -> ExprResult {
		let mut left = self.equality()?;
		while let Some(token) = self.optional_any(&[Keyword(And), Keyword(Or)]) {
			let op = lg_operator_for_token(&token);
			let right = self.equality()?;
			left = Logic(LogicData { lhs: Box::new(left), op, rhs: Box::new(right) }).to_expr(token.pos);
		}
		return left.wrap();
	}

	fn binary<F : FnMut(&mut Self) -> ExprResult>(&mut self, mut operand: F, operators: &[TokenType]) -> ExprResult {
		let mut left = operand(self)?;
		while let Some(token) = self.optional_any(operators) {
			let op = bin_operation_for_token(&token);
			let right = operand(self)?;
			left = Binary(BinaryData { lhs: Box::new(left), op, rhs: Box::new(right) }).to_expr(token.pos);
		}
		return left.wrap();
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
			return self.call();
		}
	}

	fn call(&mut self) -> ExprResult {
		let mut expr = self.primary()?;

		while let Some(token) = self.optional(Symbol(OpenPar)) {
			let mut args = Vec::new();
			loop {
				let peek = self.peek();
				match peek.typ {
					Symbol(ClosePar) => { self.next(); break; },
					_ if args.is_empty() => args.push(self.expression()?),
					_ => {
						self.expect(Symbol(Comma))?;
						self.skip_new_lines();
						args.push(self.expression()?);
					}
				}
			}
			expr = ExprType::Call(CallData { calee: Box::new(expr), args }).to_expr(token.pos);
		}

		expr.wrap()
	}

	fn primary(&mut self) -> ExprResult {
		let token = self.next();
		let expr_typ = match token.typ {
			Keyword(False) => ExprType::Literal(LiteralData::Bool(false)),
			Keyword(True) => ExprType::Literal(LiteralData::Bool(true)),
			Keyword(_None) => ExprType::Literal(LiteralData::None),
			Keyword(Function) => self.lambda()?,
			TokenType::Literal(lit) => match lit {
				LiteralType::Num(n) => ExprType::Literal(LiteralData::Num(n)),
				LiteralType::Str(s) => ExprType::Literal(LiteralData::Str(s)),
			}
			Symbol(OpenPar) => {
				let expr = self.expression()?;
				self.expect(Symbol(ClosePar))?;
				Grouping(Box::new(expr))
			}
			Identifier(name) => Variable(name),
			_ => return ErrorList::new(format!("Expected expression, found {}", token), token.pos).err()
		};
		expr_typ.to_expr(token.pos).wrap()
	}

	fn lambda(&mut self) -> Result<ExprType> {
		self.expect(Symbol(OpenPar))?;
		let mut params = Vec::new();
		loop {
			let peek = self.peek();
			match peek.typ {
				Symbol(ClosePar) => { self.next(); break; }
				Identifier(name) if params.is_empty() => { self.next(); params.push(name) },
				_ => {
					self.expect(Symbol(Comma))?;
					self.skip_new_lines();
					let next = self.next();
					if let Identifier(name) = next.typ {
						params.push(name)
					} else {
						return ErrorList::new(format!("Expected identifier, found {}", next), next.pos).err()
					}
				}
			}
		}
		let root = if self.ctx.in_func { false } else { self.ctx.in_func = true; true };
		let body = self.block()?;
		if root { self.ctx.in_func = false }
		ExprType::Lambda(LambdaData { params, body }).wrap()
	}

}
