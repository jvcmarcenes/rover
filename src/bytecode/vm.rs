
use crate::{utils::{result::Result, source_pos::SourcePos, wrap::Wrap}, environment::{Environment, ValueMap}};

use super::{chunk::Chunk, opcode::{OpCodeVisitor, OpCode}, value::{Value, number::Number, bool::Bool, none::ValNone, string::Str}};

#[cfg(feature = "trace_exec")]
use super::disassembler::Disassembler;

pub struct VM {
	chunk: Chunk,
	stack: Vec<Box<dyn Value>>,
	src_info_stack: Vec<SourcePos>,
	env: Environment<Box<dyn Value>>
}

impl VM {

	pub fn new(chunk: Chunk) -> Self {
		Self {
			chunk,
			stack: Vec::new(),
			src_info_stack: Vec::new(),
			env: Environment::new(ValueMap::new()),
		}
	}

	// fn next(&mut self) -> (u8, SourcePos) {
	// 	self.chunk.next().expect("expected a byte")
	// }

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

	fn peek(&self) -> (Box<dyn Value>, SourcePos) {
		(
			self.stack.last().expect("No value on the stack to peep").clone(),
			self.src_info_stack.last().expect("No value on the stack to peep").clone(),
		)
	}

	fn constant(&self, i: usize) -> Box<dyn Value> {
		self.chunk.constant(i)
	}

	fn binary<F : Fn((Box<dyn Value>, SourcePos), (Box<dyn Value>, SourcePos), SourcePos) -> Result<Box<dyn Value>>>(&mut self, op: F) -> Result<()> {
		let b = self.pop();
		let a = self.pop();
		let pos = self.chunk.get_src_info();
		let res = op(a, b, pos)?;
		self.push(res, pos);
		Ok(())
	}

	#[cfg(feature = "trace_exec")]
	fn print_stack(&self) {
		print!("stack: [");
		for value in &self.stack {
			print!("{}, ", value.displayf());
		}
		println!("]");
	}

	// #[cfg(feature = "trace_exec")]
	// fn run_chunk(&mut self, chunk: &mut Chunk) -> Result<()> {
	// 	self.env.push_new();
	// 	while let Some((code, pos)) = chunk.next() {
	// 		self.print_stack();
	// 		Disassembler::new(self.chunk.clone()).disassemble_instr(code, pos);
	// 		OpCode::from(code).accept(self, pos)?;
	// 	}
	// 	self.env.pop();
	// 	self.print_stack();
	// 	Ok(())
	// }

	#[cfg(feature = "trace_exec")]
	pub fn run(&mut self) -> Result<()> {
		while let Some(code) = self.chunk.next() {
			self.print_stack();
			Disassembler::new(self.chunk.clone()).disassemble_instr(code, self.chunk.get_src_info());
			OpCode::from(code).accept(self)?;
		}
		self.print_stack();
		Ok(())
	}

	// #[cfg(not(feature = "trace_exec"))]
	// fn run_chunk(&mut self, chunk: &mut Chunk) -> Result<()> {
	// 	self.env.push_new();
	// 	while let Some((code, pos)) = chunk.next() {
	// 		OpCode::from(code).accept(self, pos)?;
	// 	}
	// 	self.env.pop();
	// 	Ok(())
	// }

	#[cfg(not(feature = "trace_exec"))]
	pub fn run(&mut self) -> Result<()> {
		while let Some(code) = self.chunk.next() {
			OpCode::from(code).accept(self)?;
		}
		Ok(())
	}

}

impl OpCodeVisitor<Result<()>> for VM {
	
	fn op_pop(&mut self) -> Result<()> {
		self.pop();
		Ok(())
	}
	
	fn op_define(&mut self) -> Result<()> {
		let id = self.chunk.read8() as usize;
		let val = self.pop().0;
		self.env.define(id, val);
		Ok(())
	}

	fn op_load(&mut self) -> Result<()> {
		let id = self.chunk.read8() as usize;
		let val = self.env.get(id);
		self.push(val, self.chunk.get_src_info());
		Ok(())
	}

	fn op_store(&mut self) -> Result<()> {
		let id = self.chunk.read8() as usize;
		let val = self.pop().0;
		self.env.assign(id, val);
		Ok(())
	}

	fn op_define16(&mut self) -> Result<()> {
		let id = self.chunk.read16() as usize;
		let val = self.pop().0;
		self.env.define(id, val);
		Ok(())
	}

