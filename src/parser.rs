use crate::{
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

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_token_type(vec![TokenType::Bang, TokenType::BangEqual]) {
            let operator = self.previous();
            let right = self.comparison();
            expr = Expr::Binary(ExprBinary::new(expr, operator, right))
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_token_type(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term();
            expr = Expr::Binary(ExprBinary::new(expr, operator, right))
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_token_type(vec![TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.factor();
            expr = Expr::Binary(ExprBinary::new(expr, operator, right))
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_token_type(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary();
            expr = Expr::Binary(ExprBinary::new(expr, operator, right))
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_token_type(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary();
            return Expr::Unary(ExprUnary::new(operator, right));
        } else {
            return self.primary();
        }
    }

    fn primary(&mut self) -> Expr {
        if self.match_token_type(vec![TokenType::True]) {
            return Expr::Literal(ExprLiteral::new(Literal::bool(true)));
        }
        if self.match_token_type(vec![TokenType::False]) {
            return Expr::Literal(ExprLiteral::new(Literal::bool(false)));
        }
        if self.match_token_type(vec![TokenType::Nil]) {
            return Expr::Literal(ExprLiteral::new(Literal::null(())));
        }
        if self.match_token_type(vec![TokenType::Number]) {
            return Expr::Literal(ExprLiteral::new(self.previous().literal.unwrap()));
        }
        if self.match_token_type(vec![TokenType::String]) {
            return Expr::Literal(ExprLiteral::new(self.previous().literal.unwrap()));
        }

        if self.match_token_type(vec![TokenType::LeftParen]) {
            let expr = self.expression();
            let _ = self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Expr::Grouping(ExprGrouping::new(expr));
        } else {
            panic!();
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Token {
        let check = |token_type: TokenType| {
            if self.is_at_end() {
                return false;
            }

            self.peek().token_type == token_type
        };

        if check(token_type) {
            self.advance()
        } else {
            panic!();
        }
    }

    fn match_token_type(&mut self, token_types: Vec<TokenType>) -> bool {
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
}
