
mod utils;
mod lexer;
mod ast;
mod parser;
mod resolver;
mod environment;
mod interpreter;
mod bytecode;

use std::process;
use std::path::Path;

use utils::result::Result;

use ast::statement::Block;
use lexer::Lexer;
use parser::Parser;
use resolver::Resolver;
use interpreter::Interpreter;
use bytecode::{vm::VM, chunk_gen::ChunkGen};

fn main() {
	
	let mut args = std::env::args().skip(1);
	
	if cfg!(windows) {
		ansi_term::enable_ansi_support().unwrap_or_else(|err| eprintln!("{}", err));
	}
	
	if let Some(path) = args.next() {
		run_file(&path).unwrap_or_else(|errors| {
			errors.report(&path);
			process::exit(1);
		})
	} else {
		eprintln!("Usage: rover [path]")
	}
	
}

fn run_file(path: &str) -> Result<()> {
	let mut lexer = Lexer::from_file(&path).unwrap_or_else(|err| {
		eprintln!("{}: {}", ansi_term::Color::Red.paint("system error"), err);
		process::exit(1);
	});
	
	let (tokens, lexer_err) = lexer.scan_tokens();
	lexer_err.report(&path);
	
	let ast = Parser::new(tokens).program()?;
	
	Resolver::new().resolve(&ast)?;
	
	if !lexer_err.is_empty() { process::exit(1); }
	
	if cfg!(feature = "ast_interpreter") {
		run_ast(ast, path)?;
	} else {
		run_bytecode(ast)?;
	}

	Ok(())
}

fn run_ast(ast: Block, path: &str) -> Result<()> {
	let mut pathbuf = Path::new(path).to_path_buf();
	pathbuf.pop();
	
	Interpreter::new(pathbuf).interpret(&ast)?;
	Ok(())
}

fn run_bytecode(ast: Block) -> Result<()> {
	let chunk = ChunkGen::new().generate(ast)?;
	VM::new(chunk).run()?;
	Ok(())
}
