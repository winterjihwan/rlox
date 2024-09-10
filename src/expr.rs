use crate::{
    environment::Environment,
    errors::InterpretError,
    interpreter::Evaluation,
    token::{Literal, Token, TokenType},
};

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(ExprAssign),
    Binary(ExprBinary),
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

    pub fn evaluate(
        &mut self,
        environment: &mut Environment,
    ) -> Result<Evaluation, InterpretError> {
        match self {
            Self::Assign(expr) => {
                let value = expr.value.evaluate(environment)?;
                environment.assign(&expr.name, value.clone())?;
                Ok(value)
            }
            Self::Literal(expr_literal) => Ok(expr_literal.literal.clone().into()),
            Self::Grouping(expr_grouping) => expr_grouping.expr.evaluate(environment),
            Self::Logical(expr) => {
                let left = expr.left.evaluate(environment)?;

                if let Evaluation::bool(b) = left {
                    if (expr.operator.token_type == TokenType::Or && b)
                        || (expr.operator.token_type == TokenType::And && !b)
                    {
                        return Ok(left);
                    }
                };

                expr.right.evaluate(environment)
            }
            Self::Unary(expr_unary) => {
                let right = expr_unary.right.evaluate(environment)?;
                Self::evaluate_unary(right, expr_unary.operator.token_type)
            }
            Self::Binary(expr_binary) => {
                let left = expr_binary.left.evaluate(environment)?;
                let right = expr_binary.right.evaluate(environment)?;
                Self::evaluate_binary(left, right, expr_binary.operator.token_type)
            }
            Self::Var(expr) => environment.get(&expr.name),
        }
    }

    fn evaluate_unary(
        right: Evaluation,
        operator_type: TokenType,
    ) -> Result<Evaluation, InterpretError> {
        match (&right, operator_type) {
            (Evaluation::f64(n), TokenType::Minus) => Ok(Evaluation::f64(-n)),
            (Evaluation::bool(b), TokenType::Bang) => Ok(Evaluation::bool(!b)),
            (Evaluation::nil(()), TokenType::Bang) => Ok(Evaluation::bool(false)),
            _ => Err(InterpretError::EvaluateUnaryFail {
                right_evaluation: right,
                operator_type,
            }),
        }
    }

    fn evaluate_binary(
        left: Evaluation,
        right: Evaluation,
        operator_type: TokenType,
    ) -> Result<Evaluation, InterpretError> {
        let evaluation = match operator_type {
            TokenType::Plus => (left + right)?,
            TokenType::Minus | TokenType::Slash | TokenType::Star => {
                if let (Evaluation::f64(n1), Evaluation::f64(n2)) = (&left, &right) {
                    match operator_type {
                        TokenType::Minus => Evaluation::f64(n1 - n2),
                        TokenType::Slash => Evaluation::f64(n1 / n2),
                        TokenType::Star => Evaluation::f64(n1 * n2),
                        _ => return Err(Self::binary_fail(left, operator_type, right)),
                    }
                } else {
                    return Err(Self::binary_fail(left, operator_type, right));
                }
            }
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => {
                if let (Evaluation::f64(n1), Evaluation::f64(n2)) = (&left, &right) {
                    match operator_type {
                        TokenType::Greater => Evaluation::bool(n1 > n2),
                        TokenType::GreaterEqual => Evaluation::bool(n1 >= n2),
                        TokenType::Less => Evaluation::bool(n1 < n2),
                        TokenType::LessEqual => Evaluation::bool(n1 <= n2),
                        _ => return Err(Self::binary_fail(left, operator_type, right)),
                    }
                } else {
                    return Err(Self::binary_fail(left, operator_type, right));
                }
            }
            TokenType::BangEqual => Evaluation::bool(left != right),
            TokenType::EqualEqual => Evaluation::bool(left == right),
            _ => return Err(Self::binary_fail(left, operator_type, right)),
        };

        Ok(evaluation)
    }

    fn binary_fail(
        left: Evaluation,
        operator_type: TokenType,
        right: Evaluation,
    ) -> InterpretError {
        InterpretError::EvaluateBinaryFail {
            left_evaluation: left,
            operator_type,
            right_evaluation: right,
        }
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
