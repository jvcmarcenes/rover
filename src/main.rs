
#![allow(dead_code)]

mod result;
mod source_pos;

use result::Error;
use source_pos::SourcePos;

fn main() {
	let mut args = std::env::args();
	args.next();

	let path = args.next().expect("Expeted a file path");

	run(&path);
}

fn run(path: &str) {
	
	let err = Error::new("Expected an expression".to_owned(), SourcePos::new(2, 11));

	err.throw_and_continue(path, "parser");

}
