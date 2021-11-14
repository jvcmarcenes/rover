
pub mod expression;
pub mod statement;

use std::{iter::Peekable, vec::IntoIter};

use crate::{lexer::token::{Token, TokenType::{self, *}, Symbol}, utils::{result::{Result, ErrorList}, wrap::Wrap}};

pub type TokenIter = Peekable<IntoIter<Token>>;

#[derive(Debug, Clone, Default)]
struct ParserContext {
	in_loop: bool,
	in_func: bool,
}

#[derive(Debug, Clone)]
pub struct Parser {
	tokens: TokenIter,
	ctx: ParserContext,
}

impl Parser {

	pub fn new(tokens: Vec<Token>) -> Self {
		Self {
			tokens: tokens.into_iter().peekable(),
			ctx: ParserContext::default(),
		}
	}

	fn peek(&mut self) -> Token {
		self.tokens.peek().expect("Parser should never reach the end of tokens").to_owned()
	}

	fn next(&mut self) -> Token {
		let peek = self.peek();
		match peek.typ {
			EOF => peek,
			_ => self.tokens.next().unwrap(),
		}
	}

	fn expect(&mut self, expected: TokenType) -> Result<Token> {
		match self.next() {
			token if token.typ == expected => token.wrap(),
			token => ErrorList::new(format!("Expected {}, found {}", expected, token.typ), token.pos).err(),
		}
	}
	
	// fn expect_any(&mut self, expected: &[TokenType]) -> Result<Token> {
	// 	match self.next() {
	// 		token if expected.contains(&token.typ) => token.wrap(),
	// 		token => {
	// 			let expected_str = expected.iter().map(|typ| typ.to_string()).reduce(|a, b| format!("{}, {}", a, b)).expect("Cannot expect no tokens");
	// 			Error::new(format!("Expected any of ({}), found {}", expected_str, token.typ), token.pos).into()
	// 		}
	// 	}
	// }

	fn expect_eol(&mut self) -> Result<()> {
		match self.peek() {
			token if token.typ == EOL => { self.next(); Ok(()) }
			token if token.typ == EOF || token.typ == Symbol(Symbol::CloseBracket) => Ok(()),
			token => ErrorList::new(format!("Expected new line, found {}", token), token.pos).err()
		}
	}

	fn optional(&mut self, expected: TokenType) -> Option<Token> {
		match self.peek() {
			token if token.typ == expected => self.next().wrap(),
			_ => None,
		}
	}

	fn optional_any(&mut self, expected: &[TokenType]) -> Option<Token> {
		match self.peek() {
			token if expected.contains(&token.typ) => self.next().wrap(),
			_ => None,
		}
	}

	fn next_match(&mut self, expected: TokenType) -> bool {
		self.peek().typ == expected
	}

	fn next_match_any(&mut self, expected: &[TokenType]) -> bool {
		expected.contains(&self.peek().typ)
	}

	fn skip_new_lines(&mut self) {
		while let EOL = self.peek().typ { self.next(); }
	}

	fn synchronize_with(&mut self, stop_at: TokenType) {
		loop {
			match self.next().typ {
				EOF => break,
				typ if typ == stop_at => break,
				_ => continue,
			}
		}
	}

	fn synchronize_with_any(&mut self, stop_at: &[TokenType]) {
		loop {
			match self.next().typ {
				EOF => break,
				typ if stop_at.contains(&typ) => break,
				_ => continue,
			}
		}
	}

	fn synchronize(&mut self) {
		loop {
			match self.next().typ {
				EOL | EOF | Symbol(Symbol::CloseBracket) | Symbol(Symbol::ClosePar) => return,
				// Symbol(Symbol::OpenBracket) => { self.synchronize_with(Symbol(Symbol::CloseBracket)); return }
				_ => continue,
			}
		}
	}

	fn is_at_end(&mut self) -> bool {
		self.peek().typ == EOF
	}

}
