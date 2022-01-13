
use std::collections::HashMap;

use crate::{utils::{result::{Result, ErrorList, Stage, throw, append}, wrap::Wrap}, types::Type, lexer::token::{TokenType::*, Keyword, Symbol, Token}, ast::identifier::Identifier};

use super::Parser;

type TypeResult = Result<Type>;

impl Parser {

	pub fn type_restriction(&mut self) -> Result<Option<Type>> {
		if let Some(Token { pos, .. }) = self.optional(Symbol(Symbol::Colon)) {
			self.types()?.validate(Stage::Compile, pos)?.wrap()
		} else {
			None.wrap()
		}
	}

	pub fn types(&mut self) -> TypeResult {
		self.or_type()
	}

	fn or_type(&mut self) -> TypeResult {
		let first = self.and_type()?;
		if !self.next_match(Keyword(Keyword::Or)) { return first.wrap(); }
		let mut types = vec![first];
		while let Keyword(Keyword::Or) = self.peek().typ {
			self.next();
			let typ = self.and_type()?;
			if !types.contains(&typ) { types.push(typ); }
		}
		Type::Or(types).wrap()
	}

	fn and_type(&mut self) -> TypeResult {
		let first = self.type_optional()?;
		if !self.next_match(Keyword(Keyword::And)) { return first.wrap(); }
		let mut types = vec![first];
		while let Keyword(Keyword::And) = self.peek().typ {
			self.next();
			let typ = self.type_optional()?;
			if !types.contains(&typ) { types.push(typ); }
		}
		Type::And(types).wrap()
	}

	fn type_optional(&mut self) -> TypeResult {
		let mut typ = self.type_primitive()?;
		if self.optional(Symbol(Symbol::Question)).is_some() {
			typ = Type::Or(vec![typ, Type::None])
		}
		typ.wrap()
	}
	
	fn type_primitive(&mut self) -> TypeResult {
		let token = self.next();
		match token.typ {
			Keyword(Keyword::StringT) => Type::STR,
			Keyword(Keyword::NumberT) => Type::NUM,
			Keyword(Keyword::BoolT) => Type::BOOL,
			Keyword(Keyword::AnyT) => Type::Any,
			Keyword(Keyword::Void) => Type::Void,
			Keyword(Keyword::Function) => {
				let mut errors = ErrorList::new();
				let mut params = Vec::new();
				self.expect(Symbol(Symbol::OpenPar))?;
				loop {
					let peek = self.peek();
					match peek.typ {
						Symbol(Symbol::ClosePar) => { self.next(); break; }
						_ => {
							let typ = append!(self.types(); to errors; dummy Type::Void);
							params.push(typ);
							if self.next_match(Symbol(Symbol::ClosePar)) { continue; }
							errors.try_append(self.expect_or_sync(Symbol(Symbol::Comma)));
						}
					}
				}
				
				append!(self.expect(Symbol(Symbol::MinusCloseAng)); to errors);

				let returns = append!(self.types(); to errors; dummy Type::Void).wrap();

				Type::Function { params, returns }
			}
			Symbol(Symbol::OpenSqr) => {
				let typ = self.types()?;
				self.expect_or_sync(Symbol(Symbol::CloseSqr))?;
				Type::List(typ.wrap())
			}
			Symbol(Symbol::OpenBracket) => {
				let mut errors = ErrorList::new();
				let mut map = HashMap::new();
				loop {
					self.skip_new_lines();
					let next = self.next();
					match next.typ {
						Symbol(Symbol::CloseBracket) => break,
						Identifier(name) => {
							errors.try_append(self.expect(Symbol(Symbol::Colon)));
							let typ = append!(self.types(); to errors; dummy Type::Void);
							map.insert(name, typ);
							if self.next_match(Symbol(Symbol::CloseBracket)) { continue; }
							errors.try_append(self.expect_any_or_sync(&[Symbol(Symbol::Comma), EOL]));
						},
						typ => append!(ret comp format!("Expected identifier, found {}", typ), next.pos; to errors),
					}
				}
				throw!(errors);
				Type::Object(map)
			}
			Symbol(Symbol::OpenPar) => {
				let typ = self.types()?;
				self.expect(Symbol(Symbol::ClosePar))?;
				typ
			}
			Identifier(name) => {
				let typ = Type::Named(Identifier::new(name.clone()));

				if self.optional(Symbol(Symbol::OpenAng)).is_some() {
					let mut errors = ErrorList::new();
					let mut type_args = Vec::new();
					
					loop {
						self.skip_new_lines();
						let peek = self.peek();
						match peek.typ {
							EOF => append!(ret comp "Unexpected EOF".to_owned(), peek.pos; to errors),
							Symbol(Symbol::CloseAng) => { self.next(); break; }
							_ => {
								let typ = append!(self.types(); to errors; dummy Type::Void);
								type_args.push(typ);
		
								if self.next_match(Symbol(Symbol::CloseAng)) { continue; }
								if let Err(err) = self.expect_any(&[Symbol(Symbol::Comma), EOL]) {
									errors.append(err);
									self.synchronize_complex(&[Symbol(Symbol::Comma)], &[Symbol(Symbol::CloseAng)]);
								}
							}
						}
					}
		
					Type::Generic { base: Identifier::new(name), args: type_args }
				} else {
					typ
				}
			}
			_ => return ErrorList::comp(format!("Expected type, found {}", token), token.pos).err()
		}.wrap()
	}

}
