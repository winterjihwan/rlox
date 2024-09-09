mod ast;
mod expr;
mod parser;
mod reserved;
mod scanner;
mod token;

use std::{
    env,
    fs::File,
    io::{self, Read},
    process::exit,
};

use ast::ast_print;
use expr::{Expr, ExprBinary, ExprGrouping, ExprLiteral, ExprUnary};
use scanner::Scanner;
use token::{Literal, Token, TokenType};

//fn test() -> io::Result<()> {
//    let expr = Expr::Binary(ExprBinary::new(
//        Expr::Unary(ExprUnary::new(
//            Token::new(TokenType::Minus, "-".to_string(), None, 1),
//            Expr::Literal(ExprLiteral::new(Literal::usize(123))),
//        )),
//        Token::new(TokenType::Star, "*".to_string(), None, 1),
//        Expr::Grouping(ExprGrouping::new(Expr::Literal(ExprLiteral::new(
//            Literal::f64(45.67),
//        )))),
//    ));
//
//    let a = ast_print(expr);
//    println!("{a}");
//
//    Ok(())
//}

fn main() -> io::Result<()> {
    //test()
    rlox_run()
}

fn rlox_run() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: *.lox");
        exit(1);
    }

    let path = &args[1];

    let source = file_open(path)?;

    let mut scanner = Scanner::new(source);
    scanner.scan_tokens();

    Ok(())
}

fn file_open(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;

    Ok(string)
}
