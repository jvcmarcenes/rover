
use std::collections::HashMap;

use crate::utils::{global_ids::global_id, wrap::Wrap};

use super::Type;

pub fn global_types() -> HashMap<usize, Type> {

	let f = vec![
		("write", (vec![Type::Any], Type::Void)),
		("writeline", (vec![Type::Any], Type::Void)),
		("debug", (vec![Type::Any], Type::Void)),
		("read", (vec![], Type::STR)),

		("exit", (vec![], Type::Void)),
		("abort", (vec![Type::Any], Type::Void)),

		("clock", (vec![], Type::NUM)),
		("range", (vec![Type::NUM, Type::NUM], Type::List(Type::NUM.wrap()))),
		("random", (vec![Type::NUM], Type::Any)),
		("rand", (vec![], Type::NUM)),
	];
		
	let mut v = vec![
		("char", Type::Any),
		("paint", Type::Any),

		("math", Type::Any),
		("fs", Type::Any),

		("String", Type::Any),
		("List", Type::Any),
		("Error", Type::Any),
	];

	v.append(&mut f.into_iter().map(|(k, (params, returns))| (k, Type::Function { params, returns: returns.wrap() })).collect());

	v.into_iter().map(|(k, t)| (global_id(k), t)).collect()
}
