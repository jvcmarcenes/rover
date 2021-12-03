
use std::{cell::RefCell, fmt::Display, rc::Rc, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
	pub name: String,
	pub id: Rc<RefCell<usize>>,
}

impl Identifier {

	pub fn new(name: String) -> Self {
		Self { name, id: Rc::new(RefCell::new(usize::default())) }
	}

	pub fn get_name(&self) -> String {
		self.name.clone()
	}

	pub fn get_id(&self) -> usize {
		self.id.borrow().clone()
	}

}

impl Hash for Identifier {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.id.borrow().clone().hash(state);
	}
}

impl Display for Identifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}
