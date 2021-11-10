
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
	let mut args = std::env::args().skip(1);

	if cfg!(windows) {
		ansi_term::enable_ansi_support().unwrap_or_else(|err| eprintln!("{}", err));
	}

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

	let (tokens, errors) = lexer.scan_tokens();
	errors.report(&path, "lexer");
	
	let prog = Parser::new(tokens).program().unwrap_or_else(|errors| {
		errors.report(&path, "parser");
		process::exit(1);
	});

	if !errors.is_empty() { process::exit(1); }

	Interpreter::new().interpret(prog).unwrap_or_else(|err| {
		err.report(&path, "runtime");
		process::exit(1);
	});
}

fn run_repl() {

	println!("Mars REPL");

	let mut interpreter = Interpreter::new();

	loop {
		print!("> ");
		std::io::stdout().flush().unwrap();
		let mut input = String::new();
		std::io::stdin().read_line(&mut input).unwrap();

		if input.starts_with(".exit") { break }

		let mut lexer = Lexer::from_text(&input);
		let (tokens, errors) = lexer.scan_tokens();
		if !errors.is_empty() { errors.report_repl(&input, "lexer"); continue; }
		let stmt = match Parser::new(tokens).statement() {
			Ok(stmt) => stmt,
			Err(err) => { err.report_repl(&input, "parser"); continue; }
		};
		match stmt.accept(&mut interpreter) {
			Ok(_) => (),
			Err(err) => err.report_repl(&input, "runtime")
		}

	}

}