
use crate::utils::{result::Result, source_pos::SourcePos, wrap::Wrap};

use super::{chunk::{ChunkIter, Chunk}, opcode::{Value, OpCodeVisitor, OpCode}};

pub struct VM {
	chunk: ChunkIter,
	stack: Vec<Value>,
}

impl VM {
	
	pub fn new(chunk: Chunk) -> Self {
		Self {
			chunk: ChunkIter::from(chunk),
			stack: Vec::new(),
		}
	}
	
	fn next(&mut self) -> (u8, SourcePos) {
		self.chunk.next().expect("expected a byte")
	}
	
	fn push(&mut self, val: Value) {
		self.stack.push(val);
	}
	
	fn pop(&mut self) -> Value {
		self.stack.pop().expect("No Value on the stack to pop")
	}

	fn binary(&mut self, op: fn(Value, Value) -> Result<Value>) -> Result<()> {
		let b = self.pop();
		let a = self.pop();
		let res = op(a, b)?;
		self.push(res);
		Ok(())
	}
	
	#[cfg(feature = "debug")]
	fn debug(&self, code: u8, pos: SourcePos) {
		print!("stack: [");
		for value in &self.stack {
			print!("{}, ", value);
		}
		println!("]");
		
		super::disassembler::Disassembler::from(self.chunk.clone()).disassembe_instr(code, pos);
	}
	
	#[cfg(feature = "debug")]
	pub fn run(&mut self) -> Result<()> {
		while let Some((code, pos)) = self.chunk.next() {
			self.debug(code, pos);
			OpCode::from(code).accept(self)?;
		}
		Ok(())
	}
	
	#[cfg(not(feature = "debug"))]
	pub fn run(&mut self) -> Result<()> {
		while let Some((code, _)) = self.chunk.next() {
			OpCode::from(code).accept(self)?;
		}
		Ok(())
	}
	
}

impl OpCodeVisitor<Result<()>> for VM {
	fn op_return(&mut self) -> Result<()> {
		println!("{}", self.pop());
		Ok(())
	}
	
	fn op_const(&mut self) -> Result<()> {
		let c = self.next().0 as usize;
		self.push(self.chunk.constant(c));
		Ok(())
	}
	
	fn op_long_const(&mut self) -> Result<()> {
		let c0 = self.next().0 as usize;
		let c1 = self.next().0 as usize;
		let c2 = self.next().0 as usize;
		let c = (c2 << 16) + (c1 << 8) + (c0);
		self.push(self.chunk.constant(c));
		Ok(())
	}
	
	fn op_negate(&mut self) -> Result<()> {
		let val = -self.pop();
		self.push(val);
		Ok(())
	}
	
	fn op_add(&mut self) -> Result<()> {
		self.binary(|a, b| (a + b).wrap())
	}
	
	fn op_subtract(&mut self) -> Result<()> {
		self.binary(|a, b| (a - b).wrap())
	}
	
	fn op_multiply(&mut self) -> Result<()> {
		self.binary(|a, b| (a * b).wrap())
	}
	
	fn op_divide(&mut self) -> Result<()> {
		self.binary(|a, b| (a / b).wrap())
	}
	
	fn op_remainder(&mut self) -> Result<()> {
		self.binary(|a, b| (a % b).wrap())
	}
}
