
#![allow(dead_code)]

mod utils;
mod lexer;
mod ast;
mod parser;
mod resolver;
mod interpreter;

use std::process;

use interpreter::{Interpreter, globals::Globals};
use lexer::Lexer;
use parser::Parser;
use resolver::Resolver;

fn main() {
	let mut args = std::env::args().skip(1);

	if cfg!(windows) {
		ansi_term::enable_ansi_support().unwrap_or_else(|err| eprintln!("{}", err));
	}

	match args.next() {
		Some(path) => run_file(&path),
		// None => run_repl(),
		None => eprintln!("{}: {}", ansi_term::Color::Red.paint("error"), "no file path given"),
	}
}

fn run_file(path: &str) {
	let mut lexer = Lexer::from_file(&path).unwrap_or_else(|err| {
		eprintln!("{}: {}", ansi_term::Color::Red.paint("system error"), err);
		process::exit(1);
	});

	let (tokens, lexer_err) = lexer.scan_tokens();
	lexer_err.report(&path);

	let ast = Parser::new(tokens).program().unwrap_or_else(|errors| {
		errors.report(&path);
		process::exit(1);
	});

	let globals = Globals::new();
	
	Resolver::new(globals.clone()).resolve(&ast).unwrap_or_else(|errors| {
		errors.report(&path);
		process::exit(1);
	});

	if !lexer_err.is_empty() { process::exit(1); }
	
	let mut interpreter = Interpreter::new(globals.values);

	interpreter.interpret(&ast).unwrap_or_else(|err| {
		err.report(&path);
		process::exit(1);
	});
}

// fn run_repl() {

// 	println!("Mars REPL");

// 	let mut interpreter = Interpreter::new();

// 	loop {
// 		print!("> ");
// 		std::io::stdout().flush().unwrap();
// 		let mut input = String::new();
// 		std::io::stdin().read_line(&mut input).unwrap();

// 		if input.starts_with(".exit") { break }

// 		let mut lexer = Lexer::from_text(&input);
// 		let (tokens, errors) = lexer.scan_tokens();
// 		if !errors.is_empty() { errors.report_repl(&input); continue; }
// 		let stmt = match Parser::new(tokens).statement() {
// 			Ok(stmt) => stmt,
// 			Err(err) => { err.report_repl(&input); continue; }
// 		};
// 		let res = match stmt.typ {
// 			StmtType::Expr(expr) => expr.accept(&mut interpreter).map(|ok| ok.to_string()),
// 			_ => stmt.accept(&mut interpreter).map(|_| "none".to_owned()),
// 		};
// 		match res {
// 			Ok(s) => println!("{}", s),
// 			Err(err) => err.report_repl(&input)
// 		}

// 	}

// }