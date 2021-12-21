
use crate::{utils::{result::{Result, ErrorList}, wrap::Wrap}, types::{Type, TypePrim}, lexer::token::{TokenType::*, Keyword, Symbol}};

use super::Parser;

type TypeResult = Result<Type>;

impl Parser {

	pub fn type_restriction(&mut self) -> Result<Option<Type>> {
		if self.optional(Symbol(Symbol::Colon)).is_some() {
			self.types().wrap()
		} else {
			None.wrap()
		}
	}

	fn types(&mut self) -> TypeResult {
		self.or_type()
	}

	fn or_type(&mut self) -> TypeResult {
		let first = self.type_optional()?;
		if !self.next_match(Keyword(Keyword::Or)) { return first.wrap(); }
		let mut types = vec![first];
		while let Keyword(Keyword::Or) = self.peek().typ {
			self.next();
			types.push(self.type_optional()?);
		}
		Type::Or(types).wrap()
	}

	fn type_optional(&mut self) -> TypeResult {
		let mut typ = self.type_primitive()?;
		if self.optional(Symbol(Symbol::Question)).is_some() {
			typ = Type::Or(vec![typ, Type::Primitive(TypePrim::None)])
		}
		typ.wrap()
	}
	
	fn type_primitive(&mut self) -> TypeResult {
		let token = self.next();
		match token.typ {
			Keyword(Keyword::StringT) => Type::Primitive(TypePrim::Str),
			Keyword(Keyword::NumberT) => Type::Primitive(TypePrim::Num),
			Keyword(Keyword::BoolT) => Type::Primitive(TypePrim::Bool),
			Keyword(Keyword::AnyT) => Type::Primitive(TypePrim::Any),
			_ => return ErrorList::comp(format!("Expected type, found {}", token), token.pos).err()
		}.wrap()
	}
	
}
