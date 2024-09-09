use std::io;

use thiserror::Error;

use crate::token::Token;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("I/O fail, err: {err}")]
    IoFail { err: String },

    #[error("{} at end", token.line)]
    ParseEOF { token: Token },

    #[error("{} at '{}' {}", token.line, token.lexeme,  message)]
    ParseFail { token: Token, message: String },
}

impl From<ParseError> for io::Error {
    fn from(error: ParseError) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("{:#?}", error))
    }
}
