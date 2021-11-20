
use std::fmt::Display;

use crate::utils::source_pos::SourcePos;

use self::{Keyword::*, TokenType::*};

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
	Str(String),
	Num(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
	True, False, _None,
	Let,
	Mod, And, Or,
	If, Else,
	Loop, Break, Continue,
	Function, Return,
	_Self,
}

impl Keyword {
	pub fn get(s: &str) -> Option<Keyword> {
		let keyword = match s {
			"true" => True,
			"false" => False,
			"none" => _None,
			"let" => Let,
			"mod" => Mod,
			"and" => And,
			"or" => Or,
			"if" => If,
			"else" => Else,
			"loop" => Loop,
			"break" => Break,
			"continue" => Continue,
			"function" => Function,
			"return" => Return,
			"self" => _Self,
			_ => return None,
		};
		Some(keyword)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
	OpenPar, ClosePar, OpenSqr, CloseSqr, OpenBracket, CloseBracket, OpenAng, CloseAng,
	Dot, Comma, Colon,
	Plus, Minus, Star, Slash, Exclam,
	Equals, PlusEquals, MinusEquals,
	DoubleEquals, ExclamEquals, OpenAngEquals, CloseAngEquals,
	HashtagOpenBracket, EqualsCloseAng,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
	Literal(LiteralType),
	Keyword(Keyword),
	Symbol(Symbol),
	Identifier(String),
	Template(Vec<Token>),
	EOL, EOF,
}

impl Display for TokenType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Literal(lit) => match lit {
				LiteralType::Str(s) => write!(f, "\"{}\"", s),
				LiteralType::Num(n) => write!(f, "{}", n),
			},
			Keyword(keyword) => write!(f, "{:?}", keyword),
			Symbol(symbol) => write!(f, "{:?}", symbol),
			Identifier(name) => write!(f, "{}", name),
			Template(_) => write!(f, "str_template"),
			EOL => write!(f, "EOL"),
			EOF => write!(f, "EOF"),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
	pub typ: TokenType,
	pub pos: SourcePos,
}

impl Token {
	pub fn new(typ: TokenType, pos: SourcePos) -> Self {
		Self { typ, pos }
	}
}

impl Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.typ)
	}
}