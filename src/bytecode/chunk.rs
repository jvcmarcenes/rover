
use std::vec::IntoIter;

use crate::utils::source_pos::SourcePos;

use super::{opcode::OpCode, value::Value};

#[derive(Debug, Clone, Default)]
pub struct Chunk {
	pub code: Vec<u8>,
	pub source_info: Vec<SourcePos>,
	pub constants: Vec<Box<dyn Value>>,
}

impl Chunk {
	
	pub fn new() -> Self {
		Self::default()
	}
	
	pub fn write_instr(&mut self, instr: OpCode, src_info: SourcePos) {
		self.code.push(instr as u8);
		self.source_info.push(src_info);
	}
	
	pub fn write_byte(&mut self, byte: u8, src_info: SourcePos) {
		self.code.push(byte);
		self.source_info.push(src_info);
	}
	
	pub fn write_const(&mut self, value: Box <dyn Value>, src_info: SourcePos) {
		let c = self.add_const(value);
		if c > u8::MAX as usize {
			self.write_instr(OpCode::Const16, src_info);
			self.write_byte((c >> 8) as u8, src_info);
			self.write_byte(c as u8, src_info);
		} else {
			self.write_instr(OpCode::Const, src_info);
			self.write_byte(c as u8, src_info)
		}
	}
	
	pub fn add_const(&mut self, value: Box<dyn Value>) -> usize {
		self.constants.push(value);
		return self.constants.len() - 1;
	}
	
}

#[derive(Clone)]
pub struct ChunkIter {
	pub offset: usize,
	next_offset: usize,
	code: IntoIter<(u8, SourcePos)>,
	constants: Vec<Box<dyn Value>>,	
}

impl ChunkIter {
	pub fn constant(&self, index: usize) -> Box<dyn Value> {
		self.constants[index].clone()
	}

	pub fn read8(&mut self) -> u8 {
		self.next().expect("expected byte").0
	}

	pub fn read16(&mut self) -> u16 {
		let c0 = self.read8() as u16;
		let c1 = self.read8() as u16;
		(c0 << 8) + c1
	}

	// pub fn read24(&mut self) -> u32 {
	// 	let c0 = self.read8() as u32;
	// 	let c1 = self.read8() as u32;
	// 	let c2 = self.read8() as u32;
	// 	(c0 << 16) + (c1 << 8) + c2
	// }

}

impl From<Chunk> for ChunkIter {
	fn from(chunk: Chunk) -> Self {
		Self {
			offset: 0,
			next_offset: 0,
			code: chunk.code.iter().cloned().zip(chunk.source_info.iter().cloned()).collect::<Vec<_>>().into_iter(),
			constants: chunk.constants.clone(),
		}
	}
}

impl Iterator for ChunkIter {
	type Item = (u8, SourcePos);
	
	fn next(&mut self) -> Option<Self::Item> {
		self.offset = self.next_offset;
		self.next_offset += 1;
		self.code.next()
	}
}
