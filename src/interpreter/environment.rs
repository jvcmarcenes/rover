
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::utils::new_rcref;

use super::value::Value;

pub type ValueMap = HashMap<usize, Value>;

#[derive(Debug, Clone)]
pub struct Environment(Vec<Rc<RefCell<ValueMap>>>);

impl Environment {
	
	pub fn new(globals: ValueMap) -> Self {
		Self(vec![new_rcref(globals), new_rcref(ValueMap::new())])
	}

	pub fn push_new(&mut self) {
		self.0.push(new_rcref(ValueMap::new()))
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
		panic!("resolver should catch this");
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
		panic!("resolver should catch this");
	}

}
