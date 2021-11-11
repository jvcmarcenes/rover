
use std::collections::HashMap;

use crate::utils::{result::*, source_pos::SourcePos, wrap::Wrap};

use super::value::Value;

pub type ValueMap = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct Environment(Vec<ValueMap>);

impl Environment {
	
	pub fn new() -> Self {
		Self(vec![ValueMap::new()])
	}

	pub fn top(&mut self) -> &mut ValueMap {
		self.0.last_mut().expect("Environment should never be empty")
	}

	pub fn push(&mut self, map: ValueMap) {
		self.0.push(map);
	}

	pub fn pop(&mut self) {
		match self.0.as_slice() {
			[_, ..] => self.0.pop(),
			[] => panic!("Tried to pop the root environment"),
		};
	}

	pub fn define(&mut self, name: String, value: Value) {
		self.top().insert(name, value);
	}
	
	pub fn get(&self, name: &str, pos: SourcePos) -> Result<Value> {
		let mut cur = self.0.as_slice();
		while let [env, rest @ ..] = cur {
			match env.get(name) {
				Some(val) => return val.to_owned().wrap(),
				None => cur = rest,
			}
		}
		ErrorList::new(format!("Undefined variable '{}'", name), pos).err()
	}
	
	pub fn assign(&mut self, name: String, value: Value, pos: SourcePos) -> Result<()> {
		let mut cur = self.0.as_mut_slice();
		while let [env, rest @ ..] = cur {
			if env.contains_key(&name) {
				env.insert(name, value);
				return Ok(())
			}
			cur = rest;
		}
		ErrorList::new(format!("Undefined variable '{}'", name), pos).err()
	}

}