
pub mod wrap;
pub mod source_pos;
pub mod result;

use std::{cell::RefCell, rc::Rc};

pub fn new_rcref<T>(x: T) -> Rc<RefCell<T>> {
	Rc::new(RefCell::new(x))
}
