
#![allow(dead_code)]

mod utils;
mod result;
mod source_pos;
mod lexer;

use std::process;

use lexer::Lexer;

fn main() {
	let mut args = std::env::args();
	args.next();

	let path = args.next().expect("Expeted a file path");

	let mut lexer = match Lexer::from_file(&path) {
		Ok(lexer) => lexer,
		Err(e) => {
			eprintln!("{}: {}", ansi_term::Color::Red.paint("system error"), e);
			process::exit(1);
		}
	};

	let tokens = match lexer.scan_tokens() {
		Ok(tokens) => tokens,
		Err(errors) => {
			for error in errors {
				error.report(&path, "lexer");
			}
			process::exit(1);
		}
	};

	for token in tokens {
		println!("{}", token);
	}
	
}
