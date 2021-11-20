
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{ast::{Identifier, expression::{BinaryData, BinaryOperator::{self, *}, CallData, ExprType::{self, *}, Expression, FieldData, IndexData, LambdaData, LiteralData, LogicData, LogicOperator, UnaryData, UnaryOperator::{self, *}}, statement::{Block, StmtType}}, lexer::token::{Keyword::*, LiteralType, Symbol::*, Token, TokenType::{*, self}}, utils::{result::{ErrorList, Result}, wrap::Wrap}};

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
			EOL | EOF | Symbol(CloseBracket) => ExprType::Literal(LiteralData::None).to_expr(peek.pos).wrap(),
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
			return self.postfix();
		}
	}

	fn postfix(&mut self) -> ExprResult {
		let mut expr = self.primary()?;
		loop {
			match self.peek().typ {
				Symbol(OpenPar) => expr = self.function_call(expr)?,
				Symbol(OpenSqr) => expr = self.index(expr)?,
				Symbol(Dot) => expr = self.field(expr)?,
				_ => break,
			}
		}
		expr.wrap()
	}

	fn expr_list(&mut self, stop: fn(&TokenType) -> bool) -> Result<Vec<Expression>> {
		let mut exprs = Vec::new();
		let mut errors = ErrorList::empty();
		loop {
			self.skip_new_lines();
			let peek = self.peek();
			match peek.typ {
				EOF => {
					errors.add_comp("Unexpected EOF".to_owned(), peek.pos);
					return errors.err()
				}
				typ if stop(&typ) => break,
				_ => {
					if !exprs.is_empty() {
						if let Err(err) = self.expect(Symbol(Comma)) {
							errors.append(err);
							self.synchronize();
							continue;
						}
					}
					self.skip_new_lines();
					match self.expression() {
						Ok(expr) => exprs.push(expr),
						Err(err) => {
							errors.append(err);
							self.synchronize();
						}
					}
				}
			}
		}
		if errors.is_empty() { exprs.wrap() } else { errors.err() }
	}

	fn function_call(&mut self, calee: Expression) -> ExprResult {
		let Token { pos, .. } = self.next();
		let args = self.expr_list(|typ| *typ == Symbol(ClosePar))?;
		self.expect(Symbol(ClosePar))?;
		Call(CallData { calee: Box::new(calee), args }).to_expr(pos).wrap()
	}

	fn index(&mut self, head: Expression) -> ExprResult {
		let Token { pos, .. } = self.next();
		let index = self.expression()?;
		self.expect(Symbol(CloseSqr))?;
		Index(IndexData { head: Box::new(head), index: Box::new(index) }).to_expr(pos).wrap()
	}

	fn field(&mut self, head: Expression) -> ExprResult {
		let Token { pos, .. } = self.next();
		let next = self.next();
		let field = match next.typ {
			Identifier(name) => name,
			_ => return ErrorList::comp(format!("Expected identifier, found {}", next), next.pos).err()
		};
		ExprType::FieldGet(FieldData { head: Box::new(head), field }).to_expr(pos).wrap()
	}

	fn primary(&mut self) -> ExprResult {
		let token = self.next();
		let expr_typ = match token.typ {
			Keyword(False) => ExprType::Literal(LiteralData::Bool(false)),
			Keyword(True) => ExprType::Literal(LiteralData::Bool(true)),
			Keyword(_None) => ExprType::Literal(LiteralData::None),
			Keyword(Function) => self.lambda()?,
			Keyword(_Self) => SelfRef(Rc::new(RefCell::new(0))),
			TokenType::Literal(lit) => match lit {
				LiteralType::Num(n) => ExprType::Literal(LiteralData::Num(n)),
				LiteralType::Str(s) => ExprType::Literal(LiteralData::Str(s)),
			}
			Symbol(OpenPar) => {
				let expr = self.expression()?;
				self.expect(Symbol(ClosePar))?;
				Grouping(Box::new(expr))
			}
			Symbol(OpenSqr) => self.list_literal()?,
			Symbol(OpenBracket) => self.obj_literal()?,
			Identifier(name) => Variable(Identifier::new(name)),
			Template(tokens) => self.str_template(tokens)?,
			_ => return ErrorList::comp(format!("Expected expression, found {}", token), token.pos).err()
		};
		expr_typ.to_expr(token.pos).wrap()
	}

	fn list_literal(&mut self) -> Result<ExprType> {
		let exprs = self.expr_list(|typ| *typ == Symbol(CloseSqr))?;
		self.expect(Symbol(CloseSqr))?;
		ExprType::Literal(LiteralData::List(exprs)).wrap()
	}

	fn obj_field(&mut self) -> Result<(String, Expression)> {
		let next = self.next();
		if let Identifier(name) = next.typ {
			let expr = if self.optional(Symbol(Equals)).is_some() {
				self.expression()?
			} else {
				ExprType::Variable(Identifier::new(name.clone())).to_expr(next.pos)
			};
			(name, expr).wrap()
		} else {
			ErrorList::comp(format!("Expected identifier, found {}", next), next.pos).err()
		}
	}

	fn obj_literal(&mut self) -> Result<ExprType> {
		let mut map = HashMap::new();
		let mut errors = ErrorList::empty();
		loop {
			self.skip_new_lines();
			let peek = self.peek();
			match peek.typ {
				EOF => {
					errors.add_comp("Unexpected EOF".to_owned(), peek.pos);
					return errors.err()
				},
				Symbol(CloseBracket) => {
					self.next();
					return errors.if_empty(ExprType::Literal(LiteralData::Object(map)));
				},
				_ => {
					if !map.is_empty() {
						if let Err(err) = self.expect(Symbol(Comma)) {
							errors.append(err);
							self.synchronize();
							continue;
						}
					}
					self.skip_new_lines();
					match self.obj_field() {
						Ok((name, expr)) => { map.insert(name, expr); },
						Err(err) => {
							errors.append(err);
							return errors.err();
						}
					}
				}
			}
		}
	}

	fn str_template(&mut self, tokens: Vec<Token>) -> Result<ExprType> {
		let mut exprs = Vec::new();
		let mut errors = ErrorList::empty();

		let mut template_parser = Parser::new(tokens);

		loop {
			match template_parser.peek().typ {
				EOF => break,
				Symbol(HashtagOpenBracket) => {
					template_parser.next();
					match template_parser.expression() {
						Ok(expr) => {
							exprs.push(expr);
							if let Err(err) = template_parser.expect(Symbol(CloseBracket)) {
								errors.append(err);
								template_parser.synchronize();
							}
						}
						Err(err) => {
							errors.append(err);
							template_parser.synchronize_with(Symbol(CloseBracket));
						}
					}
				}
				_ => match template_parser.expression() {
					Ok(expr) => exprs.push(expr),
					Err(err) => {
						errors.append(err);
						template_parser.synchronize_with(Symbol(CloseBracket));
					}
				} 
			}
		}

		if errors.is_empty() {
			ExprType::Literal(LiteralData::Template(exprs)).wrap()
		} else {
			errors.err()
		}
	}

	fn lambda(&mut self) -> Result<ExprType> {
		self.expect(Symbol(OpenPar))?;
		let mut params = Vec::new();
		let mut self_ref = None;
		loop {
			let peek = self.peek();
			match peek.typ {
				Symbol(ClosePar) => { self.next(); break; }
				Keyword(_Self) if params.is_empty() => self_ref = Some(self.next().pos),
				Identifier(name) if params.is_empty() => { self.next(); params.push(Identifier::new(name)); },
				_ => {
					self.expect(Symbol(Comma))?;
					self.skip_new_lines();
					let next = self.next();
					if let Identifier(name) = next.typ {
						params.push(Identifier::new(name))
					} else {
						return ErrorList::comp(format!("Expected identifier, found {}", next), next.pos).err()
					}
				}
			}
		}
		let body = if let Some(Token { pos, .. }) = self.optional(Symbol(EqualsCloseAng)) {
			let expr = self.expression()?;
			self.expect_eol()?;
			Block::from([StmtType::Return(Box::new(expr)).to_stmt(pos)])
		} else {
			self.block()?
		};
		ExprType::Lambda(LambdaData { self_ref, params, body }).wrap()
	}

}
