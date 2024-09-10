use std::{fmt::Display, io, ops::Add};

use crate::{environment::Environment, errors::InterpretError, stmt::Stmt, token::Literal};

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

pub struct Interpreter {
    pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> io::Result<()> {
        statements
            .into_iter()
            .try_for_each(|mut stmt| stmt.execute(&mut self.environment))?;

        Ok(())
    }
}
