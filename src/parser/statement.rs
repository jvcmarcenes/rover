
use std::collections::{HashMap, HashSet};

use crate::{ast::{Block, expression::*, identifier::Identifier, statement::*}, lexer::token::{Keyword::*, Token, TokenType::{self, *}, Symbol::*}, utils::{result::{ErrorList, Result, append, throw, Stage}, wrap::Wrap}, types::Type};

use super::Parser;

const ASSIGN_OPS: &[TokenType] = &[Symbol(Equals), Symbol(PlusEquals), Symbol(MinusEquals), Symbol(StarEquals), Symbol(SlashEquals)];

impl Parser {

	pub(super) fn top_declaration(&mut self) -> Result<()> {
		match self.peek().typ {
			Keyword(Attr) => self.attr_declaration(),
			Keyword(Function) => self.func_declaration(),
			Keyword(Type) => self.type_alias(),
			_ => ErrorList::comp("Expected a declaration".to_owned(), self.next().pos).err()
		}
	}

	fn func_declaration(&mut self) -> Result<()> {
		let Token { pos, .. } = self.next();
		let next = self.next();
		match next.typ {
			Identifier(name) => {
				let id = Identifier::new(name);

				let LambdaData { params, types, returns, body } = self.lambda_data()?;
				let decl = StmtType::FuncDeclaration(FunctionData { name: id.clone(), params, types, returns, body }).to_stmt(pos);

				self.module.add(id.clone(), decl, pos)?;

				Ok(())
			}
			typ => ErrorList::comp(format!("Expected function name, found {}", typ), next.pos).err()
		}
	}

	fn attr_declaration(&mut self) -> Result<()> {
		let mut errors = ErrorList::new();
		let Token { pos, .. } = self.next();
		let next = self.next();
		let id = match next.typ { 
			TokenType::Identifier(name) => Identifier::new(name),
			typ => append!(ErrorList::comp(format!("Expected identifier, found {}", typ), next.pos).err(); to errors; dummy Identifier::new("".to_string())),
		};
		self.skip_new_lines();
		errors.try_append(self.expect(Symbol(OpenBracket)));
		let mut methods = Vec::new();
		let mut fields = HashMap::new();
		let mut attributes = HashSet::new();
		loop {
			self.skip_new_lines();
			let next = self.next();
			match next.typ {
				Symbol(CloseBracket) => break,
				Keyword(Static) => {
					match self.obj_field() {
						Ok((name, expr)) => { fields.insert(name, expr); },
						Err(err) => { errors.append(err); continue },
					}
					errors.try_append(self.expect_eol());
				},
				Keyword(Is) => {
					let next = self.next();
					match next.typ {
						Identifier(name) => { attributes.insert(Identifier::new(name)); },
						typ => { errors.add_comp(format!("Expected identifier, found {}", typ), next.pos); self.synchronize() },
					}
					if self.next_match(Symbol(CloseBracket)) { continue; }
					errors.try_append(self.expect_eol());
				}
				Identifier(name) => {
					let LambdaData { params, types, returns, body } = self.lambda_data()?;
					methods.push(FunctionData { name: Identifier::new(name), params, types, returns, body });
					errors.try_append(self.expect_eol());
				},
				typ => append!(ret comp format!("Expected Identifier or CLOSE_BRACKET, found {}", typ), pos; to errors),
			}
		}

		let decl = StmtType::AttrDeclaration(AttrDeclarationData { name: id.clone(), fields, methods, attributes }).to_stmt(pos);

		self.module.add(id.clone(), decl, pos)?;

		errors.if_empty(())
	}

	fn type_alias(&mut self) -> Result<()> {
		let Token { pos, .. } = self.next();
		let mut errors = ErrorList::new();
		let next = self.next();
		let alias = match next.typ {
			TokenType::Identifier(name) => Identifier::new(name),
			typ => append!(ErrorList::comp(format!("Expected identifier, found {}", typ), next.pos).err(); to errors; dummy Identifier::new("".to_string())),
		};
		append!(self.expect(Symbol(Equals)); to errors);
		let typ = append!(self.types(); to errors);
		let typ = append!(typ.validate(Stage::Compile, pos); to errors);
		errors.try_append(self.expect_eol());

		let decl = StmtType::TypeAlias(AliasData { alias: alias.clone(), typ }).to_stmt(pos);

		errors.try_append(self.module.add(alias.clone(), decl, pos));
		
		errors.if_empty(())
	}

