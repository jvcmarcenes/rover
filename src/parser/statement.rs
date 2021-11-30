
use crate::{ast::{expression::{BinaryData, BinaryOperator, CallData, ExprType, FieldData, IndexData, LiteralData, LambdaData}, identifier::Identifier, statement::{AssignData, Block, DeclarationData, IfData, Statement, StmtType, AttrDeclarationData, MethodData}}, lexer::token::{Keyword::*, Token, TokenType::{self, *}, Symbol::*}, utils::{result::{ErrorList, Result, append, throw}, wrap::Wrap}};

use super::Parser;

pub type StmtResult = Result<Statement>;
pub type BlockResult = Result<Block>;

const ASSIGN_OPS: &[TokenType] = &[Symbol(Equals), Symbol(PlusEquals), Symbol(MinusEquals), Symbol(StarEquals), Symbol(SlashEquals)];

impl Parser {

	pub fn program(&mut self) -> BlockResult {
		let mut statements = Block::new();
		let mut errors = ErrorList::new();
		loop {
			self.skip_new_lines();
			if self.is_at_end() { return errors.if_empty(statements); }
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
		let mut errors = ErrorList::new();

		loop {
			self.skip_new_lines();
			match self.peek().typ {
				Symbol(CloseBracket) => break,
				EOF => append!(ret comp "Statement block not closed".to_owned(), pos; to errors),
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
		errors.if_empty(statements)
	}

	pub fn statement(&mut self) -> StmtResult {
		match self.peek().typ {
			Keyword(Let) => self.declaration(),
			Keyword(If) => self.if_stmt(),
			Keyword(Loop) => self.loop_stmt(),
			Keyword(For) => self.for_stmt(),
			Keyword(Break) => self.break_stmt(),
			Keyword(Continue) => self.continue_stmt(),
			Keyword(Return) => self.return_stmt(),
			Keyword(Attr) => self.attr_declaration(),
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
		let mut errors = ErrorList::new();
		let constant = self.optional(Keyword(Const)).is_some();
		let next = self.next();
		let name = match next.typ { 
			TokenType::Identifier(name) => Identifier::new(name),
			typ => append!(ErrorList::comp(format!("Expected identifier, found {}", typ), next.pos).err(); to errors; dummy Identifier::new("".to_string())),
		};
		let expr = match self.optional(Symbol(Equals)) {
			Some(_) => append!(self.expression(); to errors),
			None => ExprType::Literal(LiteralData::None).to_expr(next.pos),
		};
		errors.try_append(self.expect_eol());
		errors.if_empty(
			StmtType::Declaration(DeclarationData { constant, name, expr: Box::new(expr) }).to_stmt(pos)
		)
	}

	fn attr_declaration(&mut self) -> StmtResult {
		let mut errors = ErrorList::new();
		let Token { pos, .. } = self.next();
		let next = self.next();
		let name = match next.typ { 
			TokenType::Identifier(name) => Identifier::new(name),
			typ => append!(ErrorList::comp(format!("Expected identifier, found {}", typ), next.pos).err(); to errors; dummy Identifier::new("".to_string())),
		};
		self.skip_new_lines();
		errors.try_append(self.expect(Symbol(OpenBracket)));
		let mut methods = Vec::new();
		loop {
			self.skip_new_lines();
			let next = self.next();
			match next.typ {
				Symbol(CloseBracket) => return StmtType::AttrDeclaration(AttrDeclarationData { name, methods }).to_stmt(pos).wrap(),
				Identifier(name) => {
					let LambdaData { params, body } = self.lambda_data()?;
					methods.push(MethodData { name, params, body });
				},
				typ => append!(ret comp format!("Expected Identifier or CLOSE_BRACKET, found {}", typ), pos; to errors),
			}
		}

	}

	fn if_stmt(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		let mut errors = ErrorList::new();
		let cond = append!(self.expression(); to errors; with self.synchronize_until(Symbol(OpenBracket)); or none);
		let then_block = append!(self.block(); to errors; dummy vec![]);
		self.skip_new_lines();
		let else_block = match self.optional(Keyword(Else)) {
			Some(_) if self.next_match(Keyword(If)) => Block::from([append!(self.if_stmt(); to errors)]),
			Some(_) => append!(self.block(); to errors),
			None => Block::new(),
		};
		throw!(errors);
		StmtType::If(IfData { cond: Box::new(cond.unwrap()), then_block, else_block }).to_stmt(pos).wrap()
	}

	fn loop_stmt(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		let block = self.block()?;
		StmtType::Loop(block).to_stmt(pos).wrap()
	}

	fn for_stmt(&mut self) -> StmtResult {
		let Token { pos, .. } = self.next();
		let mut errors = ErrorList::new();

		let next = self.next();
		if let Identifier(name) = next.typ {
			errors.try_append(self.expect(Keyword(In)));
			let list = append!(self.expression(); to errors; with {
				self.synchronize_until(Symbol(OpenBracket));
				append!(self.block(); to errors);
			});
			let body = append!(self.block(); to errors);

			StmtType::Scoped(vec![
				StmtType::Declaration(DeclarationData { constant: false, name: Identifier::new("$i".to_owned()), expr: ExprType::Literal(LiteralData::Num(-1.0)).to_expr(pos).wrap() }).to_stmt(pos),
				StmtType::Declaration(DeclarationData { constant: true, name: Identifier::new("$list".to_owned()), expr: list.wrap() }).to_stmt(pos),
				StmtType::Declaration(DeclarationData {
					constant: true, name: Identifier::new("$len".to_owned()),
					expr: ExprType::Call(CallData {
						calee: ExprType::FieldGet(FieldData {
							head: ExprType::Variable(Identifier::new("$list".to_owned())).to_expr(pos).wrap(),
							field: "size".to_owned(),
						}).to_expr(pos).wrap(), 
						args: vec![]
					}).to_expr(pos).wrap()
				}).to_stmt(pos),
				StmtType::Loop(vec![
					StmtType::Assignment(AssignData {
						head: ExprType::Variable(Identifier::new("$i".to_owned())).to_expr(pos).wrap(),
						l_pos: pos,
						expr: ExprType::Binary(BinaryData {
							lhs: ExprType::Variable(Identifier::new("$i".to_owned())).to_expr(pos).wrap(),
							op: BinaryOperator::Add,
							rhs: ExprType::Literal(LiteralData::Num(1.0)).to_expr(pos).wrap(),
						}).to_expr(pos).wrap(),
					}).to_stmt(pos),
					StmtType::If(IfData { 
						cond: Box::new(ExprType::Binary(BinaryData {
							lhs: ExprType::Variable(Identifier::new("$i".to_owned())).to_expr(pos).wrap(),
							op: BinaryOperator::Gre,
							rhs: ExprType::Variable(Identifier::new("$len".to_owned())).to_expr(pos).wrap(),
						}).to_expr(pos)),
						then_block: vec![StmtType::Break.to_stmt(pos)],
						else_block: vec![],
					}).to_stmt(pos),
					StmtType::Declaration(DeclarationData {
						constant: false, name: Identifier::new(name),
						expr: ExprType::Index(IndexData {
							head: ExprType::Variable(Identifier::new("$list".to_owned())).to_expr(pos).wrap(),
							index: ExprType::Variable(Identifier::new("$i".to_owned())).to_expr(pos).wrap(),
						}).to_expr(pos).wrap(),
					}).to_stmt(pos),
					StmtType::Scoped(body).to_stmt(pos),
				]).to_stmt(pos),
			]).to_stmt(pos).wrap()

		} else {
			self.synchronize_with(Symbol(CloseBracket));
			ErrorList::comp(format!("Expected identifier, found {}", next.typ), next.pos).err()
		}
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