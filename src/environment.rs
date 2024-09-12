use std::collections::HashMap;

use crate::{errors::InterpretError, interpreter::Evaluation, token::Token};

#[derive(Debug, Clone)]
pub struct Environment {
    pub env: HashMap<String, Option<Evaluation>>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn define(&mut self, name: String, value: Option<Evaluation>) {
        self.env.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Evaluation, InterpretError> {
        if let Some(Some(evl)) = self.env.get(&name.lexeme) {
            return Ok(evl.clone());
        }

        if let Some(enc) = &self.enclosing {
            return Ok(enc.get(name)?);
        }

        Err(InterpretError::RuntimeError {
            err: format!(
                "Undefined variable '{}' of token '{:#?}'.",
                name.lexeme, name
            ),
        })
    }

    pub fn assign(&mut self, name: &Token, value: Evaluation) -> Result<(), InterpretError> {
        if self.env.contains_key(&name.lexeme) {
            self.env.insert(name.lexeme.to_string(), Some(value));
            return Ok(());
        }

        if let Some(enc) = &mut self.enclosing {
            enc.assign(name, value)?;
            return Ok(());
        };

        Err(InterpretError::UndefinedVariable {
            lexeme: name.lexeme.to_string(),
        })
    }
}
