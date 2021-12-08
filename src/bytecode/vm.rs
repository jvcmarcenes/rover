
use crate::utils::{result::Result, source_pos::SourcePos, wrap::Wrap};

use super::{chunk::{ChunkIter, Chunk}, opcode::{OpCodeVisitor, OpCode}, value::{Value, number::Number, bool::Bool, none::ValNone, string::Str}};

pub struct VM {
	chunk: ChunkIter,
	stack: Vec<Box<dyn Value>>,
	src_info_stack: Vec<SourcePos>,
}

impl VM {
	
	pub fn new(chunk: Chunk) -> Self {
		Self {
			chunk: ChunkIter::from(chunk),
			stack: Vec::new(),
			src_info_stack: Vec::new(),
		}
	}
	
	fn next(&mut self) -> (u8, SourcePos) {
		self.chunk.next().expect("expected a byte")
	}
	
	fn push(&mut self, val: Box<dyn Value>, pos: SourcePos) {
		self.stack.push(val);
		self.src_info_stack.push(pos);
	}
	
	fn pop(&mut self) -> (Box<dyn Value>, SourcePos) {
		(
			self.stack.pop().expect("No Value on the stack to pop"),
			self.src_info_stack.pop().expect("No Value on the stack to pop"),
		)
	}
	
	fn binary<F : Fn((Box<dyn Value>, SourcePos), (Box<dyn Value>, SourcePos)) -> Result<Box<dyn Value>>>(&mut self, op: F, pos: SourcePos) -> Result<()> {
		let b = self.pop();
		let a = self.pop();
		let res = op(a, b)?;
		self.push(res, pos);
		Ok(())
	}
	
	#[cfg(feature = "trace_exec")]
	fn debug(&self, code: u8, pos: SourcePos) {
		print!("stack: [");
		for value in &self.stack {
			print!("{}, ", value);
		}
		println!("]");
		
		super::disassembler::Disassembler::from(self.chunk.clone()).disassembe_instr(code, pos);
	}
	
	#[cfg(feature = "trace_exec")]
	pub fn run(&mut self) -> Result<()> {
		while let Some((code, pos)) = self.chunk.next() {
			self.debug(code, pos);
			OpCode::from(code).accept(self, pos)?;
		}
		Ok(())
	}
	
	#[cfg(not(feature = "trace_exec"))]
	pub fn run(&mut self) -> Result<()> {
		while let Some((code, pos)) = self.chunk.next() {
			OpCode::from(code).accept(self, pos)?;
		}
		Ok(())
	}
	
}

impl OpCodeVisitor<Result<()>> for VM {
	
	fn op_return(&mut self, _pos: SourcePos) -> Result<()> {
		println!("{}", self.pop().0.display()?);
		Ok(())
	}
	
	fn op_const(&mut self, pos: SourcePos) -> Result<()> {
		let c = self.chunk.read8() as usize;
		self.push(self.chunk.constant(c), pos);
		Ok(())
	}
	
	fn op_const_16(&mut self, pos: SourcePos) -> Result<()> {
		let c = self.chunk.read16() as usize;
		self.push(self.chunk.constant(c), pos);
		Ok(())
	}
	
	fn op_false(&mut self, pos: SourcePos) -> Result<()> {
		self.push(Bool::create(false), pos);
		Ok(())
	}
	
	fn op_true(&mut self, pos: SourcePos) -> Result<()> {
		self.push(Bool::create(true), pos);
		Ok(())
	}
	
	fn op_none(&mut self, pos: SourcePos) -> Result<()> {
		self.push(ValNone::create(), pos);
		Ok(())
	}
	
	fn op_template(&mut self, pos: SourcePos) -> Result<()> {
		let mut str = String::new();
		let len = self.chunk.read8();
		for _ in 0..len {
			let (v0, p0) = self.pop();
			let mut s0 = v0.display()?;
			s0.push_str(&str);
			str = s0;
		}
		self.push(Str::create(str), pos);
		Ok(())
	}
	
	fn op_negate(&mut self, pos: SourcePos) -> Result<()> {
		let (v0, p0) = self.pop();
		let val = -v0.as_num(p0)?.data;
		self.push(Number::create(val), pos);
		Ok(())
	}
	
	fn op_not(&mut self, pos: SourcePos) -> Result<()> {
		let v0 = self.pop().0;
		let val = !v0.truthy();
		self.push(Bool::create(val), pos);
		Ok(())
	}
	
	fn op_identity(&mut self, _pos: SourcePos) -> Result<()> {
		Ok(())
	}
	
	fn op_add(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| if b.is_string() { Str::new(a.display()?).add(b, apos, bpos, pos) } else { a.add(b, apos, bpos, pos) }, pos)
	}
	
	fn op_subtract(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| (a.sub(b, apos, bpos, pos)), pos)
	}
	
	fn op_multiply(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| (a.mul(b, apos, bpos, pos)), pos)
	}
	
	fn op_divide(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| (a.div(b, apos, bpos, pos)), pos)
	}
	
	fn op_remainder(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| (a.rem(b, apos, bpos, pos)), pos)
	}
	
	fn op_equals(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| Bool::create(a.equ(b, apos, bpos, pos)?).wrap(), pos)
	}
	
	fn op_notequals(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| Bool::create(!a.equ(b, apos, bpos, pos)?).wrap(), pos)
	}
	
	fn op_greater(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| Bool::create(a.cmp(b, apos, bpos, pos)? > 0).wrap(), pos)
	}
	
	fn op_lesser(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| Bool::create(a.cmp(b, apos, bpos, pos)? < 0).wrap(), pos)
	}
	
	fn op_greatereq(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| Bool::create(a.cmp(b, apos, bpos, pos)? >= 0).wrap(), pos)
	}
	
	fn op_lessereq(&mut self, pos: SourcePos) -> Result<()> {
		self.binary(|(a, apos), (b, bpos)| Bool::create(a.cmp(b, apos, bpos, pos)? <= 0).wrap(), pos)
	}
	
}
