
use crate::{utils::{result::{Result, ErrorList}, wrap::Wrap}, types::{Type, TypePrim}, lexer::token::{TokenType::*, Keyword, Symbol}};

use super::Parser;

type TypeResult = Result<Type>;

impl Parser {

	pub fn type_restriction(&mut self) -> TypeResult {
		if self.optional(Symbol(Symbol::Colon)).is_some() {
			self.types()
		} else {
			Type::Primitive(TypePrim::Any).wrap()
		}
	}

	fn types(&mut self) -> TypeResult {
		self.or_type()
	}

	fn or_type(&mut self) -> TypeResult {
		let mut types = vec![self.type_primitive()?];
		while let Keyword(Keyword::Or) = self.peek().typ {
			self.next();
			types.push(self.type_primitive()?);
		}
		if types.len() == 1 {
			types.swap_remove(0)
		} else {
			Type::Or(types)
		}.wrap()
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
