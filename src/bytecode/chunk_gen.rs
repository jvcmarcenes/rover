
#![allow(unused_variables)]

use crate::{utils::{result::{Result, ErrorList}, wrap::Wrap, source_pos::SourcePos}, ast::{statement::*, expression::*, identifier::Identifier}};

use super::{chunk::Chunk, opcode::OpCode, disassembler::Disassembler, value::{number::Number, string::Str}};

pub struct ChunkGen {
	chunk: Chunk
}

impl ChunkGen {

	pub fn new() -> Self {
		Self {
			chunk: Chunk::new(),
		}
	}

	fn chunk(&mut self) -> &mut Chunk {
		&mut self.chunk
	}

	fn generate_block(&mut self, block: Block) -> Result<()> {
		for stmt in block { stmt.accept(self)?; }
		Ok(())
	}

	pub fn generate(mut self, code: Block) -> Result<Chunk> {
		self.generate_block(code)?;
		if cfg!(feature = "print_code") {
			Disassembler::new(self.chunk.clone()).disassemble("code");
		}
		self.chunk.wrap()
	}

	fn write_var(&mut self, instr: OpCode, id: usize, pos: SourcePos) -> Result<()> {
		if id > u8::MAX as usize {
			if id > u16::MAX as usize { panic!("not enough space to store variable id") }
			match instr {
				OpCode::Define => self.chunk.write_instr(OpCode::Define16, pos),
				OpCode::Load   => self.chunk.write_instr(OpCode::Load16, pos),
				OpCode::Store  => self.chunk.write_instr(OpCode::Store, pos),
				_ => panic!("invalid instruction for variable"),
			}
			self.chunk.write_u16(id as u16, pos);
		} else {
			self.chunk.write_instr(instr, pos);
			self.chunk.write_u8(id as u8, pos);
		}
		Ok(())
	}

}

impl ExprVisitor<()> for ChunkGen {

	fn literal(&mut self, data: LiteralData, pos: SourcePos) -> Result<()> {
		match data {
			LiteralData::None => self.chunk().write_instr(OpCode::ConstNone, pos),
			LiteralData::Str(s) => self.chunk().write_const(Str::create(s), pos),
			LiteralData::Num(n) => self.chunk().write_const(Number::create(n), pos),
			LiteralData::Bool(b) => self.chunk().write_instr(if b { OpCode::ConstTrue } else { OpCode::ConstFalse }, pos),
			LiteralData::Template(exprs) => {
				if exprs.len() > u8::MAX as usize {
					return ErrorList::comp("Template string had over 255 elements".to_owned(), pos).err()
				}
				for expr in exprs.clone() { expr.accept(self)?; }
				self.chunk.write_instr(OpCode::StrTemplate, pos);
				self.chunk.write_u8(exprs.len() as u8, pos);
			},
			LiteralData::List(_) => todo!(),
			LiteralData::Object(_, _) => todo!(),
			LiteralData::Error(_) => todo!(),
		}
		Ok(())
	}
	
	fn binary(&mut self, data: BinaryData, pos: SourcePos) -> Result<()> {
		data.lhs.accept(self)?;
		data.rhs.accept(self)?;
		match data.op {
			BinaryOperator::Add => self.chunk().write_instr(OpCode::Add, pos),
			BinaryOperator::Sub => self.chunk().write_instr(OpCode::Subtract, pos),
			BinaryOperator::Mul => self.chunk().write_instr(OpCode::Multiply, pos),
			BinaryOperator::Div => self.chunk().write_instr(OpCode::Divide, pos),
			BinaryOperator::Rem => self.chunk().write_instr(OpCode::Remainder, pos),
			BinaryOperator::Equ => self.chunk().write_instr(OpCode::Equals, pos),
			BinaryOperator::Neq => self.chunk().write_instr(OpCode::NotEquals, pos),
			BinaryOperator::Lst => self.chunk().write_instr(OpCode::Lesser, pos),
			BinaryOperator::Lse => self.chunk().write_instr(OpCode::LesserEq, pos),
			BinaryOperator::Grt => self.chunk().write_instr(OpCode::Greater, pos),
			BinaryOperator::Gre => self.chunk().write_instr(OpCode::GreaterEq, pos),
			BinaryOperator::Typ => todo!(),
		}
		Ok(())
	}
	
	fn unary(&mut self, data: UnaryData, pos: SourcePos) -> Result<()> {
		data.expr.accept(self)?;
		match data.op {
			UnaryOperator::Not => self.chunk().write_instr(OpCode::Not, pos),
			UnaryOperator::Pos => self.chunk().write_instr(OpCode::Identity, pos),
			UnaryOperator::Neg => self.chunk().write_instr(OpCode::Negate, pos),
		}
		Ok(())
	}
	
	fn logic(&mut self, data: LogicData, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn grouping(&mut self, data: Box<Expression>, pos: SourcePos) -> Result<()> {
		data.accept(self)
	}
	
	fn variable(&mut self, data: Identifier, pos: SourcePos) -> Result<()> {
		self.write_var(OpCode::Load, data.get_id(), pos)
	}
	
	fn lambda(&mut self, data: LambdaData, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn call(&mut self, data: CallData, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn index(&mut self, data: IndexData, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn field(&mut self, data: FieldData, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn self_ref(&mut self, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn do_expr(&mut self, block: Block, pos: SourcePos) -> Result<()> {
		todo!(); // self.generate_block(block)
	}
	
	fn bind_expr(&mut self, data: BindData, pos: SourcePos) -> Result<()> {
		todo!()
	}

}

impl StmtVisitor<()> for ChunkGen {

	fn expr(&mut self, expr: Box<Expression>, pos: SourcePos) -> Result<()> {
		expr.accept(self)?;
		self.chunk.write_instr(OpCode::Pop, pos);
		Ok(())
	}
	
	fn declaration(&mut self, data: DeclarationData, pos: SourcePos) -> Result<()> {
		data.expr.accept(self)?;
		self.write_var(OpCode::Define, data.name.get_id(), pos)?;
		Ok(())
	}
	
	fn attr_declaration(&mut self, data: AttrDeclarationData, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn assignment(&mut self, data: AssignData, pos: SourcePos) -> Result<()> {
		data.expr.accept(self)?;
		if let ExprType::Variable(id) = data.head.typ {
			self.write_var(OpCode::Store, id.get_id(), pos)?;
		} else {
			return ErrorList::comp("Enviroment overflowed 255 variables".to_owned(), pos).err();
		}
		Ok(())
	}
	
	fn if_stmt(&mut self, data: IfData, pos: SourcePos) -> Result<()> {
		// data.cond.accept(self)?;
		// let anchor = self.chunk.write_jump(OpCode::FalseJump, pos);
		// self.generate_block(data.then_block)?;
		// self.chunk.patch_jump(anchor);
		// Ok(())
		todo!();
	}
	
	fn loop_stmt(&mut self, block: Block, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn break_stmt(&mut self, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn continue_stmt(&mut self, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn return_stmt(&mut self, expr: Box<Expression>, pos: SourcePos) -> Result<()> {
		expr.accept(self)?;
		self.chunk().write_instr(OpCode::Return, pos);
		Ok(())
	}
	
	fn scoped_stmt(&mut self, block: Block, pos: SourcePos) -> Result<()> {
		self.generate_block(block)
	}

}
