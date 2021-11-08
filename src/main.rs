
#![allow(dead_code)]

mod utils;
mod lexer;
mod ast;
mod parser;
mod interpreter;

use std::{io::Write, process};

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn main() {
	let mut args = std::env::args();
	args.next();

	match args.next() {
		Some(path) => run_file(&path),
		None => run_repl(),
	}

}

fn run_file(path: &str) {
	let mut lexer = Lexer::from_file(&path).unwrap_or_else(|err| {
		eprintln!("{}: {}", ansi_term::Color::Red.paint("system error"), err);
		process::exit(1);
	});

	let tokens = lexer.scan_tokens().unwrap_or_else(|errors| {
		for err in errors { err.report(&path, "lexer"); }
		process::exit(1);
	});

	let mut parser = Parser::new(tokens);

	let expr = parser.expression().unwrap_or_else(|err| {
		err.report(&path, "parser"); 
		process::exit(1);
	});

	match Interpreter.evaluate(Box::new(expr)) {
		Ok(val) => println!("{}", val),
		Err(err) => err.report(&path, "runtime"),
	}
}

fn run_repl() {

	println!("Mars REPL");

	let mut interpreter = Interpreter;

	loop {
		print!("> ");
		std::io::stdout().flush().unwrap();
		let mut input = String::new();
		std::io::stdin().read_line(&mut input).unwrap();

		if input.starts_with(".exit") { break }

		let mut lexer = Lexer::from_text(&input);
		let tokens = match lexer.scan_tokens() {
			Ok(tokens) => tokens,
			Err(errors) => {
				for err in errors { err.repl_err(&input, "lexer") }
				continue;
			}
		};
		let mut parser = Parser::new(tokens);
		let expr = match parser.expression() {
			Ok(expr) => expr,
			Err(err) => { err.repl_err(&input, "parser"); continue; }
		};
		match interpreter.evaluate(Box::new(expr)) {
			Ok(val) => println!("{}", val),
			Err(err) => err.repl_err(&input, "runtime")
		}

	}

}