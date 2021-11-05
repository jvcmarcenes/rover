
pub mod token;

use std::{iter::Peekable, vec::IntoIter};

use crate::{result::{Error, Result}, source_pos::SourcePos, utils::Wrap};

use self::token::{Keyword, LiteralType::*, Symbol::{self, *}, Token, TokenType::*};

#[derive(Debug, Clone)]
pub struct Lexer {
	source: Peekable<IntoIter<char>>,
	cursor: SourcePos,
	next_cursor: SourcePos,
}

impl Lexer {

	pub fn from_text(text: &str) -> Self {
		Self {
			source: text.chars().collect::<Vec<_>>().into_iter().peekable(),
			cursor: SourcePos::new(1, 1),
			next_cursor: SourcePos::new(1, 1)
		}
	}

	pub fn from_file(path: &str) -> std::io::Result<Self> {
		let text = std::fs::read_to_string(path)?;
		Ok(Self::from_text(&text))
	}

	pub fn set_cursor(&mut self, new_cursor: SourcePos) {
		self.cursor = new_cursor;
	}

	fn next_char(&mut self) -> Option<char> {
		let next = self.source.next();
		match next {
			Some('\n') => {
				self.cursor = self.next_cursor;
				self.next_cursor.lin += 1;
				self.next_cursor.col = 1;
			}
			Some(_) => {
				self.cursor = self.next_cursor;
				self.next_cursor.col += 1;
			}
			None => (),
		}
		next
	}

	fn next_match(&mut self, expected: char) -> bool {
		match self.source.peek() {
			Some(&c) if c == expected => {
				self.next_char();
				true
			}
			_ => false
		}
	}

	fn scan_raw_while(&mut self, buf: &mut String, pred: fn(char) -> bool) -> Result<()> {
		loop {
			match self.source.peek() {
				Some(&c) if pred(c) => {
					buf.push(c);
					self.next_char();
				}
				Some(_) => return Ok(()),
				None => return Error::new("Unexpected EOF".to_owned(), self.cursor).into(),
			}
		}
	}

	fn symbol(&mut self, symbol: Symbol) -> Result<Option<Token>> { Token::new(Symbol(symbol), self.cursor).wrap() }

	fn scan_comment(&mut self) -> Result<Option<Token>> {
		let _ = self.scan_raw_while(&mut String::new(), |c| c != '\n');
		return Ok(None);
	}

	fn scan_block_comment(&mut self) -> Result<Option<Token>> {
		let pos = self.cursor;
		loop {
			match self.next_char() {
				Some('#') if self.next_match(')') => return Ok(None),
				Some(_) => continue,
				None => return Error::new("Block comment left open".to_owned(), pos).into()
			}
		}
	}

	fn scan_string(&mut self) -> Result<Option<Token>> {
		let pos = self.cursor;
		let mut str = String::new();
		self.scan_raw_while(&mut str, |c| c != '"')?;
		self.next_char();
		Token::new(Literal(Str(str)), pos).wrap()
	}

	fn scan_number(&mut self, first_digit: char) -> Result<Option<Token>> {
		let mut value = String::from(first_digit);
		loop {
			match self.source.peek() {
				Some(&c) if c.is_ascii_digit() || (c == '.' && !value.contains('.')) => {
					value.push(c);
					self.next_char();
				}
				_ => break,
			}
		}
		match value.parse::<f64>() {
			Ok(n) => Token::new(Literal(Num(n)), self.cursor).wrap(),
			Err(_) => Error::new(format!("Invalid number literal '{}'", value), self.cursor).into(),
		}
	}

	fn scan_identifier_or_keyword(&mut self, first_char: char) -> Result<Option<Token>> {
		let mut word = String::from(first_char);
		let _ = self.scan_raw_while(&mut word, |c| c.is_ascii_alphanumeric() || c == '_');
		match Keyword::get(&word) {
			Some(keyword) => Token::new(Keyword(keyword), self.cursor).wrap(),
			None => Token::new(Identifier(word), self.cursor).wrap(),
		}
	}

	fn scan_token(&mut self, first_char: char) -> Result<Option<Token>> {
		match first_char {
			c if c.is_ascii_alphabetic() => self.scan_identifier_or_keyword(c),
			c if c.is_ascii_digit() => self.scan_number(c),
			'(' if self.next_match('#') => self.scan_block_comment(),
			'(' => self.symbol(OpenPar),
			')' => self.symbol(ClosePar),
			'[' => self.symbol(OpenSqr),
			']' => self.symbol(CloseSqr),
			'{' => self.symbol(OpenBracket),
			'}' => self.symbol(CloseBracket),
			'<' if self.next_match('=') => self.symbol(OpenAngEquals),
			'<' => self.symbol(OpenAng),
			'>' if self.next_match('=') => self.symbol(CloseAngEquals),
			'>' => self.symbol(CloseAng),
			'.' => self.symbol(Dot),
			',' => self.symbol(Comma),
			';' => self.symbol(SemiColon),
			':' => self.symbol(Colon),
			'+' if self.next_match('=') => self.symbol(PlusEquals),
			'+' => self.symbol(Plus),
			'-' if self.next_match('=') => self.symbol(MinusEquals),
			'-' => self.symbol(Minus),
			'*' => self.symbol(Star),
			'/' => self.symbol(Slash),
			'!' if self.next_match('=') => self.symbol(ExclamEquals),
			'!' => self.symbol(Exclam),
			'=' if self.next_match('=') => self.symbol(DoubleEquals),
			'=' => self.symbol(Equals),
			'\'' => self.symbol(SingleQuote),
			'"' => return self.scan_string(),
			'#' if self.next_match('{') => self.symbol(HashtagOpenBracket),
			'#' => self.scan_comment(),
			_ => Error::new(format!("Unknow token {}", first_char), self.cursor).into(),
		}
	}

	pub fn scan_tokens(&mut self) -> std::result::Result<Vec<Token>, Vec<Error>> {
		let mut tokens = Vec::new();
		let mut errors = Vec::new();

		loop {
			match self.next_char() {
				Some(c) if c == '\n' => tokens.push(Token::new(EOL, self.cursor)),
				Some(c) if c.is_whitespace() => continue,
				Some(c) => {
					match self.scan_token(c) {
						Ok(Some(token)) => tokens.push(token),
						Ok(None) => continue,
						Err(err) => errors.push(err),
					}
				}
				None => break,
			}
		}

		tokens.push(Token::new(EOF, self.cursor));

		if errors.is_empty() {
			Ok(tokens)
		} else {
			Err(errors)
		}
	}

}
