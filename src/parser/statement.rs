
use crate::{ast::{Identifier, expression::{BinaryData, BinaryOperator, ExprType, LiteralData}, statement::{AssignData, Block, DeclarationData, IfData, Statement, StmtType}}, lexer::token::{Keyword::*, Token, TokenType::{self, *}, Symbol::*}, utils::{result::{ErrorList, Result}, wrap::Wrap}};

use super::Parser;

pub type StmtResult = Result<Statement>;
pub type BlockResult = Result<Block>;

const ASSIGN_OPS: &[TokenType] = &[Symbol(Equals), Symbol(PlusEquals), Symbol(MinusEquals), Symbol(StarEquals), Symbol(SlashEquals)];

impl Parser {

	pub fn program(&mut self) -> BlockResult {
		let mut statements = Block::new();
		let mut errors = ErrorList::empty();
		loop {
			self.skip_new_lines();
			if self.is_at_end() { 
				return if errors.is_empty() {
					statements.wrap()
				} else {
					errors.err()
				}
			}
			match self.statement() {
				Ok(stmt) => statements.push(stmt),
				Err(err) => {
					errors.append(err);
					self.synchronize();
				}
			}
		}
	}

	pub(super) fn block(&mut self) -> BlockResult {
		self.skip_new_lines();
		let Token { pos, .. } = self.expect(Symbol(OpenBracket))?;

		let mut statements = Block::new();
		let mut errors = ErrorList::empty();

		loop {
			self.skip_new_lines();
			match self.peek().typ {
				Symbol(CloseBracket) => break,
				EOF => { errors.add_comp("Statement block not closed".to_owned(), pos); break },
				_ => match self.statement() {
					Ok(stmt) => statements.push(stmt),
					Err(err) => {
						errors.append(err);
						self.synchronize();
					}
				},
			}
		}
		self.next();

		if errors.is_empty() {
			Ok(statements)
		} else {
			Err(errors)
		}
	}

	pub fn statement(&mut self) -> StmtResult {
		let peek = self.peek();
		match peek.typ {
			Keyword(Let) => self.declaration(),
			Keyword(If) => self.if_stmt(),
			Keyword(Loop) => self.loop_stmt(),
			Keyword(Break) => self.break_stmt(),
			Keyword(Continue) => self.continue_stmt(),
			Keyword(Return) => self.return_stmt(),
			_ => self.assignment_or_expression(),
		}
	}

	fn assignment_or_expression(&mut self) -> StmtResult {
		let left = self.expression()?;
		let l_pos = left.pos;
		if let Some(token) = self.optional_any(ASSIGN_OPS) {
			match left.typ {
				ExprType::Variable(_) | ExprType::Index(_) | ExprType::FieldGet(_) => {
					let right = if let Symbol(Equals) = token.typ {
						self.expression()?
					} else {
						let op = match token.typ {
							Symbol(PlusEquals)  => BinaryOperator::Add,
							Symbol(MinusEquals) => BinaryOperator::Sub,
							Symbol(StarEquals)  => BinaryOperator::Mul,
							Symbol(SlashEquals) => BinaryOperator::Div,
							_ => panic!("this should never be reached"),
						};
						ExprType::Binary(BinaryData { lhs: Box::new(left.clone()), op, rhs: Box::new(self.expression()?) }).to_expr(token.pos)
					};
					self.expect_eol()?;
					StmtType::Assignment(AssignData { head: Box::new(left), l_pos, expr: Box::new(right) }).to_stmt(token.pos).wrap()
				},
				_ => {
					self.synchronize();
					ErrorList::comp("Invalid assignment target".to_owned(), l_pos).err()
				}
			}
		} else {
			self.expect_eol()?;				
			StmtType::Expr(Box::new(left)).to_stmt(l_pos).wrap()
		}
	}

	fn declaration(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		let constant = self.optional(Keyword(Const)).is_some();
		let next = self.next();
		let name = match next.typ { 
			TokenType::Identifier(name) => Identifier::new(name),
			typ => return ErrorList::comp(format!("Expected identifier, found {}", typ), next.pos).err(),
		};
		let expr = match self.optional(Symbol(Equals)) {
			Some(_) => self.expression()?,
			None => ExprType::Literal(LiteralData::None).to_expr(next.pos),
		};
		self.expect_eol()?;
		StmtType::Declaration(DeclarationData { constant, name, expr: Box::new(expr) }).to_stmt(pos).wrap()
	}

	fn if_stmt(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		let cond = self.expression()?;
		let then_block = self.block()?;
		self.skip_new_lines();
		let else_block = match self.optional(Keyword(Else)) {
			Some(_) if self.next_match(Keyword(If)) => Block::from([self.if_stmt()?]),
			Some(_) => self.block()?,
			None => Block::new(),
		};
		StmtType::If(IfData { cond: Box::new(cond), then_block, else_block }).to_stmt(pos).wrap()
	}

	fn loop_stmt(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		let block = self.block()?;
		StmtType::Loop(block).to_stmt(pos).wrap()
	}

	fn break_stmt(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		StmtType::Break.to_stmt(pos).wrap()
	}

	fn continue_stmt(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		StmtType::Continue.to_stmt(pos).wrap()
	}
	
	fn return_stmt(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		let expr = self.expression_or_none()?;
		StmtType::Return(Box::new(expr)).to_stmt(pos).wrap()
	}

}