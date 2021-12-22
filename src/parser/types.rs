
use crate::{utils::{result::{Result, ErrorList, Stage}, wrap::Wrap}, types::Type, lexer::token::{TokenType::*, Keyword, Symbol, Token}, ast::identifier::Identifier};

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
			Identifier(name) => Type::Named(Identifier::new(name)),
			_ => return ErrorList::comp(format!("Expected type, found {}", token), token.pos).err()
		}.wrap()
	}
	
}