	pub(super) fn statement(&mut self, accept_decl: bool) -> Result<Option<Statement>> {
		match self.peek().typ {
			Keyword(Let) => self.declaration(),
			Keyword(If) => self.if_stmt(),
			Keyword(Loop) => self.loop_stmt(),
			Keyword(For) => self.for_stmt(),
			Keyword(Break) => self.break_stmt(),
			Keyword(Continue) => self.continue_stmt(),
			Keyword(Return) => self.return_stmt(),
			Keyword(Attr) => return if accept_decl { self.attr_declaration()?; None.wrap() } else { ErrorList::comp("Attribute declarations are only allowed in the top level".to_owned(), self.next().pos).err() },
			Keyword(Function) => return if accept_decl { self.func_declaration()?; None.wrap() } else { ErrorList::comp("Function declarations are only allowed in the top level".to_owned(), self.next().pos).err() },
			Keyword(Type) => return if accept_decl { self.type_alias()?; None.wrap() } else { ErrorList::comp("Type aliases are only allowed in the top level".to_owned(), self.next().pos).err() },
			_ => self.assignment_or_expression(),
		}?.wrap()
	}

	fn assignment_or_expression(&mut self) -> Result<Statement> {
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

	fn declaration(&mut self) -> Result<Statement> {
		let Token { pos, .. } = self.next();
		let mut errors = ErrorList::new();
		let constant = self.optional(Keyword(Const)).is_some();
		let next = self.next();
		let name = match next.typ { 
			TokenType::Identifier(name) => Identifier::new(name),
			typ => append!(ErrorList::comp(format!("Expected identifier, found {}", typ), next.pos).err(); to errors; dummy Identifier::new("".to_string())),
		};

		let type_restriction = append!(self.type_restriction(); to errors; dummy Type::Void.wrap());

		let expr = match self.optional(Symbol(Equals)) {
			Some(_) => append!(self.expression(); to errors),
			None => ExprType::Literal(LiteralData::None).to_expr(next.pos),
		};
		errors.try_append(self.expect_eol());
		errors.if_empty(
			StmtType::Declaration(DeclarationData { constant, name, type_restriction, expr: Box::new(expr) }).to_stmt(pos)
		)
	}

	fn if_stmt(&mut self) -> Result<Statement> {
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

	fn loop_stmt(&mut self) -> Result<Statement> {
		let Token { pos, .. } = self.next();
		let block = self.block()?;
		StmtType::Loop(block).to_stmt(pos).wrap()
	}

	fn for_stmt(&mut self) -> Result<Statement> {
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
				StmtType::Declaration(DeclarationData { constant: false, name: Identifier::new("$i".to_owned()), type_restriction: None, expr: ExprType::Literal(LiteralData::Num(-1.0)).to_expr(pos).wrap() }).to_stmt(pos),
				StmtType::Declaration(DeclarationData { constant: true, name: Identifier::new("$list".to_owned()), type_restriction: None, expr: list.wrap() }).to_stmt(pos),
				StmtType::Declaration(DeclarationData {
					constant: true, name: Identifier::new("$len".to_owned()),
					type_restriction: None,
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
						type_restriction: None,
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

	fn break_stmt(&mut self) -> Result<Statement> {
		let Token { pos, .. } = self.next();
		StmtType::Break.to_stmt(pos).wrap()
	}

	fn continue_stmt(&mut self) -> Result<Statement> {
		let Token { pos, .. } = self.next();
		StmtType::Continue.to_stmt(pos).wrap()
	}
	
	fn return_stmt(&mut self) -> Result<Statement> {
		let Token { pos, .. } = self.next();
		let expr = self.expression_or_none()?;
		StmtType::Return(expr.wrap()).to_stmt(pos).wrap()
	}

}
