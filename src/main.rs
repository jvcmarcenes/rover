
#![allow(dead_code, unused_variables)]

mod utils;
mod lexer;
mod ast;
mod parser;
mod resolver;
mod interpreter;
mod bytecode;

use std::{/* path::Path, */process};

use bytecode::{vm::VM, chunk_gen::ChunkGen};
// use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
// use resolver::Resolver;

fn main() {

	let mut args = std::env::args().skip(1);

	if cfg!(windows) {
		ansi_term::enable_ansi_support().unwrap_or_else(|err| eprintln!("{}", err));
	}

	match args.next() {
		Some(path) => run_file(&path),
		None => eprintln!("Usage: rover [path]"),
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

	// Resolver::new().resolve(&ast).unwrap_or_else(|errors| {
	// 	errors.report(&path);
	// 	process::exit(1);
	// });

	if !lexer_err.is_empty() { process::exit(1); }

	let chunk = ChunkGen::new().generate(ast).unwrap_or_else(|errors| {
		errors.report(&path);
		process::exit(1);
	});

	VM::new(chunk).run().unwrap_or_else(|err| {
		err.report(&path);
		process::exit(1);
	});

	// let mut pathbuf = Path::new(path).to_path_buf();
	// pathbuf.pop();

	// let mut interpreter = Interpreter::new(pathbuf);

	// interpreter.interpret(&ast).unwrap_or_else(|err| {
	// 	err.report(&path);
	// 	process::exit(1);
	// });
}
