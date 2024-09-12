use crate::{expr::Expr, token::Token};

type StmtExpression = Expr;
type StmtPrint = Expr;

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(StmtBlock),
    Expression(StmtExpression),
    Print(StmtPrint),
    Return(StmtReturn),
    Var(StmtVar),
    While(StmtWhile),
    If(StmtIf),
    Function(StmtFunction),
}

impl Stmt {}

#[derive(Debug, Clone)]
pub struct StmtBlock {
    pub statements: Vec<Stmt>,
}

impl StmtBlock {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}

#[derive(Debug, Clone)]
pub struct StmtWhile {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

impl StmtWhile {
    pub fn new(condition: Expr, body: Stmt) -> Self {
        Self {
            condition,
            body: Box::new(body),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StmtReturn {
    pub keyword: Token,
    pub value: Option<Expr>,
}

impl StmtReturn {
    pub fn new(keyword: Token, value: Option<Expr>) -> Self {
        Self { keyword, value }
    }
}

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
pub struct StmtIf {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

impl StmtIf {
    pub fn new(condition: Expr, then_branch: Stmt, else_branch: Option<Stmt>) -> Self {
        Self {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(|stmt| Box::new(stmt)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StmtFunction {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

impl StmtFunction {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Self {
        Self { name, params, body }
    }
}
