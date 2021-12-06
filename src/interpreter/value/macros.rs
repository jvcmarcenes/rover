
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
	(num $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Num => bind.to_num(SourcePos::new(0, 0)).unwrap(),
			_ => return crate::interpreter::value::primitives::error::Error::new(crate::interpreter::value::primitives::string::Str::from("Cannot cast value to number")).wrap()
		}
	}};
	(str $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Str => bind.to_str(SourcePos::new(0, 0)).unwrap(),
			_ => return crate::interpreter::value::primitives::error::Error::new(crate::interpreter::value::primitives::string::Str::from("Cannot cast value to string")).wrap()
		}
	}};
	(vec $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Vector => bind.to_vector(SourcePos::new(0, 0)).unwrap(),
			_ => return crate::interpreter::value::primitives::error::Error::new(crate::interpreter::value::primitives::string::Str::from("Cannot cast value to vector")).wrap()
		}
	}};
	(obj $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Object => bind.to_obj(SourcePos::new(0, 0)).unwrap(),
			_ => return crate::interpreter::value::primitives::error::Error::new(crate::interpreter::value::primitives::string::Str::from("Cannot cast value to object")).wrap()
		}
	}};
	(err $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Error => bind.to_error(SourcePos::new(0, 0)).unwrap(),
			_ => return crate::interpreter::value::primitives::error::Error::new(crate::interpreter::value::primitives::string::Str::from("Cannot cast value to error")).wrap()
		}
	}};
}

macro_rules! castf {
	(num $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Num => bind.to_num(SourcePos::new(0, 0)).unwrap(),
			_ => panic!("Cannot cast value to number")
		}
	}};
	(str $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Str => bind.to_str(SourcePos::new(0, 0)).unwrap(),
			_ => panic!("Cannot cast value to string")
		}
	}};
	(vec $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Vector => bind.to_list(SourcePos::new(0, 0)).unwrap(),
			_ => panic!("Cannot cast value to vector")
		}
	}};
	(obj $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Object => bind.to_obj(SourcePos::new(0, 0)).unwrap(),
			_ => panic!("Cannot cast value to object")
		}
	}};
	(err $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Error => bind.to_error(SourcePos::new(0, 0)).unwrap(),
			_ => panic!("Cannot cast value to error")
		}
	}};
	(attr $val:expr) => {{
		let bind = $val;
		match bind.get_type() {
			crate::interpreter::value::ValueType::Attribute => bind.to_attr(SourcePos::new(0, 0)).unwrap(),
			_ => panic!("Cannot cast value to attribute")
		}
	}};
}

macro_rules! pass_msg {
	($val:expr) => {{
		let bind = $val;
		if let crate::interpreter::value::ValueType::Messenger = bind.get_type() {
			return bind.clone().wrap();
		} else {
			bind
		}
	}};
}

macro_rules! unwrap_msg {
	($val:expr) => {{
		let bind = $val;
		if let crate::interpreter::value::ValueType::Messenger = bind.get_type() {
			return bind.to_message().wrap();
		} else {
			bind
		}
	}};
}

// pub(crate) use throw_err;
pub(crate) use cast;
pub(crate) use castf;
// pub(crate) use valerr;
pub(crate) use pass_msg;
pub(crate) use unwrap_msg;
