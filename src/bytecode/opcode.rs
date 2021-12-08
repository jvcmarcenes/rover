
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::utils::source_pos::SourcePos;

use self::OpCode::*;

#[derive(Copy, PartialEq, Eq, Debug, Clone, FromPrimitive)]
pub enum OpCode {
	Return,
	Const, Const16, ConstFalse, ConstTrue, ConstNone,
	StrTemplate,
	
	Negate, Identity, Not,
	Add, Subtract, Multiply, Divide, Remainder,
	Equals, NotEquals, Greater, Lesser, GreaterEq, LesserEq,
}

impl From<u8> for OpCode {
	fn from(code: u8) -> Self {
		OpCode::from_u8(code).expect(&format!("Unknown opcode {}", code))
	}
}

impl OpCode {
	pub fn accept<T>(&self, visitor: &mut dyn OpCodeVisitor<T>, pos: SourcePos) -> T {
		match self {
			Return     => visitor.op_return(pos),
			Const      => visitor.op_const(pos),
			Const16    => visitor.op_const_16(pos),
			ConstFalse => visitor.op_false(pos),
			ConstTrue  => visitor.op_true(pos),
			ConstNone  => visitor.op_none(pos),
			StrTemplate => visitor.op_template(pos),
			Negate     => visitor.op_negate(pos),
			Identity   => visitor.op_identity(pos),
			Not        => visitor.op_not(pos),
			Add        => visitor.op_add(pos),
			Subtract   => visitor.op_subtract(pos),
			Multiply   => visitor.op_multiply(pos),
			Divide     => visitor.op_divide(pos),
			Remainder  => visitor.op_remainder(pos),
			Equals     => visitor.op_equals(pos),
			NotEquals  => visitor.op_notequals(pos),
			Greater    => visitor.op_greater(pos),
			Lesser     => visitor.op_lesser(pos),
			GreaterEq  => visitor.op_greatereq(pos),
			LesserEq   => visitor.op_lessereq(pos),
		}
	}
}

pub trait OpCodeVisitor<T> {
	fn op_return(&mut self, _pos: SourcePos) -> T;
	fn op_const(&mut self, _pos: SourcePos) -> T;
	fn op_const_16(&mut self, _pos: SourcePos) -> T;
	fn op_false(&mut self, _pos: SourcePos) -> T;
	fn op_true(&mut self, _pos: SourcePos) -> T;
	fn op_none(&mut self, _pos: SourcePos) -> T;
	fn op_template(&mut self, _pos: SourcePos) -> T;
	fn op_negate(&mut self, _pos: SourcePos) -> T;
	fn op_identity(&mut self, _pos: SourcePos) -> T;
	fn op_not(&mut self, _pos: SourcePos) -> T;
	fn op_add(&mut self, _pos: SourcePos) -> T;
	fn op_subtract(&mut self, _pos: SourcePos) -> T;
	fn op_multiply(&mut self, _pos: SourcePos) -> T;
	fn op_divide(&mut self, _pos: SourcePos) -> T;
	fn op_remainder(&mut self, _pos: SourcePos) -> T;
	fn op_equals(&mut self, _pos: SourcePos) -> T;
	fn op_notequals(&mut self, _pos: SourcePos) -> T;
	fn op_greater(&mut self, _pos: SourcePos) -> T;
	fn op_lesser(&mut self, _pos: SourcePos) -> T;
	fn op_greatereq(&mut self, _pos: SourcePos) -> T;
	fn op_lessereq(&mut self, _pos: SourcePos) -> T;
}
