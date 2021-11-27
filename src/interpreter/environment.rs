
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::utils::wrap::Wrap;

use super::value::Value;

pub type ValueMap = HashMap<usize, Value>;

#[derive(Debug, Clone)]
pub struct Environment(Vec<Rc<RefCell<ValueMap>>>);

impl Environment {
	
	pub fn new(globals: ValueMap) -> Self {
		Self(vec![globals.wrap(), ValueMap::new().wrap()])
	}

	pub fn push_new(&mut self) {
		self.0.push(ValueMap::new().wrap())
	}

	pub fn pop(&mut self) {
		self.0.pop();
	}

	pub fn define(&mut self, key: usize, value: Value) {
		self.0.last_mut().unwrap().borrow_mut().insert(key, value);
	}
	
	pub fn get(&self, key: usize) -> Value {
		let mut cur = self.0.as_slice();
		while let [rest @ .., top] = cur {
			if top.borrow().contains_key(&key) {
				return top.borrow().get(&key).unwrap().clone()
			}
			cur = rest;
		}
		panic!("use of unresolved variable");
	}
	
	pub fn assign(&mut self, key: usize, value: Value) {
		let mut cur = self.0.as_mut_slice();
		while let [rest @ .., top] = cur {
			if top.borrow().contains_key(&key) {
				top.borrow_mut().insert(key, value);
				return;
			}
			cur = rest;
		}
		panic!("use of unresolved variable");
	}

}
