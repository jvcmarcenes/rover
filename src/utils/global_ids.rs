
use std::collections::HashMap;

use crate::semantics::resolver::{IdentifierData, SymbolType};

pub static GLOBAL_IDS: &[&str] = &[
	// io
	"write", "writeline", "debug", "read",

	// system / process		
	"exit", "abort",
	
	// thread		
	"sleep",
	
	// other
	"clock",
	"range",
	"typeof",
	"random", "rand",
	"char",
	"paint",
	
	// std lib	
	"math", "fs",

	// attributes
	"String", "List", "Error",
];

pub fn global_id(global: &str) -> usize {
	GLOBAL_IDS
		.iter()
		.enumerate()
		.find(|(_, &key)| key == global)
		.expect(&format!("Tried to find an undefined global '{}'", global))
		.0 + 1
}

pub fn get_global_identifiers() -> HashMap<String, IdentifierData> {
	GLOBAL_IDS
		.iter()
		.enumerate()
		.map(|(i, &key)| (key.to_owned(), IdentifierData::new(i + 1, true, SymbolType::Var)))
		.collect()
}
