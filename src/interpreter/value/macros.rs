
macro_rules! pass_msg {
	($val:expr) => {{
		let bind = $val;
		if let Value::Messenger(_) = bind {
			return bind.clone().wrap();
		} else {
			bind
		}
	}};
}

macro_rules! unwrap_msg {
	($val:expr) => {{
		let bind = $val;
		if let Value::Messenger(msg) = bind {
			return (*msg).wrap()
		} else {
			bind
		}
	}};
}

pub(crate) use pass_msg;
pub(crate) use unwrap_msg;
