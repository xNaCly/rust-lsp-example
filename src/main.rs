#![allow(unused)]
mod error;
mod lexer;
mod lsp;
mod parser;

use std::fs;

use clap::Parser;
use lexer::{Lexer, Token};
use parser::Ast;

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
    let tokens = Lexer::new(&file_contents)
        .filter_map(|x| match x {
            Err(err) => {
                errors.push(err);
                None
            }
            Ok(t) => Some(t),
        })
        .collect::<Vec<Token>>();

    let _ = parser::Parser::new(&tokens)
        .filter_map(|x| match x {
            Err(err) => {
                errors.push(err);
                None
            }
            Ok(t) => Some(t),
        })
        .collect::<Vec<Ast>>();
    dbg!(errors);
}
