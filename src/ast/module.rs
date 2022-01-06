
use std::{collections::HashMap, rc::Rc, cell::RefCell};

use crate::{utils::{source_pos::SourcePos, result::{Result, ErrorList}, wrap::Wrap}};

use super::{identifier::Identifier, statement::Statement};

#[derive(Clone, Debug)]
pub struct Module {
	pub env: HashMap<Identifier, Statement>,
	pub main_id: Rc<RefCell<Option<usize>>>,
}

impl Module {

	pub fn new() -> Self {
		Self {
			env: HashMap::new(),
			main_id: None.wrap(),
		}
	}

	pub fn add(&mut self, id: Identifier, decl: Statement, pos: SourcePos) -> Result<()> {
		if self.env.keys().any(|k| k.get_name() == id.get_name()) {
			return ErrorList::comp(format!("Name '{}' is already defined on this module", id), pos).err();
		}
		self.env.insert(id, decl);
		Ok(())
	}

}
