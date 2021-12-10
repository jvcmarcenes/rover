
use crate::{utils::source_pos::SourcePos, bytecode::opcode::OpCode};

use super::{chunk::Chunk, opcode::OpCodeVisitor, value::Value};

fn simple_instr(name: &str) {
	println!("{:-16}", name);
}

pub struct Disassembler {
	chunk: Chunk,
}

impl Disassembler {
	
	pub fn new(chunk: Chunk) -> Self {
		Self { chunk }
	}

	// fn next(&mut self) -> (u8, SourcePos) {
	// 	self.chunk.next().expect("expected a byte")
	// }
	
	fn read_const_8(&mut self) -> (usize, Box<dyn Value>) {
		let c = self.chunk.read8() as usize;
		(c, self.chunk.constant(c))
	}
	
	fn read_const_16(&mut self) -> (usize, Box<dyn Value>) {
		let c = self.chunk.read16() as usize;
		(c, self.chunk.constant(c))
	}
	
	pub fn disassemble(&mut self, name: &str) {
		println!("== {} ==", name);
		
		while let Some((code, pos)) = self.chunk.next() {
			self.disassemble_instr(code, pos);
		}
	}
	
	pub fn disassemble_instr(&mut self, code: u8, pos: SourcePos) {
		print!("{:04} ", self.chunk.offset());
		print!("[{:04}:{:04}] ", pos.lin, pos.col);
		OpCode::from(code).accept(self, pos);
	}
	
}

impl OpCodeVisitor<()> for Disassembler {
	
	fn op_pop(&mut self, _pos: SourcePos) {
		simple_instr("POP");
	}

	fn op_define(&mut self, _pos: SourcePos) {
		let id = self.chunk.read8();
		println!("{:-16} {:4}", "DEFINE", id);
	}

	fn op_load(&mut self, _pos: SourcePos) {
		let id = self.chunk.read8();
		println!("{:-16} {:4}", "LOAD", id);
	}

	fn op_store(&mut self, _pos: SourcePos) {
		let id = self.chunk.read8();
		println!("{:-16} {:4}", "STORE", id);
	}

	fn op_define16(&mut self, _pos: SourcePos) {
		let id = self.chunk.read16();
		println!("{:-16} {:4}", "DEFINE_16", id);
	}

	fn op_load16(&mut self, _pos: SourcePos) {
		let id = self.chunk.read16();
		println!("{:-16} {:4}", "LOAD_16", id);
	}

	fn op_store16(&mut self, _pos: SourcePos) {
		let id = self.chunk.read16();
		println!("{:-16} {:4}", "STORE_16", id);
	}

	// fn op_jump(&mut self, _pos: SourcePos) {
	// 	let offset = self.chunk.read16();
	// 	println!("{:-16} {:4}", "JUMP", offset);
	// }

	// fn op_false_jump(&mut self, _pos: SourcePos) {
	// 	let offset = self.chunk.read16();
	// 	println!("{:-16} {:4}", "FALSE_JUMP", offset);
	// }

	// fn op_true_jump(&mut self, _pos: SourcePos) {
	// 	let offset = self.chunk.read16();
	// 	println!("{:-16} {:4}", "TRUE_JUMP", offset);
	// }

	fn op_return(&mut self, _pos: SourcePos) {
		simple_instr("RETURN");
	}
	
	fn op_const(&mut self, _pos: SourcePos) {
		let (index, val) = self.read_const_8();
		println!("{:-16} {:4} ({})", "CONST", index, val.displayf());
	}
	
	fn op_const_16(&mut self, _pos: SourcePos) {
		let (index, val) = self.read_const_16();
		println!("{:-16} {:4} ({})", "CONST_16", index, val.displayf());
	}
	
	fn op_false(&mut self, _pos: SourcePos) {
		simple_instr("CONST_FALSE");
	}
	
	fn op_true(&mut self, _pos: SourcePos) {
		simple_instr("CONST_TRUE");
	}
	
	fn op_none(&mut self, _pos: SourcePos) {
		simple_instr("CONST_NONE");
	}
	
	fn op_template(&mut self, _pos: SourcePos) {
		let len = self.chunk.read8();
		println!("{:-16} {:4}", "TEMPLATE", len);
	}
	
	fn op_negate(&mut self, _pos: SourcePos) {
		simple_instr("NEGATE");
	}
	
	fn op_identity(&mut self, _pos: SourcePos) {
		simple_instr("IDENTITY");
	}
	
	fn op_not(&mut self, _pos: SourcePos) {
		simple_instr("NOT");
	}
	
	fn op_add(&mut self, _pos: SourcePos) {
		simple_instr("ADD");
	}
	
	fn op_subtract(&mut self, _pos: SourcePos) {
		simple_instr("SUBTRACT");
	}
	
	fn op_multiply(&mut self, _pos: SourcePos) {
		simple_instr("MULTIPLY");
	}
	
	fn op_divide(&mut self, _pos: SourcePos) {
		simple_instr("DIVIDE");
	}
	
	fn op_remainder(&mut self, _pos: SourcePos) {
		simple_instr("REMAINDER");
	}
	
	fn op_equals(&mut self, _pos: SourcePos) {
		simple_instr("EQUALS");
	}
	
	fn op_notequals(&mut self, _pos: SourcePos) {
		simple_instr("NOT_EQUALS");
	}
	
	fn op_greater(&mut self, _pos: SourcePos) {
		simple_instr("GREATER");
	}
	
	fn op_lesser(&mut self, _pos: SourcePos) {
		simple_instr("LESSER");
	}
	
	fn op_greatereq(&mut self, _pos: SourcePos) {
		simple_instr("GREATER_EQUALS");
	}
	
	fn op_lessereq(&mut self, _pos: SourcePos) {
		simple_instr("LESSER_EQUALS");
	}
	
}
