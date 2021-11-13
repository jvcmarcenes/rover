
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::utils::{result::*, source_pos::SourcePos, wrap::Wrap};

use super::value::Value;

pub type ValueMap = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct Environment(Vec<Rc<RefCell<ValueMap>>>);

impl Environment {
	
	pub fn new() -> Self {
		Self(vec![Rc::new(RefCell::new(HashMap::new()))])
	}

	pub fn top(&mut self) -> &Rc<RefCell<ValueMap>> {
		self.0.last().expect("Environment should never be empty")
	}

	pub fn push(&mut self, map: ValueMap) {
		self.0.push(Rc::new(RefCell::new(map)));
	}

	pub fn pop(&mut self) {
		match self.0.as_slice() {
			[_, _, ..] => self.0.pop(),
			[_] => panic!("Tried to pop the root environment"),
			[] => panic!("Environment should never be empty"),
		};
	}

	pub fn definef(&mut self, name: &str, value: Value) {
		self.top().borrow_mut().insert(name.to_owned(), value);
	}

	pub fn define(&mut self, name: &str, value: Value, pos: SourcePos) -> Result<()> {
		if self.0.first().unwrap().borrow().contains_key(name) {
			return ErrorList::new(format!("Cannot redefine '{}'", name), pos).err()
		}
		self.top().borrow_mut().insert(name.to_owned(), value);
		Ok(())
	}
	
	pub fn get(&self, name: &str, pos: SourcePos) -> Result<Value> {
		let mut cur = self.0.as_slice();
		while let [rest @ .., env] = cur {
			match env.borrow().get(name) {
				Some(val) => return val.to_owned().wrap(),
				None => cur = rest,
			}
		}
		ErrorList::new(format!("Undefined variable '{}'", name), pos).err()
	}
	
	pub fn assign(&mut self, name: &str, value: Value, pos: SourcePos) -> Result<()> {
		if self.0.first().unwrap().borrow().contains_key(name) {
			return ErrorList::new(format!("Cannot assign to '{}'", name), pos).err()
		}
		let mut cur = self.0.as_mut_slice();
		while let [_, rest @ .., env] = cur {
			if env.borrow().contains_key(name) {
				env.borrow_mut().insert(name.to_owned(), value);
				return Ok(())
			}
			cur = rest;
		}
		ErrorList::new(format!("Undefined variable '{}'", name), pos).err()
	}

}