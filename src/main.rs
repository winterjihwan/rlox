mod ast;
mod errors;
mod expr;
mod interpreter;
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

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;

fn main() -> io::Result<()> {
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

    let mut parser = Parser::new(scanner.tokens);
    let expr = parser.parse()?;

    Interpreter::interpret(expr)?;

    Ok(())
}

fn file_open(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;

    Ok(string)
}
