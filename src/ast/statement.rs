
use crate::utils::{result::Result, source_pos::SourcePos};

use super::expression::Expression;

use self::StmtType::*;

pub type Block = Vec<Statement>;

#[derive(Clone)]
pub struct DeclarationData { pub name: String, pub expr: Box<Expression> }
#[derive(Clone)]
pub struct AssignData { pub name: String, pub l_pos: SourcePos, pub expr: Box<Expression> }
#[derive(Clone)]
pub struct IfData { pub cond: Box<Expression>, pub then_block: Block, pub else_block: Block }

#[derive(Clone)]
pub enum StmtType {
	Writeline(Box<Expression>),
	Declaration(DeclarationData),
	Assignment(AssignData),
	If(IfData),
	Loop(Block),
	Break, Continue,
}

impl StmtType {
	pub fn to_stmt(self, pos: SourcePos) -> Statement {
		Statement { typ: self, pos }
	}
}

#[derive(Clone)]
pub struct Statement {
	pub typ: StmtType,
	pub pos: SourcePos,
}

impl Statement {
	pub fn new(typ: StmtType, pos: SourcePos) -> Self {
		Self { typ, pos }
	}
}

impl Statement {
	pub fn accept<T>(self, visitor: &mut dyn StmtVisitor<T>) -> Result<T> {
		match self.typ {
			Writeline(expr) => visitor.writeline(expr, self.pos),
			Declaration(data) => visitor.declaration(data, self.pos),
			Assignment(data) => { let l_pos = data.l_pos; visitor.assignment(data, l_pos) },
			If(data) => visitor.if_stmt(data, self.pos),
			Loop(block) => visitor.loop_stmt(block, self.pos),
			Break => visitor.break_stmt(self.pos),
			Continue => visitor.continue_stmt(self.pos),
		}
	}
}

pub trait StmtVisitor<T> {
	fn writeline(&mut self, data: Box<Expression>, pos: SourcePos) -> Result<T>;
	fn declaration(&mut self, data: DeclarationData, pos: SourcePos) -> Result<T>;
	fn assignment(&mut self, data: AssignData, pos: SourcePos) -> Result<T>;
	fn if_stmt(&mut self, data: IfData, pos: SourcePos) -> Result<T>;
	fn loop_stmt(&mut self, block: Block, pos: SourcePos) -> Result<T>;
	fn break_stmt(&mut self, pos: SourcePos) -> Result<T>;
	fn continue_stmt(&mut self, pos: SourcePos) -> Result<T>;
}
