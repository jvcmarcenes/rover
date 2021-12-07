
use crate::{utils::{result::Result, wrap::Wrap, source_pos::SourcePos}, ast::{statement::*, expression::*, identifier::Identifier}};

use super::{chunk::Chunk, opcode::OpCode, disassembler::Disassembler};

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

	pub fn generate(&mut self, code: Block) -> Result<Chunk> {
		self.generate_block(code)?;
		if cfg!(feature = "print_code") { Disassembler::new(self.chunk.clone()).disassemble("code"); }
		self.chunk.clone().wrap()
	}
}

impl ExprVisitor<()> for ChunkGen {
	fn literal(&mut self, data: LiteralData, pos: SourcePos) -> Result<()> {
		match data {
			LiteralData::None => todo!(),
			LiteralData::Str(_) => todo!(),
			LiteralData::Num(n) => self.chunk().write_const(n, pos),
			LiteralData::Bool(_) => todo!(),
			LiteralData::Template(_) => todo!(),
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
			BinaryOperator::Equ => todo!(),
			BinaryOperator::Neq => todo!(),
			BinaryOperator::Lst => todo!(),
			BinaryOperator::Lse => todo!(),
			BinaryOperator::Grt => todo!(),
			BinaryOperator::Gre => todo!(),
			BinaryOperator::Typ => todo!(),
		}
		Ok(())
	}
	
	fn unary(&mut self, data: UnaryData, pos: SourcePos) -> Result<()> {
		data.expr.accept(self)?;
		match data.op {
			UnaryOperator::Not => todo!(),
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
		todo!()
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
		todo!()
	}
	
	fn bind_expr(&mut self, data: BindData, pos: SourcePos) -> Result<()> {
		todo!()
	}
}

impl StmtVisitor<()> for ChunkGen {
	fn expr(&mut self, expr: Box<Expression>, pos: SourcePos) -> Result<()> {
		expr.accept(self)
	}
	
	fn declaration(&mut self, data: DeclarationData, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn attr_declaration(&mut self, data: AttrDeclarationData, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn assignment(&mut self, data: AssignData, pos: SourcePos) -> Result<()> {
		todo!()
	}
	
	fn if_stmt(&mut self, data: IfData, pos: SourcePos) -> Result<()> {
		todo!()
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
		self.chunk().write_instr(OpCode::Return, pos);
		Ok(())
	}
	
	fn scoped_stmt(&mut self, block: Block, pos: SourcePos) -> Result<()> {
		self.generate_block(block)
	}
}
