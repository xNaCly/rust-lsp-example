mod error;
mod lexer;
mod lsp;
mod parser;

use std::fs;

use clap::Parser;
use lexer::{Lexer, Token, TokenType};

#[derive(clap::Parser)]
struct Config {
    /// start in lsp mode
    #[arg(long)]
    lsp: bool,
    /// file to evaluate
    path: Option<String>,
}

fn main() {
    let args = Config::parse();
    if args.lsp {
        todo!("args.lsp");
    }
    let file = match args.path {
        Some(file) => file,
        None => panic!("no source file provided"),
    };

    let file_contents = match fs::read(&file) {
        Ok(bytes) => bytes,
        Err(err) => panic!("failed to read file {file}: {err}"),
    };

    let mut errors = vec![];
    let mut tokens = vec![];
    let mut lexer = Lexer::new(&file_contents);
    'lexer_loop: loop {
        match lexer.next() {
            Ok(Token {
                token_type: TokenType::EOF,
                ..
            }) => break 'lexer_loop,
            Err(err) => errors.push(err),
            Ok(token) => tokens.push(token),
        }
    }
    dbg!(tokens);
    dbg!(errors);
}
