use crate::token::{Literal, Token};

#[derive(Debug)]
pub enum Expr {
    Binary(ExprBinary),
    Grouping(ExprGrouping),
    Literal(ExprLiteral),
    Unary(ExprUnary),
}

impl Expr {
    pub fn parenthesize(name: String, exprs: Vec<Expr>) -> String {
        let mut exprs_string = String::new();
        exprs.into_iter().for_each(|expr| {
            exprs_string.push_str(" ");

            let child_expr = match expr {
                Self::Binary(expr) => {
                    Expr::parenthesize(expr.operator.lexeme, vec![*expr.left, *expr.right])
                }
                Self::Grouping(expr) => Expr::parenthesize("group".to_string(), vec![*expr.expr]),
                Self::Literal(expr) => Expr::parenthesize(expr.literal.to_string(), Vec::new()),
                Self::Unary(expr) => Expr::parenthesize(expr.operator.lexeme, vec![*expr.right]),
            };
            exprs_string.push_str(&child_expr);
        });

        format!("({}{})", name, exprs_string)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ExprLiteral {
    pub literal: Literal,
}

impl ExprLiteral {
    pub fn new(literal: Literal) -> Self {
        Self { literal }
    }
}

#[derive(Debug)]
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