	fn op_load16(&mut self) -> Result<()> {
		let id = self.chunk.read16() as usize;
		let val = self.env.get(id);
		self.push(val, self.chunk.get_src_info());
		Ok(())
	}

	fn op_store16(&mut self) -> Result<()> {
		let id = self.chunk.read16() as usize;
		let val = self.pop().0;
		self.env.assign(id, val);
		Ok(())
	}

	fn op_jump(&mut self) -> Result<()> {
		let offset = self.chunk.read16();
		self.chunk.jump(offset);
		Ok(())
	}

	fn op_false_jump(&mut self) -> Result<()> {
		let offset = self.chunk.read16();
		if !self.peek().0.truthy() { self.chunk.jump(offset); }
		Ok(())
	}

	fn op_true_jump(&mut self) -> Result<()> {
		let offset = self.chunk.read16();
		if self.peek().0.truthy() { self.chunk.jump(offset); }
		Ok(())
	}

	fn op_jump_back(&mut self) -> Result<()> {
		let offset = self.chunk.read16();
		self.chunk.jump_back(offset);
		Ok(())
	}

	fn op_return(&mut self,) -> Result<()> {
		todo!()
	}
	
	fn op_const(&mut self) -> Result<()> {
		let c = self.chunk.read8() as usize;
		self.push(self.constant(c), self.chunk.get_src_info());
		Ok(())
	}
	
	fn op_const_16(&mut self) -> Result<()> {
		let c = self.chunk.read16() as usize;
		self.push(self.constant(c), self.chunk.get_src_info());
		Ok(())
	}
	
	fn op_false(&mut self) -> Result<()> {
		self.push(Bool::create(false), self.chunk.get_src_info());
		Ok(())
	}
	
	fn op_true(&mut self) -> Result<()> {
		self.push(Bool::create(true), self.chunk.get_src_info());
		Ok(())
	}
	
	fn op_none(&mut self) -> Result<()> {
		self.push(ValNone::create(), self.chunk.get_src_info());
		Ok(())
	}
	
	fn op_template(&mut self) -> Result<()> {
		let mut str = String::new();
		let len = self.chunk.read8();
		for _ in 0..len {
			let (v0, _) = self.pop();
			let mut s0 = v0.display()?;
			s0.push_str(&str);
			str = s0;
		}
		self.push(Str::create(str), self.chunk.get_src_info());
		Ok(())
	}
	
	fn op_negate(&mut self) -> Result<()> {
		let (v0, p0) = self.pop();
		let val = -v0.as_num(p0)?.data;
		self.push(Number::create(val), self.chunk.get_src_info());
		Ok(())
	}
	
	fn op_not(&mut self) -> Result<()> {
		let v0 = self.pop().0;
		let val = !v0.truthy();
		self.push(Bool::create(val), self.chunk.get_src_info());
		Ok(())
	}
	
	fn op_identity(&mut self,) -> Result<()> {
		Ok(())
	}
	
	fn op_add(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| if b.is_string() { Str::new(a.display()?).add(b, apos, bpos, pos) } else { a.add(b, apos, bpos, pos) })
	}
	
	fn op_subtract(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| (a.sub(b, apos, bpos, pos)))
	}
	
	fn op_multiply(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| (a.mul(b, apos, bpos, pos)))
	}
	
	fn op_divide(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| (a.div(b, apos, bpos, pos)))
	}
	
	fn op_remainder(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| (a.rem(b, apos, bpos, pos)))
	}
	
	fn op_equals(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| Bool::create(a.equ(b, apos, bpos, pos)?).wrap())
	}
	
	fn op_notequals(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| Bool::create(!a.equ(b, apos, bpos, pos)?).wrap())
	}
	
	fn op_greater(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| Bool::create(a.cmp(b, apos, bpos, pos)? > 0).wrap())
	}
	
	fn op_lesser(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| Bool::create(a.cmp(b, apos, bpos, pos)? < 0).wrap())
	}
	
	fn op_greatereq(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| Bool::create(a.cmp(b, apos, bpos, pos)? >= 0).wrap())
	}
	
	fn op_lessereq(&mut self) -> Result<()> {
		self.binary(|(a, apos), (b, bpos), pos| Bool::create(a.cmp(b, apos, bpos, pos)? <= 0).wrap())
	}
	
}
