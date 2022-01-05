
#![allow(unused_variables)]

use std::collections::HashMap;

use crate::{ast::{expression::*, statement::*, identifier::Identifier, Block, module::Module}, utils::{source_pos::SourcePos, result::Result, wrap::Wrap}};

pub struct Optimizer;

impl Optimizer {
	
	pub fn optimize(&mut self, module: &mut Module) -> Result<()> {
		module.env = module.env.iter().map(|(k, s)| (k.clone(), s.clone().accept(self).unwrap())).collect();
		Ok(())
	}

	pub fn optimize_block(&mut self, block: Block) -> Result<Block> {
		let mut statements = Block::new();
		for stmt in block {
			statements.push(stmt.accept(self)?);
		}
		statements.wrap()
	}
	
}

impl ExprVisitor<Expression> for Optimizer {
	
	fn literal(&mut self, mut data: LiteralData, pos: SourcePos) -> Result<Expression> {
		match data {
			LiteralData::None => (),
			LiteralData::Str(_) => (),
			LiteralData::Num(_) => (),
			LiteralData::Bool(_) => (),
			LiteralData::Template(exprs) => {
				let mut oexprs = Vec::new();
				for expr in exprs { oexprs.push(expr.accept(self)?); }
				data = LiteralData::Template(oexprs);
			},
			LiteralData::List(exprs) => {
				let mut oexprs = Vec::new();
				for expr in exprs { oexprs.push(expr.accept(self)?); }
				data = LiteralData::List(oexprs);
			},
			LiteralData::Object(ref fields, _) => {
				let mut ofields = HashMap::new();
				for (key, expr) in fields.iter() { ofields.insert(key, expr.clone().accept(self)?); }
			},
			LiteralData::Error(expr) => {
				data = LiteralData::Error(expr.accept(self)?.wrap());
			},
		}
		ExprType::Literal(data).to_expr(pos).wrap()
	}
	
	fn binary(&mut self, mut data: BinaryData, pos: SourcePos) -> Result<Expression> {
		data.lhs = data.lhs.clone().accept(self)?.wrap();
		data.rhs = data.rhs.clone().accept(self)?.wrap();
		if let (ExprType::Literal(lhs), ExprType::Literal(rhs)) = (data.lhs.clone().typ, data.clone().rhs.typ) {
			match (lhs, rhs) {
				(LiteralData::Num(l), LiteralData::Num(r)) => match data.op {
					BinaryOperator::Add => return ExprType::Literal(LiteralData::Num(l + r)).to_expr(pos).wrap(),
					BinaryOperator::Sub => return ExprType::Literal(LiteralData::Num(l - r)).to_expr(pos).wrap(),
					BinaryOperator::Mul => return ExprType::Literal(LiteralData::Num(l * r)).to_expr(pos).wrap(),
					BinaryOperator::Div => return ExprType::Literal(LiteralData::Num(l / r)).to_expr(pos).wrap(),
					BinaryOperator::Rem => return ExprType::Literal(LiteralData::Num(l % r)).to_expr(pos).wrap(),
					BinaryOperator::Equ => return ExprType::Literal(LiteralData::Bool(l == r)).to_expr(pos).wrap(),
					BinaryOperator::Neq => return ExprType::Literal(LiteralData::Bool(l != r)).to_expr(pos).wrap(),
					BinaryOperator::Lst => return ExprType::Literal(LiteralData::Bool(l < r)).to_expr(pos).wrap(),
					BinaryOperator::Lse => return ExprType::Literal(LiteralData::Bool(l <= r)).to_expr(pos).wrap(),
					BinaryOperator::Grt => return ExprType::Literal(LiteralData::Bool(l > r)).to_expr(pos).wrap(),
					BinaryOperator::Gre => return ExprType::Literal(LiteralData::Bool(l >= r)).to_expr(pos).wrap(),
					BinaryOperator::Typ => (),
				},
				_ => (),
			}
		}
		ExprType::Binary(data).to_expr(pos).wrap()
	}
	
	fn unary(&mut self, mut data: UnaryData, pos: SourcePos) -> Result<Expression> {
		data.expr = data.expr.clone().accept(self)?.wrap();
		
		if let ExprType::Literal(lit) = data.expr.clone().typ {
			match lit {
				LiteralData::Num(n) => match data.op {
					UnaryOperator::Pos => return ExprType::Literal(LiteralData::Num(n)).to_expr(pos).wrap(),
					UnaryOperator::Neg => return ExprType::Literal(LiteralData::Num(-n)).to_expr(pos).wrap(),
					_ => (),
				},
				LiteralData::Bool(b) => match data.op {
					UnaryOperator::Not => return ExprType::Literal(LiteralData::Bool(!b)).to_expr(pos).wrap(),
					_ => (),
				},
				_ => (),
			}
		}
		
		ExprType::Unary(data).to_expr(pos).wrap()
	}
	
	fn logic(&mut self, data: LogicData, pos: SourcePos) -> Result<Expression> {
		ExprType::Logic(data).to_expr(pos).wrap()
	}
	
