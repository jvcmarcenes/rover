
pub mod token;

use std::{iter::Peekable, vec::IntoIter};

use crate::utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap};

use self::token::{Keyword, LiteralType::*, Symbol::{self, *}, Token, TokenType::*};

type TokenResult = Result<Option<Token>>;
type LexerResult = (Vec<Token>, ErrorList);

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
				None => return ErrorList::comp("Unexpected EOF".to_owned(), self.cursor).err(),
			}
		}
	}

	fn symbol(&mut self, symbol: Symbol) -> TokenResult { Token::new(Symbol(symbol), self.cursor).wrap() }

	fn scan_comment(&mut self) -> TokenResult {
		let _ = self.scan_raw_while(&mut String::new(), |c| c != '\n');
		return Ok(None);
	}

	fn scan_block_comment(&mut self) -> TokenResult {
		let pos = self.cursor;
		loop {
			match self.next_char() {
				Some('#') if self.next_match(')') => return Ok(None),
				Some(_) => continue,
				None => return ErrorList::comp("Block comment left open".to_owned(), pos).err()
			}
		}
	}

	fn scan_string(&mut self) -> TokenResult {
		let pos = self.cursor;
		let mut str = String::new();
		self.scan_raw_while(&mut str, |c| c != '"')?;
		self.next_char();
		Token::new(Literal(Str(str)), pos).wrap()
	}

	fn scan_number(&mut self, first_digit: char) -> TokenResult {
		let mut value = String::from(first_digit);
		while let Some(&c) = self.source.peek() {
			if !(c.is_ascii_digit() || (c == '.' && !value.contains('.'))) { break }
			value.push(c);
			self.next_char();
		}
		match value.parse::<f64>() {
			Ok(n) => Token::new(Literal(Num(n)), self.cursor).wrap(),
			Err(_) => ErrorList::comp(format!("Invalid number literal '{}'", value), self.cursor).err(),
		}
	}

	fn scan_identifier_or_keyword(&mut self, first_char: char) -> TokenResult {
		let pos = self.cursor;
		let mut word = String::from(first_char);
		let _ = self.scan_raw_while(&mut word, |c| c.is_ascii_alphanumeric() || c == '_');
		match Keyword::get(&word) {
			Some(keyword) => Token::new(Keyword(keyword), pos).wrap(),
			None => Token::new(Identifier(word), pos).wrap(),
		}
	}

	fn scan_str_template(&mut self) -> TokenResult {
		let mut tokens = Vec::new();
		let mut errors = ErrorList::new();

		let template_pos = self.cursor;

		loop {
			match self.next_char() {
				Some('\'') => break,
				Some('#') if self.next_match('{') => {
					tokens.push(Token::new(Symbol(HashtagOpenBracket), self.cursor));
					loop {
						match self.next_char() {
							Some('}') => break,
							Some(c) if c == '\n' => { errors.add_comp("Illegal EOL inside string template term".to_owned(), self.cursor); return errors.err() },
							Some(c) if c.is_whitespace() => continue,
							Some(c) => match self.scan_token(c) {
								Ok(Some(token)) => tokens.push(token),
								Ok(None) => continue,
								Err(err) => errors.append(err),
							}
							None => {
								errors.add_comp("Unexpected EOF".to_owned(), self.cursor);
								return errors.err();
							}
						}
					}
					tokens.push(Token::new(Symbol(CloseBracket), self.cursor))
				}
				Some(c) => {
					let text_start = self.cursor;
					let mut text = String::from(c);
					match self.scan_raw_while(&mut text, |c| c != '\'' && c != '#') {
						Ok(()) => tokens.push(Token::new(Literal(Str(text)), text_start)),
						Err(err) => errors.append(err),
					}
				}
				None => {
					errors.add_comp("Unexpected EOF".to_owned(), self.cursor);
					return errors.err();
				}
			}
		}

		tokens.push(Token::new(EOF, self.cursor));

		if errors.is_empty() {
			Token::new(Template(tokens), template_pos).wrap()
		} else {
			errors.err()
		}
	}

	fn scan_token(&mut self, first_char: char) -> TokenResult {
		match first_char {
			c if c.is_ascii_alphabetic() || c == '_' => self.scan_identifier_or_keyword(c),
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
			';' => Token::new(EOL, self.cursor).wrap(),
			':' => self.symbol(Colon),
			'+' if self.next_match('=') => self.symbol(PlusEquals),
			'+' => self.symbol(Plus),
			'-' if self.next_match('=') => self.symbol(MinusEquals),
			'-' => self.symbol(Minus),
			'*' if self.next_match('=') => self.symbol(StarEquals),
			'*' => self.symbol(Star),
			'/' if self.next_match('=') => self.symbol(SlashEquals),
			'/' => self.symbol(Slash),
			'!' if self.next_match('=') => self.symbol(ExclamEquals),
			'!' => self.symbol(Exclam),
			'=' if self.next_match('>') => self.symbol(EqualsCloseAng),
			'=' if self.next_match('=') => self.symbol(DoubleEquals),
			'=' => self.symbol(Equals),
			'|'  if self.next_match('>') => self.symbol(BarCloseAng),
			'?' => self.symbol(Question),
			'\'' => self.scan_str_template(),
			'"' => return self.scan_string(),
			'#' => self.scan_comment(),
			_ => ErrorList::comp(format!("Unknow token {}", first_char), self.cursor).err(),
		}
	}

	pub fn scan_tokens(&mut self) -> LexerResult {
		let mut tokens = Vec::new();
		let mut errors = ErrorList::new();

		loop {
			match self.next_char() {
				Some(c) if c == '\n' => tokens.push(Token::new(EOL, self.cursor)),
				Some(c) if c.is_whitespace() => continue,
				Some(c) => match self.scan_token(c) {
					Ok(Some(token)) if token.typ == Symbol(BarCloseAng) => {
						// allows function piping operator '|>' to be put in the line after an expression
						if let Some(token) = tokens.last() { if token.typ == EOL { tokens.pop(); } }
						tokens.push(token);
					},
					Ok(Some(token)) => tokens.push(token),
					Ok(None) => continue,
					Err(err) => errors.append(err),
				}
				None => break,
			}
		}

		tokens.push(Token::new(EOF, self.cursor));

		(tokens, errors)
	}

}
