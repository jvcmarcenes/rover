
#![allow(dead_code)]

mod utils;
mod lexer;
mod ast;
mod parser;
mod types;
mod semantics;
mod interpreter;

use std::{path::Path, process};

use interpreter::Interpreter;
use lexer::{Lexer, LexerResult};
use parser::Parser;
use semantics::{resolver::Resolver, optimizer::Optimizer, type_checker::TypeChecker};
use utils::result::{Result, ErrorList};

fn main() {
	let mut args = std::env::args().skip(1);

	if cfg!(windows) {
		ansi_term::enable_ansi_support().unwrap_or_else(|err| eprintln!("{}", ansi_term::Color::Red.paint(format!("[os error {}]", err))));
	}

	match args.next() {
		Some(path) => run_file(&path).unwrap_or_else(|errors| { errors.report(&path); process::exit(1) }),
		None => eprintln!("{}: {}", ansi_term::Color::Red.paint("error"), "No file path specified"),
	}
}

fn run_file(path: &str) -> Result<()> {
	let lexer = Lexer::from_file(&path).map_err(|err| ErrorList::sys(err.to_string()))?;

	let lexer_res = lexer.scan_tokens();
	lexer_res.errors.report(path);

	if lexer_res.directives.script {
		run_script(path, lexer_res)
	} else {
		run_module(path, lexer_res)
	}
}

fn run_module(path: &str, lexer_res: LexerResult) -> Result<()> {
	let LexerResult { tokens, directives, errors: lexer_err } = lexer_res;
	let mut errors = ErrorList::new();
	
	let mut module = Parser::new(tokens).module()?;

	errors.try_append(Resolver::new().resolve(&module));
	if module.main_id.borrow().is_none() {
		errors.add_mod_comp("Module did not contain a main function".to_owned());
	}
	errors.if_empty(())?;
	
	errors.try_append(TypeChecker::new(!directives.dynamic).check(&module));
	errors.if_empty(())?;

	lexer_err.if_empty(())?;
	
	Optimizer.optimize(&mut module).unwrap();
	
	let mut pathbuf = Path::new(path).to_path_buf();
	pathbuf.pop();
	
	let mut interpreter = Interpreter::new(pathbuf);
	
	interpreter.interpret_and_run(module)?;
	
	Ok(())
}

fn run_script(path: &str, lexer_res: LexerResult) -> Result<()> {
	let LexerResult { tokens, directives, errors: lexer_err } = lexer_res;
	let mut errors = ErrorList::new();
	
	let (mut module, block) = Parser::new(tokens).script()?;

	let mut resolver = Resolver::new();
	errors.try_append(resolver.resolve(&module));
	errors.try_append(resolver.resolve_block(&block));
	errors.if_empty(())?;

	let mut type_checker = TypeChecker::new(!directives.dynamic);
	errors.try_append(type_checker.check(&module));
	errors.try_append(type_checker.check_block(&block));
	errors.if_empty(())?;

	lexer_err.if_empty(())?;

	Optimizer.optimize(&mut module).unwrap();
	let block = Optimizer.optimize_block(block).unwrap();

	let mut pathbuf = Path::new(path).to_path_buf();
	pathbuf.pop();

	let mut interpreter = Interpreter::new(pathbuf);

	interpreter.interpret_script(module, block)?;

	Ok(())
}
