
pub mod expression;
pub mod statement;

use std::{iter::Peekable, vec::IntoIter};

use crate::{lexer::token::{Token, TokenType::{self, *}, Symbol}, utils::{result::{Result, ErrorList, append}, wrap::Wrap}, ast::{Block, module::Module}};

#[derive(Debug, Clone)]
pub struct Parser {
	tokens: Peekable<IntoIter<Token>>,
	module: Module,
	script: bool,
}

impl Parser {
	
	pub fn new(tokens: Vec<Token>, script: bool) -> Self {
		Self {
			tokens: tokens.into_iter().peekable(),
			module: Module::new(),
			script,
		}
	}
	
	pub fn module(mut self) -> Result<Module> {
		let mut errors = ErrorList::new();
		
		loop {
			self.skip_new_lines();
			if self.is_at_end() { break; }
			if let Err(err) = self.top_declaration() {
				errors.append(err);
				self.synchronize();
			}
		}
		
		errors.if_empty(self.module)
	}
	
	pub(super) fn block(&mut self) -> Result<Block> {
		self.skip_new_lines();
		let Token { pos, .. } = self.expect(Symbol(Symbol::OpenBracket))?;
		
		let mut block = Block::new();
		let mut errors = ErrorList::new();
		
		loop {
			self.skip_new_lines();
			match self.peek().typ {
				Symbol(Symbol::CloseBracket) => break,
				EOF => append!(ret comp "Statement block not closed".to_owned(), pos; to errors),
				_ => match self.statement() {
					Ok(stmt) => block.push(stmt),
					Err(err) => {
						errors.append(err);
						self.synchronize();
					}
				},
			}
		}
		
		self.next();
		errors.if_empty(block)
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
			token => ErrorList::comp(format!("Expected {}, found {}", expected, token.typ), token.pos).err(),
		}
	}
	
	fn expect_or_sync(&mut self, expected: TokenType) -> Result<Token> {
		match self.next() {
			token if token.typ == expected => token.wrap(),
			token => {
				self.synchronize();
				ErrorList::comp(format!("Expected {}, found {}", expected, token.typ), token.pos).err()
			}
		}
	}
	
	fn expect_any(&mut self, expected: &[TokenType]) -> Result<Token> {
		match self.next() {
			token if expected.contains(&token.typ) => token.wrap(),
			token => {
				let expected_str = expected.iter().map(|typ| typ.to_string()).reduce(|a, b| format!("{}, {}", a, b)).expect("Cannot expect no tokens");
				ErrorList::comp(format!("Expected any of ({}), found {}", expected_str, token.typ), token.pos).err()
			}
		}
	}
	
	fn expect_any_or_sync(&mut self, expected: &[TokenType]) -> Result<Token> {
		match self.next() {
			token if expected.contains(&token.typ) => token.wrap(),
			token => {
				self.synchronize();
				let expected_str = expected.iter().map(|typ| typ.to_string()).reduce(|a, b| format!("{}, {}", a, b)).expect("Cannot expect no tokens");
				ErrorList::comp(format!("Expected any of ({}), found {}", expected_str, token.typ), token.pos).err()
			}
		}
	}
	
	fn expect_eol(&mut self) -> Result<()> {
		match self.peek() {
			token if token.typ == EOL => { self.next(); Ok(()) }
			token if token.typ == EOF || token.typ == Symbol(Symbol::CloseBracket) => Ok(()),
			token => ErrorList::comp(format!("Expected new line, found {}", token), token.pos).err()
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
	
	// fn next_match_any(&mut self, expected: &[TokenType]) -> bool {
		// 	expected.contains(&self.peek().typ)
		// }
		
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
		
		fn synchronize_until(&mut self, stop_at: TokenType) {
			loop {
				match self.peek().typ {
					EOF => break,
					typ if typ == stop_at => break,
					_ => { self.next(); },
				}
			}
		}
		
		// fn synchronize_with_any(&mut self, stop_at: &[TokenType]) {
			// 	loop {
				// 		match self.next().typ {
					// 			EOF => break,
					// 			typ if stop_at.contains(&typ) => break,
					// 			_ => continue,
					// 		}
					// 	}
					// }
					
					fn synchronize_until_any(&mut self, stop_at: &[TokenType]) {
						loop {
							match self.peek().typ {
								EOF => break,
								typ if stop_at.contains(&typ) => break,
								_ => { self.next(); },
							}
						}
					}
					
					fn synchronize_complex(&mut self, stop_at_and_consume: &[TokenType], stop_at: &[TokenType]) {
						loop {
							match self.peek().typ {
								EOF => break,
								typ if stop_at_and_consume.contains(&typ) => { self.next(); break },
								typ if stop_at.contains(&typ) => break,
								_ => { self.next(); },
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
				