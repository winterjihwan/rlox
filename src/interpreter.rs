use std::{fmt::Display, io, ops::Add};

use crate::{
    errors::InterpretError,
    expr::Expr,
    token::{Literal, TokenType},
};

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Evaluation {
    string(String),
    f64(f64),
    bool(bool),
    nil(()),
}

impl Display for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluation::string(string) => write!(f, "{string}"),
            Evaluation::f64(f64) => write!(f, "{f64:.2}"),
            Evaluation::bool(bool) => write!(f, "{bool}"),
            Evaluation::nil(()) => write!(f, "nil"),
        }
    }
}

impl Add for Evaluation {
    type Output = Result<Evaluation, InterpretError>;

    fn add(self, rhs: Self) -> Self::Output {
        let evaluation = match (self, rhs) {
            (Self::string(mut s1), Self::string(s2)) => Evaluation::string({
                s1.push_str(&s2);
                s1
            }),
            (Self::f64(f1), Self::f64(f2)) => Evaluation::f64(f1 + f2),
            (lhs, rhs) => return Err(InterpretError::EvaluationAddOverloaderError { lhs, rhs }),
        };

        Ok(evaluation)
    }
}

impl PartialEq for Evaluation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::string(s1), Self::string(s2)) => s1 == s2,
            (Self::f64(f1), Self::f64(f2)) => f1 == f2,
            (Self::bool(b1), Self::bool(b2)) => b1 == b2,
            (Self::nil(()), Self::nil(())) => true,
            _ => false,
        }
    }
}

impl From<Literal> for Evaluation {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::string(string) => Self::string(string),
            Literal::f64(f64) => Self::f64(f64),
            Literal::bool(bool) => Self::bool(bool),
            Literal::nil(nil) => Self::nil(nil),
        }
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn interpret(expr: Expr) -> io::Result<()> {
        let value = Interpreter::evaluate(expr).map_err(|err| {
            println!("INTERPRET ERROR: {}", err);
            err
        })?;

        println!("{value}");

        Ok(())
    }

    fn evaluate(expr: Expr) -> Result<Evaluation, InterpretError> {
        match expr {
            Expr::Literal(expr_literal) => Ok(expr_literal.literal.into()),
            Expr::Grouping(expr_grouping) => Interpreter::evaluate(*expr_grouping.expr),
            Expr::Unary(expr_unary) => {
                let right = Interpreter::evaluate(*expr_unary.right)?;
                Interpreter::evaluate_unary(right, expr_unary.operator.token_type)
            }
            Expr::Binary(expr_binary) => {
                let left = Interpreter::evaluate(*expr_binary.left)?;
                let right = Interpreter::evaluate(*expr_binary.right)?;
                Interpreter::evaluate_binary(left, right, expr_binary.operator.token_type)
            }
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
                        _ => return Err(Interpreter::binary_fail(left, operator_type, right)),
                    }
                } else {
                    return Err(Interpreter::binary_fail(left, operator_type, right));
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
                        _ => return Err(Interpreter::binary_fail(left, operator_type, right)),
                    }
                } else {
                    return Err(Interpreter::binary_fail(left, operator_type, right));
                }
            }
            TokenType::BangEqual => Evaluation::bool(left != right),
            TokenType::EqualEqual => Evaluation::bool(left == right),
            _ => return Err(Interpreter::binary_fail(left, operator_type, right)),
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
