use std::process::exit;

use crate::{
    reserved::RESERVED_KEYWORDS,
    token::{Literal, Token},
    token_type::TokenType,
};

struct Scanner {
    source: String,
    tokens: Vec<Token>,

    start: usize,
    line: usize,
    current: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),

            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_end() {
            self.start = self.current;
            self._scan_tokens();
        }

        self.tokens
            .push(Token::new(TokenType::EOF, " ".to_string(), None, self.line))
    }

    fn _scan_tokens(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),

            '!' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::BangEqual,
                    false => TokenType::Bang,
                };

                self.add_token(token_type, None)
            }
            '=' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::EqualEqual,
                    false => TokenType::Equal,
                };

                self.add_token(token_type, None)
            }
            '<' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::LessEqual,
                    false => TokenType::Less,
                };

                self.add_token(token_type, None)
            }
            '>' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::GreaterEqual,
                    false => TokenType::Greater,
                };

                self.add_token(token_type, None)
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_end() {
                        self.advance();
                    }
                };

                self.add_token(TokenType::Slash, None)
            }

            ' ' => {}
            '\r' => {}
            '\t' => {}
            '\n' => {
                self.line += 1;
            }
            '"' => {
                self.string();
                self.line += 1;
            }

            'o' => {
                if self.match_char('r') {
                    self.add_token(TokenType::Or, None)
                }
            }

            _ => {
                if c.is_numeric() {
                    self.number();
                } else if c.is_alphabetic() {
                    self.identifier()
                } else {
                    println!("Unexpected token: {c}");
                    exit(11)
                }
            }
        }
    }

    fn is_end(&self) -> bool {
        if self.source.len() < self.current {
            return true;
        }
        false
    }

    fn advance(&mut self) -> char {
        let char = self.source.chars().nth(self.current).unwrap();
        self.current += 1;

        char
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let lexeme = String::from(&self.source[self.start..self.current]);

        self.tokens
            .push(Token::new(token_type, lexeme, literal, self.line));
    }

    fn match_char(&mut self, expected_char: char) -> bool {
        if self.is_end() {
            return false;
        }

        if self.source.chars().nth(self.current).unwrap() != expected_char {
            return false;
        }

        self.current += 1;
        return true;
    }

    fn peek(&self) -> char {
        if self.is_end() {
            return '\0';
        }

        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.source.len() < self.current + 1 {
            return '\0';
        }

        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_end() {
            println!("Unterminated string");
            exit(12)
        }

        self.advance();

        let literal = String::from(&self.source[self.start + 1..self.current - 1]);
        self.add_token(TokenType::String, Some(Literal::string(literal)));
    }

    fn number(&mut self) {
        while self.peek().is_numeric() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_numeric() {
            self.advance();

            while self.peek().is_numeric() {
                self.advance();
            }

            //TODO: return here as fsize
        };

        let literal = (&self.source[self.start + 1..self.current - 1])
            .parse::<usize>()
            .unwrap();

        self.add_token(TokenType::String, Some(Literal::usize(literal)));
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_type = {
            match RESERVED_KEYWORDS.get(text) {
                Some(t) => *t,
                None => TokenType::Identifier,
            }
        };

        self.add_token(token_type, None);
    }
}
