
use crate::utils::source_pos::SourcePos;

use super::{opcode::OpCode, value::Value};

#[derive(Debug, Clone, Default)]
pub struct Chunk {
	pub code: Vec<u8>,
	pub source_info: Vec<SourcePos>,
	pub constants: Vec<Box<dyn Value>>,
	offset: (usize, usize),
}

impl Chunk {

	pub fn new() -> Self {
		Self::default()
	}

	pub fn write_instr(&mut self, instr: OpCode, src_info: SourcePos) {
		self.code.push(instr as u8);
		self.source_info.push(src_info);
	}

	pub fn write_u8(&mut self, data: u8, src_info: SourcePos) {
		self.code.push(data);
		self.source_info.push(src_info);
	}

	pub fn write_u16(&mut self, data: u16, src_info: SourcePos) {
		self.write_u8((data >> 8) as u8, src_info);
		self.write_u8(data as u8, src_info);
	}

	pub fn write_const(&mut self, value: Box <dyn Value>, src_info: SourcePos) {
		let c = self.add_const(value);
		if c > u8::MAX as usize {
			if c > u16::MAX as usize { panic!("not enough space to store constant") }
			self.write_instr(OpCode::Const16, src_info);
			self.write_u16(c as u16, src_info);
		} else {
			self.write_instr(OpCode::Const, src_info);
			self.write_u8(c as u8, src_info);
		}
	}

	// pub fn write_jump(&mut self, instr: OpCode, src_info: SourcePos) -> usize {
	// 	self.write_instr(instr, src_info);
	// 	let anchor = self.offset.0;
	// 	self.write_u16(0xffff, src_info);
	// 	return anchor;
	// }

	// pub fn patch_jump(&mut self, anchor: usize) {
	// 	let offset = self.offset.0;
	// 	*self.code.get_mut(anchor).unwrap() = (offset >> 8) as u8;
	// 	*self.code.get_mut(anchor + 1).unwrap() = offset as u8;
	// }

	fn add_const(&mut self, value: Box<dyn Value>) -> usize {
		if self.constants.contains(&value) {
			return self.constants.iter().position(|val| val == &value).unwrap()
		}
		self.constants.push(value);
		return self.constants.len() - 1;
	}

}

impl Chunk {

	pub fn next(&mut self) -> Option<(u8, SourcePos)> {
		self.offset.0 = self.offset.1;
		self.offset.1 += 1;
		if let (Some(&byte), Some(&pos)) = (self.code.get(self.offset.0), self.source_info.get(self.offset.0)) {
			Some((byte, pos))
		} else {
			None
		}
	}

	pub fn constant(&self, index: usize)  -> Box<dyn Value> {
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

	pub fn offset(&self) -> usize {
		self.offset.0
	}

	// pub fn jump(&mut self, offset: u16) {
	// 	self.offset.0 += offset as usize;
	// 	self.offset.1 = self.offset.0 + 1;
	// }

}
