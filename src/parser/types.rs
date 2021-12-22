
use std::collections::HashMap;

use crate::{utils::{result::{Result, ErrorList, Stage, throw, append}, wrap::Wrap}, types::Type, lexer::token::{TokenType::*, Keyword, Symbol, Token}, ast::identifier::Identifier};

use super::Parser;

type TypeResult = Result<Type>;

impl Parser {

	pub fn type_restriction(&mut self) -> Result<Option<Type>> {
		if let Some(Token { pos, .. }) = self.optional(Symbol(Symbol::Colon)) {
			let typ = self.types()?;
			typ.validate(Stage::Compile, pos)?;
			typ.wrap()
		} else {
			None.wrap()
		}
	}

	pub fn types(&mut self) -> TypeResult {
		self.or_type()
	}

	fn or_type(&mut self) -> TypeResult {
		let first = self.type_optional()?;
		if !self.next_match(Keyword(Keyword::Or)) { return first.wrap(); }
		let mut types = vec![first];
		while let Keyword(Keyword::Or) = self.peek().typ {
			self.next();
			let typ = self.type_optional()?;
			if !types.contains(&typ) { types.push(typ); }
		}
		Type::Or(types).wrap()
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
			Symbol(Symbol::OpenSqr) => {
				let typ = self.types()?;
				self.expect_or_sync(Symbol(Symbol::CloseSqr))?;
				Type::List(typ.wrap())
			},
			Symbol(Symbol::OpenBracket) => {
				let mut errors = ErrorList::new();
				let mut map = HashMap::new();
				loop {
					self.skip_new_lines();
					let peek = self.peek();
					match peek.typ {
						Symbol(Symbol::CloseBracket) => { self.next(); break; }
						Identifier(name) => {
							self.next();
							errors.try_append(self.expect(Symbol(Symbol::Colon)));
							let typ = append!(self.types(); to errors; dummy Type::Void);
							map.insert(name, typ);
							if self.next_match(Symbol(Symbol::CloseBracket)) { continue; }
							errors.try_append(self.expect_any_or_sync(&[Symbol(Symbol::Comma), EOL]));
						},
						typ => { self.next(); append!(ret comp format!("Expected identifier, found {}", typ), peek.pos; to errors) },
					}
				}
				throw!(errors);
				Type::Object(map)
			},
			Identifier(name) => Type::Named(Identifier::new(name)),
			_ => return ErrorList::comp(format!("Expected type, found {}", token), token.pos).err()
		}.wrap()
	}
	
}