	fn grouping(&mut self, data: Box<Expression>, pos: SourcePos) -> Result<Expression> {
		let expr = data.accept(self)?;
		if let ExprType::Literal(_) = expr.typ {
			expr.wrap()
		} else {
			ExprType::Grouping(expr.wrap()).to_expr(pos).wrap()
		}
	}
	
	fn variable(&mut self, data: Identifier, pos: SourcePos) -> Result<Expression> {
		ExprType::Variable(data).to_expr(pos).wrap()
	}
	
	fn lambda(&mut self, mut data: LambdaData, pos: SourcePos) -> Result<Expression> {
		data.body = self.optimize_block(data.body)?;
		ExprType::Lambda(data).to_expr(pos).wrap()
	}
	
	fn call(&mut self, mut data: CallData, pos: SourcePos) -> Result<Expression> {
		data.calee = data.calee.accept(self)?.wrap();
		let mut args = Vec::new();
		for arg in data.args { args.push(arg.accept(self)?); }
		data.args = args;
		ExprType::Call(data).to_expr(pos).wrap()
	}
	
	fn index(&mut self, mut data: IndexData, pos: SourcePos) -> Result<Expression> {
		data.head = data.head.accept(self)?.wrap();
		data.index = data.index.accept(self)?.wrap();
		ExprType::Index(data).to_expr(pos).wrap()
	}
	
	fn field(&mut self, mut data: FieldData, pos: SourcePos) -> Result<Expression> {
		data.head = data.head.accept(self)?.wrap();
		ExprType::FieldGet(data).to_expr(pos).wrap()
	}
	
	fn self_ref(&mut self, pos: SourcePos) -> Result<Expression> {
		ExprType::SelfRef.to_expr(pos).wrap()
	}
	
	fn do_expr(&mut self, block: Block, pos: SourcePos) -> Result<Expression> {
		let block = self.optimize_block(block)?;
		ExprType::DoExpr(block).to_expr(pos).wrap()
	}
	
	fn bind_expr(&mut self, mut data: BindData, pos: SourcePos) -> Result<Expression> {
		data.expr = data.expr.accept(self)?.wrap();
		data.method = data.method.accept(self)?.wrap();
		ExprType::Binding(data).to_expr(pos).wrap()
	}
	
}

impl StmtVisitor<Statement> for Optimizer {
	
	fn expr(&mut self, expr: Box<Expression>, pos: SourcePos) -> Result<Statement> {
		let expr = expr.accept(self)?;
		StmtType::Expr(expr.wrap()).to_stmt(pos).wrap()
	}
	
	fn declaration(&mut self, mut data: DeclarationData, pos: SourcePos) -> Result<Statement> {
		data.expr = data.expr.accept(self)?.wrap();
		StmtType::Declaration(data).to_stmt(pos).wrap()
	}
	
	fn func_declaration(&mut self, mut data: FunctionData, pos: SourcePos) -> Result<Statement> {
		data.body = self.optimize_block(data.body)?;
		StmtType::FuncDeclaration(data).to_stmt(pos).wrap()
	}

	fn attr_declaration(&mut self, mut data: AttrDeclarationData, pos: SourcePos) -> Result<Statement> {
		let mut fields = HashMap::new();
		for (key, expr) in data.fields.iter() { fields.insert(key.clone(), expr.clone().accept(self)?); }
		data.fields = fields;

		let mut methods = Vec::new();
		for mut method in data.methods.clone() {
			method.body = self.optimize_block(method.body)?;
			methods.push(method);
		}
		data.methods = methods;

		StmtType::AttrDeclaration(data).to_stmt(pos).wrap()
	}
	
	fn assignment(&mut self, mut data: AssignData, pos: SourcePos) -> Result<Statement> {
		data.expr = data.expr.accept(self)?.wrap();
		StmtType::Assignment(data).to_stmt(pos).wrap()
	}
	
	fn if_stmt(&mut self, mut data: IfData, pos: SourcePos) -> Result<Statement> {
		data.cond = data.cond.accept(self)?.wrap();
		data.then_block = self.optimize_block(data.then_block)?;
		data.else_block = self.optimize_block(data.else_block)?;
		StmtType::If(data).to_stmt(pos).wrap()
	}
	
	fn loop_stmt(&mut self, mut block: Block, pos: SourcePos) -> Result<Statement> {
		block = self.optimize_block(block)?;
		StmtType::Loop(block).to_stmt(pos).wrap()
	}
	
	fn break_stmt(&mut self, pos: SourcePos) -> Result<Statement> {
		StmtType::Break.to_stmt(pos).wrap()
	}
	
	fn continue_stmt(&mut self, pos: SourcePos) -> Result<Statement> {
		StmtType::Break.to_stmt(pos).wrap()
	}
	
	fn return_stmt(&mut self, mut expr: Box<Expression>, pos: SourcePos) -> Result<Statement> {
		expr = expr.accept(self)?.wrap();
		StmtType::Return(expr).to_stmt(pos).wrap()
	}
	
	fn scoped_stmt(&mut self, mut block: Block, pos: SourcePos) -> Result<Statement> {
		block = self.optimize_block(block)?;
		StmtType::Scoped(block).to_stmt(pos).wrap()
	}
	
}
