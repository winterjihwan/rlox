use crate::{
    errors::ParseError,
    expr::{Expr, ExprAssign, ExprBinary, ExprGrouping, ExprLiteral, ExprUnary, ExprVar},
    stmt::{Stmt, StmtBlock, StmtIf, StmtVar, StmtWhile},
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration())
        }

        let stmts = statements
            .into_iter()
            .filter_map(|stmt| Some(stmt.unwrap()))
            .collect();

        Ok(stmts)
    }

    fn declaration(&mut self) -> Option<Stmt> {
        if self.match_token(&vec![TokenType::Var]) {
            match self.var_declaration() {
                Ok(stmt) => return Some(stmt),
                Err(err) => {
                    println!("Prior to synchronize: {}", err);
                    self.synchronize();
                    return None;
                }
            }
        };

        match self.statement() {
            Ok(stmt) => Some(stmt),
            Err(err) => {
                println!("Prior to synchronize: {}", err);
                self.synchronize();
                None
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.".to_string())?;

        let mut initializer = None;
        if self.match_token(&vec![TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.".to_string(),
        )?;

        Ok(Stmt::Var(StmtVar::new(name, initializer)))
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&vec![TokenType::For]) {
            return self.for_statement();
        };
        if self.match_token(&vec![TokenType::While]) {
            return self.while_statement();
        };
        if self.match_token(&vec![TokenType::If]) {
            return self.if_statement();
        };
        if self.match_token(&vec![TokenType::Print]) {
            return self.print_statement();
        };
        if self.match_token(&vec![TokenType::LeftBrace]) {
            return Ok(Stmt::Block(StmtBlock::new(self.block()?)));
        };

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(
            TokenType::LeftParen,
            "Expect '(' after 'while'.".to_string(),
        )?;

        let initializer = if self.match_token(&vec![TokenType::Semicolon]) {
            None
        } else if self.match_token(&vec![TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenType::Semicolon)? {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after loop condition.".to_string(),
        )?;

        let increment = if !self.check(TokenType::RightParen)? {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::RightParen,
            "Expect ')' after for clauses.".to_string(),
        )?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(StmtBlock::new(vec![body, Stmt::Expression(inc)]))
        }

        if let Some(cnd) = condition {
            body = Stmt::Block(StmtBlock::new(vec![Stmt::While(StmtWhile::new(cnd, body))]))
        }

        if let Some(ini) = initializer {
            body = Stmt::Block(StmtBlock::new(vec![ini, body]))
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(
            TokenType::LeftParen,
            "Expect '(' after 'while'.".to_string(),
        )?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            "Expect ')' after condition.".to_string(),
        )?;

        let body = self.statement()?;

        Ok(Stmt::While(StmtWhile::new(condition, body)))
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.".to_string())?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after 'if'.".to_string())?;

        let then_branch = self.statement()?;
        let mut else_branch = None;

        if self.match_token(&vec![TokenType::Else]) {
            let a = self.statement()?;
            else_branch = Some(a);
        };

        Ok(Stmt::If(StmtIf::new(condition, then_branch, else_branch)))
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.".to_string())?;
        Ok(Stmt::Print(value))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if self.peek().token_type == TokenType::RightBrace {
                break;
            }
            statements.push(self.declaration().unwrap())
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.".to_string())?;

        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after expression.".to_string(),
        )?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if self.match_token(&vec![TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Var(ExprVar { name }) = expr {
                let name = name;
                return Ok(Expr::Assign(ExprAssign::new(name, value)));
            }

            return Err(ParseError::InvalidAssignmentTarget { token: equals });
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        self.binary_expr(Self::and, vec![TokenType::Or])
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        self.binary_expr(Self::equality, vec![TokenType::And])
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

        let expr = match self.peek_consume().token_type {
            TokenType::True => Expr::Literal(ExprLiteral::new(Literal::bool(true))),
            TokenType::False => Expr::Literal(ExprLiteral::new(Literal::bool(false))),
            TokenType::Nil => Expr::Literal(ExprLiteral::new(Literal::nil(()))),
            TokenType::Number => Expr::Literal(ExprLiteral::new(self.previous().literal.unwrap())),
            TokenType::String => Expr::Literal(ExprLiteral::new(self.previous().literal.unwrap())),
            TokenType::Identifier => Expr::Var(ExprVar::new(self.previous())),
            TokenType::Var => Expr::Var(ExprVar::new(self.peek())),
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

    fn match_token(&mut self, token_types: &Vec<TokenType>) -> bool {
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

    fn check(&self, token_type: TokenType) -> Result<bool, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::ParseEOF { token: self.peek() });
        }

        Ok(self.peek().token_type == token_type)
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.current).unwrap().clone()
    }

    fn peek_consume(&mut self) -> Token {
        let current_token = self.tokens.get(self.current).unwrap().clone();
        self.current += 1;

        current_token
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

        while self.match_token(&match_types) {
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
        if self.match_token(&match_types) {
            let operator = self.previous();
            let right = self.unary_expr(parse_fn, match_types)?;
            return Ok(Expr::Unary(ExprUnary::new(operator, right)));
        }

        parse_fn(self)
    }
}
