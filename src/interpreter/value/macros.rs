
// macro_rules! throw_err {
// 	($val:expr) => {{
// 		let bind = $val;
// 		if let Value::Error(_) = bind {
// 			return bind.clone().wrap(),
// 		} else {
// 			bind
// 		}	
// 	}};
// }

// macro_rules! valerr {
// 	($res:expr) => {
// 		match $res {
// 			Ok(val) => val,
// 			Err(err) => return Value::Error(Value::Str(err.to_string()).wrap()).wrap()
// 		}
// 	};
// }

macro_rules! cast {
	(num $val:expr) => {
		match $val {
			Value::Num(n) => n,
			_ => return Value::Error(Value::Str("Value isn't a number".to_owned()).wrap()).wrap()
		}
	};
}

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

// pub(crate) use throw_err;
pub(crate) use cast;
// pub(crate) use valerr;
pub(crate) use pass_msg;
pub(crate) use unwrap_msg;
