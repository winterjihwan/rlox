use crate::{environment::Environment, errors::InterpretError, expr::Expr, token::Token};

type StmtExpression = Expr;
type StmtPrint = Expr;

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(StmtBlock),
    Expression(StmtExpression),
    Print(StmtPrint),
    Var(StmtVar),
}

impl Stmt {
    pub fn execute(&mut self, environment: &mut Environment) -> Result<(), InterpretError> {
        self.evaluate(environment)
    }

    pub fn evaluate(&mut self, environment: &mut Environment) -> Result<(), InterpretError> {
        match self {
            Stmt::Block(stmt) => {
                Self::execute_block(&mut stmt.statements, environment, &mut Environment::new())?;
                Ok(())
            }
            Stmt::Expression(expr) => {
                let evl = expr.evaluate(environment)?;
                Ok(())
            }
            Stmt::Print(value) => {
                let value = value.evaluate(environment)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Var(var) => {
                let a = var
                    .initializer
                    .as_mut()
                    .map(|expr| expr.evaluate(environment))
                    .transpose()?;

                environment.define(var.name.lexeme.to_string(), a);
                Ok(())
            }
        }
    }

    pub fn execute_block(
        statements: &mut Vec<Stmt>,
        environment: &mut Environment,
        new_environment: &mut Environment,
    ) -> Result<(), InterpretError> {
        statements
            .iter_mut()
            .try_for_each(|stmt| stmt.execute(new_environment))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StmtVar {
    name: Token,
    initializer: Option<Expr>,
}
impl StmtVar {
    pub fn new(name: Token, initializer: Option<Expr>) -> Self {
        Self { name, initializer }
    }
}

#[derive(Debug, Clone)]
pub struct StmtBlock {
    pub statements: Vec<Stmt>,
}

impl StmtBlock {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}
