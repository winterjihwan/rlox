use crate::{
    errors::ParseError,
    expr::{Expr, ExprBinary, ExprGrouping, ExprLiteral, ExprUnary},
    token::{Literal, Token, TokenType},
};

pub struct Parser {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.expression().map_err(|err| {
            println!("PARSE ERROR: {}", err);
            err
        })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        self.binary_expr(
            Self::comparison,
            vec![TokenType::Bang, TokenType::BangEqual],
        )
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        self.binary_expr(
            Self::term,
            vec![
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ],
        )
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        self.binary_expr(Self::factor, vec![TokenType::Plus, TokenType::Minus])
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        self.binary_expr(Self::unary, vec![TokenType::Slash, TokenType::Star])
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        self.unary_expr(Self::primary, vec![TokenType::Bang, TokenType::Minus])
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::ParseEOF { token: self.peek() });
        }

        let expr = match self.peek().token_type {
            TokenType::True => Expr::Literal(ExprLiteral::new(Literal::bool(true))),
            TokenType::False => Expr::Literal(ExprLiteral::new(Literal::bool(false))),
            TokenType::Nil => Expr::Literal(ExprLiteral::new(Literal::nil(()))),
            TokenType::Number => Expr::Literal(ExprLiteral::new(self.previous().literal.unwrap())),
            TokenType::String => Expr::Literal(ExprLiteral::new(self.previous().literal.unwrap())),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                let _ = self.consume(
                    TokenType::RightParen,
                    "Expect ')' after expression.".to_string(),
                );
                Expr::Grouping(ExprGrouping::new(expr))
            }
            _ => {
                return Err(ParseError::ParseFail {
                    token: self.peek(),
                    message: "Expect expression.".to_string(),
                });
            }
        };

        Ok(expr)
    }

    fn synchronize(&mut self) {
        let previous = self.advance();

        while !self.is_at_end() {
            if previous.token_type == TokenType::Semicolon {
                break;
            }

            match self.peek().token_type {
                TokenType::Class => {}
                TokenType::Fun => {}
                TokenType::Var => {}
                TokenType::For => {}
                TokenType::If => {}
                TokenType::While => {}
                TokenType::Print => {}
                TokenType::Return => {
                    break;
                }
                _ => unimplemented!(),
            }

            self.advance();
        }
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> Result<Token, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::ParseEOF { token: self.peek() });
        }

        if self.peek().token_type == token_type {
            return Ok(self.advance());
        }

        Err(ParseError::ParseFail {
            token: self.peek(),
            message,
        })
    }

    fn match_token_type(&mut self, token_types: &Vec<TokenType>) -> bool {
        if self.is_at_end() {
            return false;
        }

        token_types.iter().any(|tt| {
            if self.peek().token_type == *tt {
                let _ = self.advance();
                true
            } else {
                false
            }
        })
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.current).unwrap().clone()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    pub fn binary_expr(
        &mut self,
        parse_fn: fn(&mut Self) -> Result<Expr, ParseError>,
        match_types: Vec<TokenType>,
    ) -> Result<Expr, ParseError> {
        let mut expr = parse_fn(self)?;

        while self.match_token_type(&match_types) {
            let operator = self.previous();
            let right = parse_fn(self)?;
            expr = Expr::Binary(ExprBinary::new(expr, operator, right))
        }

        Ok(expr)
    }

    pub fn unary_expr(
        &mut self,
        parse_fn: fn(&mut Self) -> Result<Expr, ParseError>,
        match_types: Vec<TokenType>,
    ) -> Result<Expr, ParseError> {
        if self.match_token_type(&match_types) {
            let operator = self.previous();
            let right = self.unary_expr(parse_fn, match_types)?;
            return Ok(Expr::Unary(ExprUnary::new(operator, right)));
        }

        parse_fn(self)
    }
}
