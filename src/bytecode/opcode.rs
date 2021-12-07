
use self::OpCode::*;

pub type Value = f64;

static OP_CODES: &[OpCode] = &[Return, Const, LongConst];

#[derive(Copy, PartialEq, Eq, Debug, Clone)]
pub enum OpCode {
	Return,
	Const,
	LongConst,
}

impl From<u8> for OpCode {
	fn from(code: u8) -> Self {
		OP_CODES
		.get(code as usize)
		.expect(&format!("Unknown opcode {}", code))
		.to_owned()
	}
}

impl OpCode {
	pub fn accept<T>(&self, visitor: &mut dyn OpCodeVisitor<T>) -> T {
		match self {
			Return    => visitor.op_return(),
			Const     => visitor.op_const(),
			LongConst => visitor.op_long_const(),
		}
	}
}

pub trait OpCodeVisitor<T> {
	fn op_return(&mut self) -> T;
	fn op_const(&mut self) -> T;
	fn op_long_const(&mut self) -> T;
}
