
use crate::{ast::{expression::{ExprType, Expression, LiteralData}, statement::{AssignData, Block, DeclarationData, IfData, Statement, StmtType}}, lexer::token::{Keyword::*, Token, TokenType::*, Symbol::*}, utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap}};

use super::Parser;

pub type StmtResult = Result<Statement>;
pub type BlockResult = Result<Block>;

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

	fn block(&mut self) -> BlockResult {
		self.skip_new_lines();
		let Token { pos, .. } = self.expect(Symbol(OpenBracket))?;

		let mut statements = Block::new();
		let mut errors = ErrorList::empty();

		loop {
			self.skip_new_lines();
			match self.peek().typ {
				Symbol(CloseBracket) => break,
				EOF => { errors.add("Statement block not closed".to_owned(), pos); break },
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
		let res = match peek.typ {
			Keyword(Writeline) => self.writeline()?,
			Keyword(Let) => self.declaration()?,
			Keyword(If) => self.if_stmt()?,
			_ => {
				let expr = self.expression()?;
				match self.optional(Symbol(Equals)) {
					Some(Token { pos, .. }) => self.assignment(expr, pos)?,
					None => return ErrorList::new("Expeced statement, found expression".to_owned(), expr.pos).err(),
				}
			},
		};
		self.expect_eol()?;
		res.wrap()
	}

	fn writeline(&mut self) -> StmtResult {
		let token = self.next();
		let expr = self.expression_or_none()?;
		StmtType::Writeline(Box::new(expr)).to_stmt(token.pos).wrap()
	}

	fn assignment(&mut self, left: Expression, pos: SourcePos) -> StmtResult {
		let rhs = self.expression()?;
		match left.typ {
			ExprType::Variable(name) => StmtType::Assignment(AssignData { name, l_pos: left.pos, expr: Box::new(rhs) }).to_stmt(pos).wrap(),
			_ => return ErrorList::new("Invalid assignment target".to_owned(), left.pos).err(),
		}
	}

	fn declaration(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		let next = self.next();
		let name = match next.typ { 
			Identifier(name) => name,
			typ => return ErrorList::new(format!("Expected identifier, found {}", typ), next.pos).err(),
		};
		let expr = match self.optional(Symbol(Equals)) {
			Some(_) => self.expression()?,
			None => ExprType::Literal(LiteralData::None).to_expr(next.pos),
		};
		StmtType::Declaration(DeclarationData { name, expr: Box::new(expr) }).to_stmt(pos).wrap()
	}

	fn if_stmt(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		let cond = self.expression()?;
		let then_block = self.block()?;
		self.skip_new_lines();
		let else_block = if self.optional(Keyword(Else)).is_some() {
			if self.next_match(Keyword(If)) {
				Block::from([self.if_stmt()?])
			} else {
				self.block()?
			}
		} else {
			Block::new()
		};
		StmtType::If(IfData { cond: Box::new(cond), then_block, else_block }).to_stmt(pos).wrap()
	}

}