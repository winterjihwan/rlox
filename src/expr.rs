use crate::token::{Literal, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(ExprAssign),
    Binary(ExprBinary),
    Call(ExprCall),
    Grouping(ExprGrouping),
    Literal(ExprLiteral),
    Logical(ExprLogical),
    Unary(ExprUnary),
    Var(ExprVar),
}

impl Expr {
    pub fn parenthesize(name: String, exprs: Vec<Expr>) -> String {
        let mut exprs_string = String::new();
        exprs.into_iter().for_each(|expr| {
            exprs_string.push_str(" ");

            let child_expr = match expr {
                Self::Assign(expr) => unimplemented!(),
                Self::Binary(expr) => {
                    Expr::parenthesize(expr.operator.lexeme, vec![*expr.left, *expr.right])
                }
                Self::Call(expr_literal) => unimplemented!(),
                Self::Grouping(expr) => Expr::parenthesize("group".to_string(), vec![*expr.expr]),
                Self::Literal(expr) => Expr::parenthesize(expr.literal.to_string(), Vec::new()),
                Self::Logical(expr) => unimplemented!(),
                Self::Unary(expr) => Expr::parenthesize(expr.operator.lexeme, vec![*expr.right]),
                Self::Var(expr) => unimplemented!(),
            };
            exprs_string.push_str(&child_expr);
        });

        format!("({}{})", name, exprs_string)
    }
}

#[derive(Debug, Clone)]
pub struct ExprAssign {
    pub name: Token,
    pub value: Box<Expr>,
}

impl ExprAssign {
    pub fn new(name: Token, value: Expr) -> Self {
        Self {
            name,
            value: Box::new(value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExprBinary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl ExprBinary {
    pub fn new(left_expr: Expr, token: Token, right_expr: Expr) -> Self {
        Self {
            left: Box::new(left_expr),
            operator: token,
            right: Box::new(right_expr),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExprCall {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

impl ExprCall {
    pub fn new(callee: Expr, paren: Token, arguments: Vec<Expr>) -> Self {
        Self {
            callee: Box::new(callee),
            paren,
            arguments,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExprGrouping {
    pub expr: Box<Expr>,
}

impl ExprGrouping {
    pub fn new(expr: Expr) -> Self {
        Self {
            expr: Box::new(expr),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExprLiteral {
    pub literal: Literal,
}

impl ExprLiteral {
    pub fn new(literal: Literal) -> Self {
        Self { literal }
    }
}

#[derive(Debug, Clone)]
pub struct ExprLogical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl ExprLogical {
    pub fn new(left: Expr, operator: Token, right: Expr) -> Self {
        Self {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExprUnary {
    pub operator: Token,
    pub right: Box<Expr>,
}

impl ExprUnary {
    pub fn new(token: Token, right_expr: Expr) -> Self {
        Self {
            operator: token,
            right: Box::new(right_expr),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExprVar {
    pub name: Token,
}

impl ExprVar {
    pub fn new(name: Token) -> Self {
        Self { name }
    }
}
