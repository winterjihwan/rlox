use std::io;

use thiserror::Error;

use crate::{
    interpreter::Evaluation,
    token::{Token, TokenType},
};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("I/O fail, err: {err}")]
    IoFail { err: String },

    #[error("{} at end", token.line)]
    ParseEOF { token: Token },

    #[error("Invalid assignment target, {:#?}", token)]
    InvalidAssignmentTarget { token: Token },

    #[error("{} at '{}' {}", token.line, token.lexeme,  message)]
    ParseFail { token: Token, message: String },
}

impl From<ParseError> for io::Error {
    fn from(error: ParseError) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("{:#?}", error))
    }
}

#[derive(Error, Debug)]
pub enum InterpretError {
    #[error("{err}")]
    RuntimeError { err: String },

    #[error("Undefined variable '{lexeme}'.")]
    UndefinedVariable { lexeme: String },

    #[error("Evaluation add overloader error, lhs: {lhs:#?}, rhs: {rhs:#?}")]
    EvaluationAddOverloaderError { lhs: Evaluation, rhs: Evaluation },

    #[error(
        "Evaluate unary fail, right evaluation: {right_evaluation:#?}, operator type: {operator_type:#?}"
    )]
    EvaluateUnaryFail {
        right_evaluation: Evaluation,
        operator_type: TokenType,
    },

    #[error(
        "Evaluate binary fail, left evaluation: {left_evaluation:#?}, operator type: {operator_type:#?}, right evaluation: {right_evaluation:#?}"
    )]
    EvaluateBinaryFail {
        left_evaluation: Evaluation,
        operator_type: TokenType,
        right_evaluation: Evaluation,
    },
}

impl From<InterpretError> for io::Error {
    fn from(error: InterpretError) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("{:#?}", error))
    }
}
