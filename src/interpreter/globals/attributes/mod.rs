
use std::{rc::Rc, cell::RefCell};

use crate::interpreter::value::Value;

pub mod string;
pub mod vector;
pub mod error;

type NatSelf = Option<Rc<RefCell<Box<dyn Value>>>>;
