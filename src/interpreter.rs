use std::{
    fmt::{Debug, Display},
    io,
    ops::Add,
    process::exit,
};

use crate::{
    environment::Environment,
    errors::InterpretError,
    expr::Expr,
    stmt::{Stmt, StmtFunction},
    token::{Literal, TokenType},
};

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Evaluation {
    string(String),
    f64(f64),
    bool(bool),
    nil(()),
    callable(Box<dyn Callable>),
}

impl From<Evaluation> for Result<Box<dyn Callable>, InterpretError> {
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

pub trait Callable: CallableClone {
    fn arity(&self) -> u8;
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        callee: String,
        arguments: Vec<Evaluation>,
    ) -> Result<Option<Evaluation>, InterpretError>;
    fn display(&self) -> String;
}

pub trait CallableClone {
    fn clone_box(&self) -> Box<dyn Callable>;
}

impl<T> CallableClone for T
where
    T: 'static + Callable + Clone,
{
    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Box<dyn Callable> {
        self.clone_box()
    }
}

impl Debug for Box<dyn Callable> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.display())
    }
}

#[derive(Clone)]
pub struct NativeFunction {
    pub arity: u8,
    pub fn_name: String,
    pub fun: Option<fn(&mut Interpreter, Vec<Evaluation>) -> Option<Evaluation>>,
}

impl NativeFunction {
    pub fn new(
        fn_name: String,
        fun: fn(&mut Interpreter, Vec<Evaluation>) -> Option<Evaluation>,
    ) -> Self {
        Self {
            arity: 0,
            fn_name,
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
        interpreter: &mut Interpreter,
        _callee: String,
        arguments: Vec<Evaluation>,
    ) -> Result<Option<Evaluation>, InterpretError> {
        Ok((self.fun.as_mut().unwrap())(interpreter, arguments))
    }

    fn display(&self) -> String {
        self.fn_name.to_string()
    }
}

#[derive(Clone)]
pub struct RloxFunction {
    pub arity: u8,
    pub fn_name: String,
    pub declaration: StmtFunction,
    pub closure: Box<Environment>,
}

impl RloxFunction {
    pub fn new(
        arity: u8,
        fn_name: String,
        declaration: StmtFunction,
        closure: Box<Environment>,
    ) -> Self {
        Self {
            arity,
            fn_name,
            declaration,
            closure,
        }
    }
}

impl Callable for RloxFunction {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        callee: String,
        arguments: Vec<Evaluation>,
    ) -> Result<Option<Evaluation>, InterpretError> {
        let mut environment = self.closure.clone();
        environment.enclosing = Some(Box::new(interpreter.environment.clone()));

        let mut declaration = self.declaration.clone();

        println!("----------------");
        for i in 0..declaration.params.len() {
            println!(
                "{:#?}{:#?}",
                declaration.params[i].lexeme.to_string(),
                Some(arguments[i].clone()),
            );

            environment.define(
                declaration.params[i].lexeme.to_string(),
                Some(arguments[i].clone()),
            )
        }

        let a = interpreter.stmt_execute_block(&mut declaration.body, &mut environment)?;

        self.closure = environment.clone();

        interpreter
            .environment
            .define(callee, Some(Evaluation::callable(Box::new(self.clone()))));

        Ok(a)
    }

    fn display(&self) -> String {
        format!("{:#?}", self.fn_name)
    }
}

impl Display for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluation::string(string) => write!(f, "{string}"),
            Evaluation::f64(f64) => write!(f, "{f64:.2}"),
            Evaluation::bool(bool) => write!(f, "{bool}"),
            Evaluation::nil(()) => write!(f, "nil"),
            Evaluation::callable(fun) => write!(f, "fn <{:#?}>", fun.display()),
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

        let clock_closure = |_: &mut Interpreter, _: Vec<Evaluation>| {
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

        let clock_callable = NativeFunction::new("clock".to_string(), clock_closure);

        globals.define(
            "clock".to_string(),
            Some(Evaluation::callable(Box::new(clock_callable))),
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
                    if let Some(eval) = self.execute_with_return(&mut stmt.body)? {
                        return Ok(Some(eval));
                    }
                }
                Ok(None)
            }
            Stmt::If(stmt) => {
                if let Evaluation::bool(truthy) = self.evaluate(stmt.condition.clone())? {
                    if truthy {
                        if let Some(eval) = self.execute_with_return(&mut stmt.then_branch)? {
                            return Ok(Some(eval));
                        }
                    } else if let Some(_) = stmt.else_branch {
                        if let Some(eval) =
                            self.execute_with_return(&mut stmt.else_branch.clone().unwrap())?
                        {
                            return Ok(Some(eval));
                        }
                    }
                };
                Ok(None)
            }
            Stmt::Function(stmt) => {
                let function = RloxFunction::new(
                    stmt.params.len() as u8,
                    stmt.name.lexeme.to_string(),
                    stmt.clone(),
                    Box::new(self.environment.clone()),
                );

                self.environment.define(
                    stmt.name.lexeme.to_string(),
                    Some(Evaluation::callable(Box::new(function))),
                );

                Ok(None)
            }
            Stmt::Block(stmt) => {
                let a = self.stmt_execute_block(&mut stmt.statements, &mut Environment::new())?;
                Ok(a)
            }
            Stmt::Expression(expr) => {
                let evl = self.evaluate(expr.clone())?;
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
        environment: &mut Environment,
    ) -> Result<Option<Evaluation>, InterpretError> {
        let previous = self.environment.clone();
        self.environment = environment.clone();
        self.environment.enclosing = Some(Box::new(previous));

        let a = statements
            .iter_mut()
            .find_map(|stmt| match self.stmt_execute(stmt) {
                Ok(Some(eval)) => Some(Ok(eval)),
                Ok(None) => None,
                Err(err) => Some(Err(err)),
            })
            .transpose()?;

        *environment = self.environment.clone();
        self.environment = *self.environment.enclosing.take().unwrap();

        Ok(a)
    }

    fn execute_with_return(
        &mut self,
        stmt: &mut Stmt,
    ) -> Result<Option<Evaluation>, InterpretError> {
        match self.stmt_execute(stmt)? {
            Some(eval) => Ok(Some(eval)),
            None => Ok(None),
        }
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
                let callee = if let Expr::Var(expr_var) = *expr.callee.clone() {
                    expr_var.name.lexeme
                } else {
                    println!("Invalid callee type");
                    exit(22)
                };
                let callee_evaluated = self.evaluate(*expr.callee.clone())?;

                let mut arguments = Vec::new();
                expr.arguments.into_iter().try_for_each(|arg| {
                    arguments.push(self.evaluate(arg)?);
                    Ok::<(), InterpretError>(())
                })?;

                let mut function: Box<dyn Callable> = Result::from(callee_evaluated.clone())?;

                let arity = function.arity().into();

                if arguments.len() != arity {
                    return Err(InterpretError::RuntimeError {
                        err: format!(
                            "Expected '{}' arguments but got '{}'",
                            arity,
                            arguments.len()
                        ),
                    });
                }

                let a = function.call(self, callee, arguments)?;

                let a = match a {
                    Some(eval) => eval,
                    None => Evaluation::nil(()),
                };

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
