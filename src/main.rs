
#![allow(dead_code, unused_variables)]

mod utils;
mod lexer;
mod ast;
mod parser;
mod resolver;
mod interpreter;
mod bytecode;

use std::{path::Path, process};

use bytecode::{chunk::Chunk, opcode::OpCode, vm::VM};
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use resolver::Resolver;
use utils::source_pos::SourcePos;

fn main() {

	let mut chunk = Chunk::new();

	let dummy = SourcePos::new(123, 456);

	chunk.write_const(1.2, dummy);
	chunk.write_const(3.4, dummy);
	chunk.write_instr(OpCode::Add, dummy);
	chunk.write_const(5.6, dummy);
	chunk.write_instr(OpCode::Divide, dummy);
	chunk.write_instr(OpCode::Negate, dummy);
	chunk.write_instr(OpCode::Return, dummy);

	VM::new(chunk).run().unwrap();

	// let mut args = std::env::args().skip(1);

	// if cfg!(windows) {
	// 	ansi_term::enable_ansi_support().unwrap_or_else(|err| eprintln!("{}", err));
	// }

	// match args.next() {
	// 	Some(path) => run_file(&path),
	// 	None => eprintln!("{}: {}", ansi_term::Color::Red.paint("error"), "no file path specified"),
	// }
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

	Resolver::new().resolve(&ast).unwrap_or_else(|errors| {
		errors.report(&path);
		process::exit(1);
	});

	if !lexer_err.is_empty() { process::exit(1); }
	
	let mut pathbuf = Path::new(path).to_path_buf();
	pathbuf.pop();

	let mut interpreter = Interpreter::new(pathbuf);

	interpreter.interpret(&ast).unwrap_or_else(|err| {
		err.report(&path);
		process::exit(1);
	});
}
