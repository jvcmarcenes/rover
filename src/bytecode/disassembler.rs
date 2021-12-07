
use crate::{utils::source_pos::SourcePos, bytecode::opcode::OpCode};

use super::{chunk::{Chunk, ChunkIter}, opcode::{Value, OpCodeVisitor}};

fn simple_instr(name: &str) {
	println!("{:-16}", name);
}

pub struct Disassembler {
	chunk: ChunkIter,
}

impl Disassembler {
	
	pub fn new(chunk: Chunk) -> Self {
		Self {
			chunk: ChunkIter::from(chunk),
		}
	}
	
	pub fn from(chunk: ChunkIter) -> Self {
		Self { chunk }
	}
	
	fn next(&mut self) -> (u8, SourcePos) {
		self.chunk.next().expect("expected a byte")
	}
	
	fn read_const(&mut self) -> (usize, Value) {
		let c = self.next().0 as usize;
		(c, self.chunk.constant(c))
	}
	
	fn read_long_const(&mut self) -> (usize, Value) {
		let c0 = self.next().0 as usize;
		let c1 = self.next().0 as usize;
		let c2 = self.next().0 as usize;
		let c = (c2 << 16) + (c1 << 8) + (c0);
		(c, self.chunk.constant(c))
	}
	
	pub fn disassemble(&mut self, name: &str) {
		println!("== {} ==", name);
		
		while let Some((code, pos)) = self.chunk.next() {
			self.disassembe_instr(code, pos);
		}
	}
	
	pub fn disassembe_instr(&mut self, code: u8, pos: SourcePos) {
		print!("{:04} ", self.chunk.offset);
		print!("[{:04}:{:04}] ", pos.lin, pos.col);
		OpCode::from(code).accept(self);
	}
	
}



impl OpCodeVisitor<()> for Disassembler {
	fn op_return(&mut self) {
		simple_instr("OP_RETURN");
	}
	
	fn op_const(&mut self) {
		let (index, val) = self.read_const();
		println!("{:-16} {:4} ({})", "OP_CONST", index, val);
	}
	
	fn op_long_const(&mut self) {
		let (index, val) = self.read_long_const();
		println!("{:-16} {:4} ({})", "OP_LONG_CONST", index, val);
	}
	
	fn op_negate(&mut self) -> () {
		simple_instr("OP_NEGATE")
	}
	
	fn op_add(&mut self) -> () {
		simple_instr("OP_ADD")
	}
	
	fn op_subtract(&mut self) -> () {
		simple_instr("OP_SUBTRACT")
	}
	
	fn op_multiply(&mut self) -> () {
		simple_instr("OP_MULTIPLY")
	}
	
	fn op_divide(&mut self) -> () {
		simple_instr("OP_DIVIDE")
	}
	
	fn op_remainder(&mut self) -> () {
		simple_instr("OP_REMAINDER")
	}
}
