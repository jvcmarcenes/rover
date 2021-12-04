
use std::collections::{HashMap, HashSet};

use crate::{ast::{identifier::Identifier, expression::{*, BinaryOperator::{self, *}, ExprType::{self, *}, UnaryOperator::{self, *}}, statement::{Block, DeclarationData, IfData, StmtType}}, lexer::token::{Keyword::*, LiteralType, Symbol::*, Token, TokenType::{*, self}}, utils::{result::{ErrorList, Result, append}, source_pos::SourcePos, wrap::Wrap}};

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
		Keyword(Is) => Typ,
		_ => panic!("This function should only be called when we know it will match"),
	}
}

fn un_operator_for_token(token: &Token) -> UnaryOperator {
	match token.typ {
		Symbol(Exclam) => Not,
		Symbol(Plus) => Pos,
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

fn err_handler(expr: Expression, handler: Block, pos: SourcePos) -> Expression {
	ExprType::DoExpr(vec![
		StmtType::Declaration(DeclarationData { constant: true, name: Identifier::new("$res".to_owned()), expr: expr.wrap() }).to_stmt(pos),
		StmtType::If(IfData {
			cond: ExprType::Binary(BinaryData { 
				lhs: ExprType::Variable(Identifier::new("$res".to_owned())).to_expr(pos).wrap(),
				op: BinaryOperator::Typ,
				rhs: ExprType::Variable(Identifier::new("Error".to_owned())).to_expr(pos).wrap(),
			}).to_expr(pos).wrap(),
			then_block: handler,
			else_block: vec![],
		}).to_stmt(pos),
		StmtType::Expr(ExprType::Variable(Identifier::new("$res".to_owned())).to_expr(pos).wrap()).to_stmt(pos),
	]).to_expr(pos)
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
		self.bind_expr()
	}
	
	fn bind_expr(&mut self) -> ExprResult {
		fn access(parser: &mut Parser) -> ExprResult {
			let Token { typ, pos } = parser.next();
			if let Identifier(name) = typ {
				let mut head = ExprType::Variable(Identifier::new(name)).to_expr(pos);
				loop {
					let peek = parser.peek();
					head = match peek.typ {
						Symbol(OpenSqr) => parser.index(head)?,
						Symbol(Dot) => parser.field(head)?,
						_ => return head.wrap(),
					}
				}
			} else {
				ErrorList::comp(format!("Expected identifier, found {}", typ), pos).err()
			}
		}
		
		let mut expr = self.logic()?;
		
		if let Some(Token { pos, .. }) = self.optional(Symbol(DoubleColon)) {
			let method = access(self)?;
			expr = ExprType::Binding(BindData { expr: Box::new(expr), method: Box::new(method) }).to_expr(pos);
		}

		if self.next_match(Symbol(OpenPar)) {
			expr = self.function_call(expr)?;
		}
		
		expr.wrap()
	}
	
	fn logic(&mut self) -> ExprResult {
		let mut left = self.equality()?;
		while let Some(token) = self.optional_any(&[Keyword(And), Keyword(Or)]) {
			let op = lg_operator_for_token(&token);
			let right = self.equality()?;
			left = Logic(LogicData { lhs: Box::new(left), op, rhs: Box::new(right) }).to_expr(token.pos);
		}
		left.wrap()
	}
	
	fn binary<F : FnMut(&mut Self) -> ExprResult>(&mut self, mut operand: F, operators: &[TokenType]) -> ExprResult {
		let mut left = operand(self)?;
		while let Some(token) = self.optional_any(operators) {
			let op = bin_operation_for_token(&token);
			let right = operand(self)?;
			left = Binary(BinaryData { lhs: Box::new(left), op, rhs: Box::new(right) }).to_expr(token.pos);
		}
		left.wrap()
	}
	
	fn equality(&mut self) -> ExprResult {
		self.binary(|parser| parser.comparison(), &[Symbol(DoubleEquals), Symbol(ExclamEquals)])
	}
	
	fn comparison(&mut self) -> ExprResult {
		self.binary(|parser| parser.term(), &[Symbol(CloseAng), Symbol(CloseAngEquals), Symbol(OpenAng), Symbol(OpenAngEquals), Keyword(Is)])
	}
	
	fn term(&mut self) -> ExprResult {
		self.binary(|parser| parser.factor(), &[Symbol(Plus), Symbol(Minus), Keyword(Mod)])
	}
	
	fn factor(&mut self) -> ExprResult {
		self.binary(|parser| parser.unary(), &[Symbol(Star), Symbol(Slash)])
	}
	
	fn unary(&mut self) -> ExprResult {
		if let Some(token) = self.optional_any(&[Symbol(Exclam), Symbol(Minus), Symbol(Plus)]) {
			let op = un_operator_for_token(&token);
			let expr = self.unary()?;
			Unary(UnaryData { op, expr: Box::new(expr) }).to_expr(token.pos).wrap()
		} else {
			self.pipe_infix()
		}
	}
	
	fn pipe_infix(&mut self) -> ExprResult {
		let mut expr = self.postfix()?;
		while let Symbol(BarCloseAng) = self.peek().typ {
			let Token { pos, .. } = self.next();
			let calee = self.postfix()?;
			expr = Call(CallData { calee: Box::new(calee), args: vec![expr] }).to_expr(pos);
		}
		expr.wrap()
	}
	
	fn postfix(&mut self) -> ExprResult {
		let mut expr = self.primary()?;
		loop {
			expr = match self.peek().typ {
				Symbol(OpenPar) => self.function_call(expr)?,
				Symbol(OpenSqr) => self.index(expr)?,
				Symbol(Dot) => self.field(expr)?,
				Symbol(Question) => {
					let Token { pos, .. } = self.next();
					err_handler(expr, vec![StmtType::Return(ExprType::Variable(Identifier::new("$res".to_owned())).to_expr(pos).wrap()).to_stmt(pos)], pos)
				}
				Symbol(Exclam) => {
					let Token { pos, .. } = self.next();
					err_handler(expr, vec![StmtType::Expr(ExprType::Call(CallData {
						calee: ExprType::Variable(Identifier::new("abort".to_owned())).to_expr(pos).wrap(),
						args: vec![ExprType::Variable(Identifier::new("$res".to_owned())).to_expr(pos)],
					}).to_expr(pos).wrap()).to_stmt(pos)], pos)
				}
				_ => return expr.wrap(),
			};
		}
	}
	
	fn expr_list(&mut self, stop: TokenType) -> Result<Vec<Expression>> {
		let mut exprs = Vec::new();
		let mut errors = ErrorList::new();
		loop {
			self.skip_new_lines();
			let peek = self.peek();
			match peek.typ {
				EOF => append!(ret comp "Unexpected EOF".to_owned(), peek.pos; to errors),
				typ if typ == stop => break,
				_ => {
					match self.expression() {
						Ok(expr) => exprs.push(expr),
						Err(err) => errors.append(err),
					}
					if self.peek().typ == stop { continue; }
					if let Err(err) = self.expect_any(&[Symbol(Comma), EOL]) {
						errors.append(err);
						self.synchronize_complex(&[Symbol(Comma)], &[stop.clone()]);
					}
				}
			}
		}
		errors.if_empty(exprs)
	}
	
	fn function_call(&mut self, calee: Expression) -> ExprResult {
		let Token { pos, .. } = self.next();
		let args = self.expr_list(Symbol(ClosePar))?;
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
		match token.typ {
			Keyword(False) => ExprType::Literal(LiteralData::Bool(false)),
			Keyword(True) => ExprType::Literal(LiteralData::Bool(true)),
			Keyword(_None) => ExprType::Literal(LiteralData::None),
			Keyword(Function) => self.lambda()?,
			Keyword(_Self) => SelfRef,
			Keyword(Do) => DoExpr(self.block()?),
			Keyword(Error) => ExprType::Literal(LiteralData::Error(Box::new(self.expression_or_none()?))),
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
		}.to_expr(token.pos).wrap()
	}
	
	fn list_literal(&mut self) -> Result<ExprType> {
		let mut errors = ErrorList::new();
		let exprs = append!(self.expr_list(Symbol(CloseSqr)); to errors; dummy vec![]);
		errors.try_append(self.expect(Symbol(CloseSqr)));
		errors.if_empty(ExprType::Literal(LiteralData::List(exprs)))
	}
	
	pub(super) fn obj_field(&mut self) -> Result<(String, Expression)> {
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
		let mut errors = ErrorList::new();
		let mut map = HashMap::new();
		let mut attrs = HashSet::new();
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
					return errors.if_empty(ExprType::Literal(LiteralData::Object(map, attrs)));
				},
				Keyword(Is) => {
					self.next();
					let next = self.next();
					match next.typ {
						Identifier(name) => { attrs.insert(Identifier::new(name)); },
						typ => { errors.add_comp(format!("Expected identifier, found {}", typ), next.pos); self.synchronize() }
					}
					if self.next_match(Symbol(CloseBracket)) { continue; }
					errors.try_append(self.expect_any_or_sync(&[Symbol(Comma), EOL]));
				},
				_ => {
					match self.obj_field() {
						Ok((name, expr)) => { map.insert(name, expr); },
						Err(err) => { errors.append(err); continue },
					}
					if self.next_match(Symbol(CloseBracket)) { continue; }
					errors.try_append(self.expect_any_or_sync(&[Symbol(Comma), EOL]));
				}
			}
		}
	}
	
	fn str_template(&mut self, tokens: Vec<Token>) -> Result<ExprType> {
		let mut exprs = Vec::new();
		let mut errors = ErrorList::new();
		
		let mut template_parser = Parser::new(tokens);
		
		loop {
			match template_parser.peek().typ {
				EOF => break,
				Symbol(HashtagOpenBracket) => {
					template_parser.next();
					match template_parser.expression() {
						Ok(expr) => {
							exprs.push(expr);
							errors.try_append(template_parser.expect_or_sync(Symbol(CloseBracket)));
						}
						Err(err) => {
							errors.append(err);
							template_parser.synchronize_with(Symbol(CloseBracket));
						}
					}
				}
				_ => exprs.push(template_parser.expression().expect("this should never be an error")),
			}
		}
		
		errors.if_empty(ExprType::Literal(LiteralData::Template(exprs)))
	}
	
	pub(super) fn lambda_data(&mut self) -> Result<LambdaData> {
		self.expect_or_sync(Symbol(OpenPar))?;
		let mut params = Vec::new();
		let mut errors = ErrorList::new();
		loop {
			let peek = self.peek();
			match peek.typ {
				EOF => {
					errors.add_comp("Unexpected EOF".to_owned(), peek.pos);
					return errors.err();
				},
				Symbol(ClosePar) => { self.next(); break; }
				Identifier(name) if params.is_empty() => { self.next(); params.push(Identifier::new(name)); },
				Symbol(Comma) => {
					self.next();
					self.skip_new_lines();
					let next = self.next();
					if let Identifier(name) = next.typ {
						params.push(Identifier::new(name))
					} else {
						errors.add_comp(format!("Expected identifier, found {}", next), next.pos);
						self.synchronize_until_any(&[Symbol(Comma), Symbol(ClosePar)]);
					}
				},
				_ => {
					if params.is_empty() {
						errors.add_comp(format!("Expected identifier, found {}", peek), peek.pos);
					} else {
						errors.add_comp(format!("Expected COMMA or CLOSE_PAR, found {}", peek), peek.pos);
					}
					self.synchronize_until_any(&[Symbol(Comma), Symbol(ClosePar)]);
				}
			}
		}
		
		let body = if let Some(Token { pos, .. }) = self.optional(Symbol(EqualsCloseAng)) {
			let expr = append!(self.expression(); to errors);
			Block::from([StmtType::Return(Box::new(expr)).to_stmt(pos)])
		} else {
			append!(self.block(); to errors)
		};
		
		errors.if_empty(LambdaData { params, body })
	}
	
	fn lambda(&mut self) -> Result<ExprType> {
		self.lambda_data().map(|data| ExprType::Lambda(data))
	}
	
}
	