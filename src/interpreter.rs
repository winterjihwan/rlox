use std::{
    fmt::{Debug, Display},
    io, mem,
    ops::Add,
    process::exit,
};

use crate::{environment::Environment, errors::InterpretError, stmt::Stmt, token::Literal};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Evaluation {
    string(String),
    f64(f64),
    bool(bool),
    nil(()),
    callable(RloxCallable),
}

impl Clone for Evaluation {
    fn clone(&self) -> Self {
        match self {
            Evaluation::string(string) => Evaluation::string(string.clone()),
            Evaluation::f64(f64) => Evaluation::f64(f64.clone()),
            Evaluation::bool(bool) => Evaluation::bool(bool.clone()),
            Evaluation::nil(()) => Evaluation::nil(()),
            Evaluation::callable(fun) => {
                println!("Invalid clone operation to fn, {fun:#?}");
                exit(16)
            }
        }
    }
}

impl From<Evaluation> for Result<RloxCallable, InterpretError> {
    fn from(value: Evaluation) -> Self {
        if let Evaluation::callable(rlox_callable) = value {
            Ok(rlox_callable)
        } else {
            Err(InterpretError::CastError {
                expect: "Callable".to_string(),
                actual: value.to_string(),
            })
        }
    }
}

pub struct RloxCallable {
    pub arity: u8,
    pub name: String,
    pub fun: Box<dyn FnMut(&mut Environment, Vec<Evaluation>) -> Evaluation>,
}

impl RloxCallable {
    pub fn new(
        arity: u8,
        name: String,
        fun: Box<dyn FnMut(&mut Environment, Vec<Evaluation>) -> Evaluation>,
    ) -> Self {
        Self { arity, name, fun }
    }
}

impl Debug for RloxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn<> {}", self.name)
    }
}

impl Display for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluation::string(string) => write!(f, "{string}"),
            Evaluation::f64(f64) => write!(f, "{f64:.2}"),
            Evaluation::bool(bool) => write!(f, "{bool}"),
            Evaluation::nil(()) => write!(f, "nil"),
            Evaluation::callable(fun) => write!(f, "fn <{fun:#?}>"),
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
    pub globals: Environment,
    pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();

        let clock_closure = |_: &mut Environment, _: Vec<Evaluation>| {
            let call = || {
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as f64;

                Evaluation::f64(time)
            };

            let display = || Evaluation::string(String::from("native <fn>"));

            Evaluation::nil(())
        };

        let clock_callable = RloxCallable::new(0, "clock".to_string(), Box::new(clock_closure));
        globals.define(
            "clock".to_string(),
            Some(Evaluation::callable(clock_callable)),
        );

        Self {
            globals,
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> io::Result<()> {
        statements
            .into_iter()
            .try_for_each(|mut stmt| self.execute(&mut stmt))?;

        Ok(())
    }

    pub fn execute(&mut self, stmt: &mut Stmt) -> Result<(), InterpretError> {
        self.evaluate(stmt)
    }

    pub fn evaluate(&mut self, stmt: &mut Stmt) -> Result<(), InterpretError> {
        match stmt {
            Stmt::While(stmt) => {
                while let Evaluation::bool(true) = stmt.condition.evaluate(&mut self.environment)? {
                    self.execute(&mut stmt.body)?;
                }
                Ok(())
            }
            Stmt::If(stmt) => {
                if let Evaluation::bool(truthy) = stmt.condition.evaluate(&mut self.environment)? {
                    if truthy {
                        self.execute(&mut stmt.then_branch)?
                    } else if let Some(_) = stmt.else_branch {
                        self.execute(&mut stmt.else_branch.clone().unwrap())?
                    }
                };
                Ok(())
            }
            Stmt::Function(stmt) => {
                unimplemented!();
                Ok(())
            }
            Stmt::Block(stmt) => {
                self.execute_block(&mut stmt.statements, Environment::new())?;
                Ok(())
            }
            Stmt::Expression(expr) => {
                let evl = expr.evaluate(&mut self.environment)?;
                Ok(())
            }
            Stmt::Print(value) => {
                let value = value.evaluate(&mut self.environment)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Var(var) => {
                let a = var
                    .initializer
                    .as_mut()
                    .map(|expr| expr.evaluate(&mut self.environment))
                    .transpose()?;

                self.environment.define(var.name.lexeme.to_string(), a);
                Ok(())
            }
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &mut Vec<Stmt>,
        environment: Environment,
    ) -> Result<(), InterpretError> {
        let previous = mem::replace(&mut self.environment, environment);
        self.environment.enclosing = Some(Box::new(previous.clone()));

        statements
            .iter_mut()
            .try_for_each(|mut stmt| self.execute(&mut stmt))?;

        self.environment = *self.environment.enclosing.take().unwrap();

        Ok(())
    }
}
