
use std::collections::HashMap;

use crate::utils::{result::*, source_pos::SourcePos, wrap::Wrap};

use super::value::Value;

#[derive(Debug, Clone)]
pub struct Environment {
	values: HashMap<String, Value>,
	parent: Option<Box<Environment>>,
}

impl Environment {
	
	pub fn new() -> Self {
		Self {
			values: HashMap::new(),
			parent: None,
		}
	}

	pub fn with_parent(parent: Environment) -> Self {
		Self {
			values: HashMap::new(),
			parent: Box::new(parent).wrap(),
		}
	}

	pub fn define(&mut self, name: String, value: Value) {
		self.values.insert(name, value);
	}

	pub fn get(&self, name: &str, pos: SourcePos) -> Result<Value> {
		match self.values.get(name) {
			Some(val) => val.to_owned().wrap(),
			None => match self.parent {
				Some(ref env) => env.get(name, pos),
				None => ErrorList::new(format!("Undefined variable '{}'", name), pos).err()
			}
		}
	}

	pub fn assign(&mut self, name: String, value: Value, pos: SourcePos) -> Result<()> {
		if !self.values.contains_key(&name) {
			return match self.parent {
				Some(ref mut env) => env.assign(name, value, pos),
				None => ErrorList::new(format!("Undefined variable '{}'", name), pos).err()
			}
		}
		self.values.insert(name, value);
		Ok(())
	}

}