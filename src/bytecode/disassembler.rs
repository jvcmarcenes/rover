
use std::vec::IntoIter;

use crate::{utils::source_pos::SourcePos, bytecode::opcode::OpCode};

use super::{chunk::Chunk, opcode::{Value, OpCodeVisitor}};

pub struct Disassembler {
	offset: usize,
	next_offset: usize,
	code: IntoIter<(u8, SourcePos)>,
	constants: Vec<Value>,
}

impl Disassembler {

	pub fn new(chunk: &Chunk) -> Self {
		Self {
			offset: 0,
			next_offset: 0,
			code: chunk.code.iter().cloned().zip(chunk.source_info.iter().cloned()).collect::<Vec<_>>().into_iter(),
			constants: chunk.constants.clone(),
		}
	}

	fn next(&mut self) -> (u8, SourcePos) {
		self.offset = self.next_offset;
		self.next_offset += 1;
		self.code.next().expect("expected a byte")
	}

	pub fn disassemble(&mut self, name: &str) {
		println!("== {} ==", name);

		while let Some((code, pos)) = self.code.next() {
			print!("{:04} ", self.offset);
			print!("[{:04}:{:04}] ", pos.lin, pos.col);
			OpCode::from(code).accept(self);
			self.offset += 1;
		}
	}

	fn simple_instr(&mut self, name: &str) {
		println!("{:-16}", name);
	}
	
}

impl OpCodeVisitor<()> for Disassembler {
	fn op_return(&mut self) {
		self.simple_instr("OP_RETURN");
	}

	fn op_const(&mut self) {
		let c = self.next().0 as usize;
		println!("{:-16} {:4} ({})", "OP_CONST", c, self.constants[c]);
	}

	fn op_long_const(&mut self) {
		let c0 = self.next().0 as usize;
		let c1 = self.next().0 as usize;
		let c2 = self.next().0 as usize;
		let c = (c2 << 16) + (c1 << 8) + (c0);
		println!("{:-16} {:4} ({})", "OP_LONG_CONST", c, self.constants[c]);
	}
}
