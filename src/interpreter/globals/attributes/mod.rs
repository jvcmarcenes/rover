
pub mod string;
pub mod vector;
pub mod error;

use std::{cell::RefCell, rc::Rc};

use crate::{interpreter::{globals::attributes::{string::string, vector::vector}, value::Value}, resolver::IdentifierData};

use self::error::error;

use super::Globals;

pub const STRING_ATTR: usize = 1;
pub const VECTOR_ATTR: usize = 2;
pub const ERROR_ATTR: usize = 3;

type NatSelf = Option<Rc<RefCell<Box<dyn Value>>>>;

pub(super) fn register_default_attr(globals: &mut Globals) -> usize {
	
	let v = vec![
	("String", string()),
	("Vector", vector()),
	("Error", error()),
	];
	
	let len = v.len();
	
	let mut i = 1;
	for (key, val) in v {
		globals.ids.insert(key.to_owned(), IdentifierData::new(i, true));
		globals.values.insert(i, val);
		i += 1;
	}
	
	len
}
