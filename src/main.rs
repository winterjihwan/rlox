mod scanner;
mod token;
mod token_type;

use std::{
    env,
    fs::File,
    io::{self, Read},
    process::exit,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: *.lox");
        exit(1);
    }

    let path = &args[1];

    let source = file_open(path)?;
    println!("source: {}", source);

    Ok(())
}

fn file_open(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;

    Ok(string)
}
