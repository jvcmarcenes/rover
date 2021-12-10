
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use self::OpCode::*;

#[derive(Copy, PartialEq, Eq, Debug, Clone, FromPrimitive)]
pub enum OpCode {
	Pop,
	Define, Load, Store, Define16, Load16, Store16,
	Jump, FalseJump, TrueJump, JumpBack, Return,

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
	pub fn accept<T>(&self, visitor: &mut dyn OpCodeVisitor<T>) -> T {
		match self {
			Pop        => visitor.op_pop(),
			Define     => visitor.op_define(),
			Load       => visitor.op_load(),
			Store      => visitor.op_store(),
			Define16   => visitor.op_define16(),
			Load16     => visitor.op_load16(),
			Store16    => visitor.op_store16(),
			Jump       => visitor.op_jump(),
			FalseJump  => visitor.op_false_jump(),
			TrueJump   => visitor.op_true_jump(),
			JumpBack   => visitor.op_jump_back(),
			Return     => visitor.op_return(),
			Const      => visitor.op_const(),
			Const16    => visitor.op_const_16(),
			ConstFalse => visitor.op_false(),
			ConstTrue  => visitor.op_true(),
			ConstNone  => visitor.op_none(),
			StrTemplate => visitor.op_template(),
			Negate     => visitor.op_negate(),
			Identity   => visitor.op_identity(),
			Not        => visitor.op_not(),
			Add        => visitor.op_add(),
			Subtract   => visitor.op_subtract(),
			Multiply   => visitor.op_multiply(),
			Divide     => visitor.op_divide(),
			Remainder  => visitor.op_remainder(),
			Equals     => visitor.op_equals(),
			NotEquals  => visitor.op_notequals(),
			Greater    => visitor.op_greater(),
			Lesser     => visitor.op_lesser(),
			GreaterEq  => visitor.op_greatereq(),
			LesserEq   => visitor.op_lessereq(),
		}
	}
}

pub trait OpCodeVisitor<T> {
	fn op_pop(&mut self) -> T;
	
	fn op_define(&mut self) -> T;
	fn op_load(&mut self) -> T;
	fn op_store(&mut self) -> T;
	fn op_define16(&mut self) -> T;
	fn op_load16(&mut self) -> T;
	fn op_store16(&mut self) -> T;
	
	fn op_jump(&mut self) -> T;
	fn op_false_jump(&mut self) -> T;
	fn op_true_jump(&mut self) -> T;
	fn op_jump_back(&mut self) -> T;
	fn op_return(&mut self) -> T;
	
	fn op_const(&mut self) -> T;
	fn op_const_16(&mut self) -> T;
	fn op_false(&mut self) -> T;
	fn op_true(&mut self) -> T;
	fn op_none(&mut self) -> T;
	fn op_template(&mut self) -> T;
	
	fn op_negate(&mut self) -> T;
	fn op_identity(&mut self) -> T;
	fn op_not(&mut self) -> T;
	
	fn op_add(&mut self) -> T;
	fn op_subtract(&mut self) -> T;
	fn op_multiply(&mut self) -> T;
	fn op_divide(&mut self) -> T;
	fn op_remainder(&mut self) -> T;
	fn op_equals(&mut self) -> T;
	fn op_notequals(&mut self) -> T;
	fn op_greater(&mut self) -> T;
	fn op_lesser(&mut self) -> T;
	fn op_greatereq(&mut self) -> T;
	fn op_lessereq(&mut self) -> T;
}
