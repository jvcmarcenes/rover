
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use self::OpCode::*;

pub type Value = f64;

#[derive(Copy, PartialEq, Eq, Debug, Clone, FromPrimitive)]
pub enum OpCode {
	Return,
	Const,
	LongConst,
	
	Negate,
	Add, Subtract, Multiply, Divide, Remainder,
}

impl From<u8> for OpCode {
	fn from(code: u8) -> Self {
		OpCode::from_u8(code).expect(&format!("Unknown opcode {}", code))
	}
}

impl OpCode {
	pub fn accept<T>(&self, visitor: &mut dyn OpCodeVisitor<T>) -> T {
		match self {
			Return    => visitor.op_return(),
			Const     => visitor.op_const(),
			LongConst => visitor.op_long_const(),
			Negate    => visitor.op_negate(),
			Add       => visitor.op_add(),
			Subtract  => visitor.op_subtract(),
			Multiply  => visitor.op_multiply(),
			Divide    => visitor.op_divide(),
			Remainder => visitor.op_remainder(),
		}
	}
}

pub trait OpCodeVisitor<T> {
	fn op_return(&mut self) -> T;
	fn op_const(&mut self) -> T;
	fn op_long_const(&mut self) -> T;
	fn op_negate(&mut self) -> T;
	fn op_add(&mut self) -> T;
	fn op_subtract(&mut self) -> T;
	fn op_multiply(&mut self) -> T;
	fn op_divide(&mut self) -> T;
	fn op_remainder(&mut self) -> T;
}
