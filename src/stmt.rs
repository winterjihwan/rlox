use crate::{expr::Expr, token::Token};

type StmtExpression = Expr;
type StmtPrint = Expr;

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(StmtBlock),
    Expression(StmtExpression),
    Print(StmtPrint),
    Var(StmtVar),
}

impl Stmt {}

#[derive(Debug, Clone)]
pub struct StmtVar {
    pub name: Token,
    pub initializer: Option<Expr>,
}
impl StmtVar {
    pub fn new(name: Token, initializer: Option<Expr>) -> Self {
        Self { name, initializer }
    }
}

#[derive(Debug, Clone)]
pub struct StmtBlock {
    pub statements: Vec<Stmt>,
}

impl StmtBlock {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}
