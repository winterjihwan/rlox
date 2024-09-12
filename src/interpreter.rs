use std::{
    fmt::{Debug, Display},
    io, mem,
    ops::Add,
    sync::{Arc, Mutex},
};

use crate::{
    environment::Environment,
    errors::InterpretError,
    expr::Expr,
    stmt::{Stmt, StmtFunction},
    token::{Literal, TokenType},
};

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub enum Evaluation {
    string(String),
    f64(f64),
    bool(bool),
    nil(()),
    callable(Arc<Mutex<dyn Callable>>),
}

impl Debug for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluation::string(string) => write!(f, "{string}"),
            Evaluation::f64(f64) => write!(f, "{f64}"),
            Evaluation::bool(bool) => write!(f, "{bool}"),
            Evaluation::nil(()) => write!(f, "nil"),
            Evaluation::callable(_) => write!(f, "fn <>"),
        }
    }
}

impl From<Evaluation> for Result<Arc<Mutex<dyn Callable>>, InterpretError> {
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

pub trait Callable {
    fn arity(&self) -> u8;
    fn call(
        &mut self,
        interpreter: Interpreter,
        arguments: Vec<Evaluation>,
    ) -> Result<Option<Evaluation>, InterpretError>;
}

pub struct NativeFunction {
    pub arity: u8,
    pub fun: Option<Box<dyn FnMut(Interpreter, Vec<Evaluation>) -> Option<Evaluation>>>,
}

impl NativeFunction {
    pub fn new(fun: Box<dyn FnMut(Interpreter, Vec<Evaluation>) -> Option<Evaluation>>) -> Self {
        Self {
            arity: 0,
            fun: Some(fun),
        }
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(
        &mut self,
        interpreter: Interpreter,
        arguments: Vec<Evaluation>,
    ) -> Result<Option<Evaluation>, InterpretError> {
        Ok((self.fun.as_mut().unwrap())(interpreter, arguments))
    }
}

#[derive(Clone)]
pub struct RloxFunction {
    pub arity: u8,
    pub declaration: StmtFunction,
}

impl RloxFunction {
    pub fn new(arity: u8, declaration: StmtFunction) -> Self {
        Self { arity, declaration }
    }
}

impl Callable for RloxFunction {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(
        &mut self,
        mut interpreter: Interpreter,
        arguments: Vec<Evaluation>,
    ) -> Result<Option<Evaluation>, InterpretError> {
        let mut environment = Environment::new();
        environment.enclosing = Some(Box::new(interpreter.globals.clone()));

        println!("environment, {environment:#?}");

        let mut declaration = self.declaration.clone();
        println!("declaration params len, {:#?}", declaration.params.len());

        for i in 0..declaration.params.len() {
            println!(
                "declarations params {}: {}, arguments {}: {:#?}",
                i,
                declaration.params[i].lexeme.to_string(),
                i,
                arguments[i]
            );
            environment.define(
                declaration.params[i].lexeme.to_string(),
                Some(arguments[i].clone()),
            )
        }

        let a = interpreter.stmt_execute_block(&mut declaration.body, environment)?;
        println!("aaa: {a:#?}");

        Ok(a)
    }
}

impl Debug for RloxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.clone().name.lexeme)
    }
}

impl Display for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluation::string(string) => write!(f, "{string}"),
            Evaluation::f64(f64) => write!(f, "{f64:.2}"),
            Evaluation::bool(bool) => write!(f, "{bool}"),
            Evaluation::nil(()) => write!(f, "nil"),
            Evaluation::callable(fun) => write!(f, "fn <>"),
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

#[derive(Clone)]
pub struct Interpreter {
    pub globals: Environment,
    pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();

        let clock_closure = |_: Interpreter, _: Vec<Evaluation>| {
            let call = || {
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as f64;

                Evaluation::f64(time)
            };

            let display = || Evaluation::string(String::from("native <fn>"));

            Some(Evaluation::nil(()))
        };

        let clock_callable = NativeFunction::new(Box::new(clock_closure));

        globals.define(
            "clock".to_string(),
            Some(Evaluation::callable(Arc::new(Mutex::new(clock_callable)))),
        );

