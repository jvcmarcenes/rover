
use crate::utils::{result::Result, source_pos::SourcePos};

use super::{expression::Expression};

use self::StmtType::*;

#[derive(Debug, Clone)]
pub struct DeclarationData { pub name: String, pub expr: Box<Expression> }

#[derive(Clone, Debug)]
pub struct AssignData { pub name: String, pub l_pos: SourcePos, pub expr: Box<Expression> }

#[derive(Debug, Clone)]
pub enum StmtType {
	Writeline(Box<Expression>),
	Declaration(DeclarationData),
	Assignment(AssignData),
	Block(Vec<Statement>),
}

impl StmtType {
	pub fn to_stmt(self, pos: SourcePos) -> Statement {
		Statement { typ: self, pos }
	}
}

#[derive(Debug, Clone)]
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
			Block(data) => visitor.block(data, self.pos),
		}
	}
}

pub trait StmtVisitor<T> {
	fn writeline(&mut self, data: Box<Expression>, pos: SourcePos) -> Result<T>;
	fn declaration(&mut self, data: DeclarationData, pos: SourcePos) -> Result<T>;
	fn assignment(&mut self, data: AssignData, pos: SourcePos) -> Result<T>;
	fn block(&mut self, data: Vec<Statement>, pos: SourcePos) -> Result<T>;
}
