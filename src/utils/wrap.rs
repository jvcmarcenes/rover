
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
