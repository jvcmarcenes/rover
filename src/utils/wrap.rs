
use std::{cell::RefCell, rc::Rc};

pub trait Wrap<T> {
	fn wrap(self) -> T;
}

impl<T> Wrap<Option<T>> for T {
	fn wrap(self) -> Option<T> { Some(self) }
}

impl<T, E> Wrap<Result<T, E>> for T {
	fn wrap(self) -> Result<T, E> { Ok(self) }
}

impl<T, E> Wrap<Result<Option<T>, E>> for T {
	fn wrap(self) -> Result<Option<T>, E> { Ok(Some(self)) }
}

impl<T> Wrap<Rc<RefCell<T>>> for T {
	fn wrap(self) -> Rc<RefCell<T>> { Rc::new(RefCell::new(self)) }
}

impl<T> Wrap<Box<T>> for T {
	fn wrap(self) -> Box<T> { Box::new(self) }
}
