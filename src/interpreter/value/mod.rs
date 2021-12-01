
use std::{cell::RefCell, fmt::{Debug, Display}, rc::Rc};

use crate::{interpreter::value::macros::castf, utils::{result::{ErrorList, Result}, source_pos::SourcePos, wrap::Wrap}};

use self::primitives::{attribute::Attribute, callable::Callable, object::ObjectMap, vector::VectorData};

use super::{Interpreter, Message};

pub mod macros;

pub mod primitives;
pub mod messenger;

pub type ValueRef = Rc<RefCell<Box<dyn Value>>>;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
	None, Num, Str, Bool, 
	Vector, Object, Callable,
	Error, Messenger,
	Attribute,
}

impl Display for ValueType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ValueType::None      => write!(f, "none"),
			ValueType::Num       => write!(f, "number"),
			ValueType::Str       => write!(f, "string"),
			ValueType::Bool      => write!(f, "bool"),
			ValueType::Vector    => write!(f, "vector"),
			ValueType::Object    => write!(f, "object"),
			ValueType::Callable  => write!(f, "callable"),
			ValueType::Error     => write!(f, "error"),
			ValueType::Messenger => write!(f, "messenger"),
			ValueType::Attribute => write!(f, "attribute"),
		}
	}
}

pub trait Value : Debug {
	
	fn get_type(&self) -> ValueType;
	
	fn to_num(&self, pos: SourcePos) -> Result<f64> { ErrorList::run("Cannot cast value to number".to_owned(), pos).err() }
	fn to_str(&self, pos: SourcePos) -> Result<String> { ErrorList::run("Cannot cast value to string".to_owned(), pos).err() }
	fn to_vector(&self, pos: SourcePos) -> Result<VectorData> { ErrorList::run("Cannot cast value to vector".to_owned(), pos).err() }
	fn to_obj(&self, pos: SourcePos) -> Result<ObjectMap> { ErrorList::run("Cannot cast value to object".to_owned(), pos).err() }
	fn to_callable(&self, pos: SourcePos) -> Result<Rc<RefCell<dyn Callable>>> { ErrorList::run("Cannot cast value to callable".to_owned(), pos).err() }
	fn to_error(&self, pos: SourcePos) -> Result<Box<dyn Value>> { ErrorList::run("Cannot cast value to error".to_owned(), pos).err() }
	fn to_attr(&self, pos: SourcePos) -> Result<Attribute> { ErrorList::run("Cannot cast value to attribute".to_owned(), pos).err() }
	
	fn to_message(&self) -> Message { panic!("Cannot cast value to messenger") }
	
	fn is_truthy(&self) -> bool { true }
	
	fn cloned(&self) -> Box<dyn Value>;
	
	fn get_attributes(&self) -> Vec<usize> { vec![] }
	
	fn get_field(&self, field: &str, interpreter: &mut Interpreter, pos: SourcePos) -> Result<ValueRef> {
		let attrs = self.get_attributes();
		let mut cur = attrs.as_slice();
		while let [ rest @ .., top ] = cur {
			let attr = interpreter.env.get(*top);
			match castf!(attr attr).get(field) {
				Some(method) => return method.wrap(),
				None => cur = rest,
			}
		}
		ErrorList::run(format!("Property {} is undefined for {}", field, self.get_type()), pos).err()
	}
	
	fn to_string(&self, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<String>;
	
	fn add(&self, other: Box<dyn Value>, _other_pos: SourcePos,  _interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> { ErrorList::run(format!("Operation ADD is not defined for {} and {}", self.get_type(), other.get_type()), pos).err() }
	fn sub(&self, other: Box<dyn Value>, _other_pos: SourcePos,  _interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> { ErrorList::run(format!("Operation SUB is not defined for {} and {}", self.get_type(), other.get_type()), pos).err() }
	fn mul(&self, other: Box<dyn Value>, _other_pos: SourcePos,  _interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> { ErrorList::run(format!("Operation MUL is not defined for {} and {}", self.get_type(), other.get_type()), pos).err() }
	fn div(&self, other: Box<dyn Value>, _other_pos: SourcePos,  _interpreter: &mut Interpreter, pos: SourcePos) -> Result<Box<dyn Value>> { ErrorList::run(format!("Operation DIV is not defined for {} and {}", self.get_type(), other.get_type()), pos).err() }
	
	fn equ(&self, other: Box<dyn Value>, other_pos: SourcePos, _interpreter: &mut Interpreter, _pos: SourcePos) -> Result<bool>;
	
	fn equals(&self, other: Box<dyn Value>, other_pos: SourcePos, interpreter: &mut Interpreter, pos: SourcePos) -> Result<bool> {
		if self.get_type() == other.get_type() {
			self.equ(other, other_pos, interpreter, pos)?
		} else {
			false
		}.wrap()
	}
	
	fn is_attr(&self, attr: usize) -> bool {
		self.get_attributes().contains(&attr)
	}

}

impl Clone for Box<dyn Value> {
	fn clone(&self) -> Self { self.cloned() }
}

impl <T : Value + 'static> Wrap<Box<dyn Value>> for T {
	fn wrap(self) -> Box<dyn Value> { Box::new(self) }
}

impl <T : Value + 'static> Wrap<Result<Box<dyn Value>>> for T {
	fn wrap(self) -> Result<Box<dyn Value>> { Ok(Box::new(self)) }
}

impl Wrap<Result<Rc<RefCell<Box<dyn Value>>>>> for Box<dyn Value> {
	fn wrap(self) -> Result<Rc<RefCell<Box<dyn Value>>>> {
		Ok(Rc::new(RefCell::new(self)))
	}
}

impl Wrap<Option<Rc<RefCell<Box<dyn Value>>>>> for Box<dyn Value> {
	fn wrap(self) -> Option<Rc<RefCell<Box<dyn Value>>>> {
		Some(Rc::new(RefCell::new(self)))
	}
}
