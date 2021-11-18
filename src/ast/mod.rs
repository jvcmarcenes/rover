use std::{cell::RefCell, fmt::Display, rc::Rc};


pub mod expression;
pub mod statement;

#[derive(Debug, Clone)]
pub struct Identifier {
	pub name: String,
	pub id: Rc<RefCell<usize>>,
}

impl Identifier {

	pub fn new(name: String) -> Self {
		Self { name, id: Rc::new(RefCell::new(0)) }
	}

	pub fn get_name(&self) -> String {
		self.name.clone()
	}

	pub fn get_id(&self) -> usize {
		self.id.borrow().clone()
	}

}

impl Display for Identifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}
