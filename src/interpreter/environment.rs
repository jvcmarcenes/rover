
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::value::Value;

pub type ValueMap = HashMap<usize, Value>;

#[derive(Debug, Clone)]
pub struct Environment(Rc<RefCell<ValueMap>>);

impl Environment {
	
	pub fn new(globals: ValueMap) -> Self {
		Self(Rc::new(RefCell::new(globals)))
	}

	pub fn define(&mut self, key: usize, value: Value) {
		self.0.borrow_mut().insert(key, value);
	}
	
	pub fn get(&self, key: usize) -> Value {
		self.0.borrow().get(&key).expect("resolver should catch this").clone()
	}
	
	pub fn assign(&mut self, key: usize, value: Value) {
		self.0.borrow_mut().insert(key, value);
	}

}
