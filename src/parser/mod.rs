
pub mod expression;

use std::{iter::Peekable, vec::IntoIter};

use crate::{lexer::token::{Token, TokenType}, result::{Result, Error}, utils::wrap::Wrap};

pub type TokenIter = Peekable<IntoIter<Token>>;

#[derive(Debug, Clone)]
pub struct Parser {
	tokens: TokenIter
}

impl Parser {

	pub fn new(tokens: Vec<Token>) -> Self {
		Self {
			tokens: tokens.into_iter().peekable()
		}
	}

	fn peek(&mut self) -> Token {
		self.tokens.peek().expect("Parser should never reach the end of tokens").to_owned()
	}

	fn next(&mut self) -> Token {
		let peek = self.peek();
		match peek.typ {
			TokenType::EOF => peek,
			_ => {
				self.tokens.next();
				peek
			}
		}
	}

	fn expect(&mut self, expected: TokenType) -> Result<Token> {
		match self.next() {
			token if token.typ == expected => token.wrap(),
			token => Error::new(format!("Expected {}, found {}", expected, token.typ), token.pos).into(),
		}
	}
	
	fn expect_any(&mut self, expected: &[TokenType]) -> Result<Token> {
		match self.next() {
			token if expected.contains(&token.typ) => token.wrap(),
			token => {
				let expected_str = expected.iter().map(|typ| typ.to_string()).reduce(|a, b| format!("{}, {}", a, b)).expect("Cannot expect no tokens");
				Error::new(format!("Expected any of ({}), found {}", expected_str, token.typ), token.pos).into()
			}
		}
	}

	fn expect_eol(&mut self) -> Result<()> {
		match self.peek() {
			token if token.typ == TokenType::EOL => {
				self.next();
				Ok(())
			}
			token if token.typ == TokenType::EOF => Ok(()),
			token => Error::new(format!("Expected new line, found {}", token), token.pos).into()
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

	fn skip_new_lines(&mut self) {
		loop {
			match self.peek().typ {
				TokenType::EOL => { self.next(); },
				_ => return,
			}
		}
	}

	fn synchronize(&mut self) {
		loop {
			match self.expect_eol() {
				Ok(_) => return,
				Err(_) => { self.next(); },
			}
		}
	}

}