        Self {
            globals,
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> io::Result<()> {
        statements
            .into_iter()
            .try_for_each(|mut stmt| self.stmt_execute(&mut stmt).and_then(|_| Ok(())))?;

        Ok(())
    }

    pub fn stmt_execute(&mut self, stmt: &mut Stmt) -> Result<Option<Evaluation>, InterpretError> {
        self.stmt_evaluate(stmt)
    }

    pub fn stmt_evaluate(&mut self, stmt: &mut Stmt) -> Result<Option<Evaluation>, InterpretError> {
        match stmt {
            Stmt::Return(stmt) => {
                let mut value = None;
                if let Some(val) = &stmt.value {
                    value = Some(self.evaluate(val.clone())?);
                }
                Ok(value)
            }
            Stmt::While(stmt) => {
                while let Evaluation::bool(true) = self.evaluate(stmt.condition.clone())? {
                    self.stmt_execute(&mut stmt.body)?;
                }
                Ok(None)
            }
            Stmt::If(stmt) => {
                if let Evaluation::bool(truthy) = self.evaluate(stmt.condition.clone())? {
                    if truthy {
                        self.stmt_execute(&mut stmt.then_branch)?;
                    } else if let Some(_) = stmt.else_branch {
                        self.stmt_execute(&mut stmt.else_branch.clone().unwrap())?;
                    }
                };
                Ok(None)
            }
            Stmt::Function(stmt) => {
                let function = RloxFunction::new(stmt.params.len() as u8, stmt.clone());

                self.environment.define(
                    stmt.name.lexeme.to_string(),
                    Some(Evaluation::callable(Arc::new(Mutex::new(function)))),
                );

                Ok(None)
            }
            Stmt::Block(stmt) => {
                let a = self.stmt_execute_block(&mut stmt.statements, Environment::new())?;
                Ok(a)
            }
            Stmt::Expression(expr) => {
                let evl = self.evaluate(expr.clone())?;
                println!("EVLEVLEVLEVLEVL, {evl:#?}");
                Ok(None)
            }
            Stmt::Print(value) => {
                let value = self.evaluate(value.clone())?;
                println!("{value}");
                Ok(None)
            }
            Stmt::Var(var) => {
                let a = var
                    .initializer
                    .as_mut()
                    .map(|expr| self.evaluate(expr.clone()))
                    .transpose()?;

                self.environment.define(var.name.lexeme.to_string(), a);
                Ok(None)
            }
        }
    }

    pub fn stmt_execute_block(
        &mut self,
        statements: &mut Vec<Stmt>,
        environment: Environment,
    ) -> Result<Option<Evaluation>, InterpretError> {
        let previous = mem::replace(&mut self.environment, environment);
        self.environment.enclosing = Some(Box::new(previous.clone()));

        let a = statements
            .iter_mut()
            .find_map(|stmt| match self.stmt_execute(stmt) {
                Ok(Some(eval)) => Some(Ok(eval)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            })
            .transpose()?;

        println!("Executed block for function call, {a:#?}");

        self.environment = *self.environment.enclosing.take().unwrap();

        Ok(a)
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Evaluation, InterpretError> {
        match expr {
            Expr::Assign(expr) => {
                let value = self.evaluate(*expr.value)?;
                self.environment.assign(&expr.name, value.clone())?;
                Ok(value)
            }
            Expr::Literal(expr) => Ok(expr.literal.clone().into()),
            Expr::Call(expr) => {
                let callee = self.evaluate(*expr.callee.clone())?;
                println!("callee... {callee:#?}");

                println!("My name is {expr:?} trying to acquire function lock");
                let mut arguments = Vec::new();
                expr.arguments.into_iter().try_for_each(|arg| {
                    arguments.push(self.evaluate(arg)?);
                    Ok::<(), InterpretError>(())
                })?;

                let function: Arc<Mutex<dyn Callable>> = Result::from(callee)?;

                let arity = {
                    let function_acq = function.lock().unwrap();
                    function_acq.arity().into()
                };

                if arguments.len() != arity {
                    println!("haha");
                    return Err(InterpretError::RuntimeError {
                        err: format!(
                            "Expected '{}' arguments but got '{}'",
                            arity,
                            arguments.len()
                        ),
                    });
                }

                println!("Calling...");
                let a = {
                    let mut function_acq = function.lock().unwrap();
                    function_acq.call(self.clone(), arguments)?
                };

                let a = match a {
                    Some(eval) => eval,
                    None => Evaluation::nil(()),
                };

                println!("A: {a:#?}");
                Ok(a)
            }
            Expr::Grouping(expr_grouping) => self.evaluate(*expr_grouping.expr),
            Expr::Logical(expr) => {
                let left = self.evaluate(*expr.left)?;

                if let Evaluation::bool(b) = left {
                    if (expr.operator.token_type == TokenType::Or && b)
                        || (expr.operator.token_type == TokenType::And && !b)
                    {
                        return Ok(left);
                    }
                };

                self.evaluate(*expr.right)
            }
            Expr::Unary(expr_unary) => {
                let right = self.evaluate(*expr_unary.right)?;
                Self::evaluate_unary(right, expr_unary.operator.token_type)
            }
            Expr::Binary(expr_binary) => {
                let left = self.evaluate(*expr_binary.left)?;
                let right = self.evaluate(*expr_binary.right)?;
                Self::evaluate_binary(left, right, expr_binary.operator.token_type)
            }
            Expr::Var(expr) => self.environment.get(&expr.name),
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
