#![allow(unused)]
mod error;
mod eval;
mod lexer;
mod lsp;
mod parser;

use std::fs;

use clap::Parser;
use lexer::{Lexer, Token};
use parser::{Context, Node};

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

    let mut ctx = Context::default();
    let ast = parser::Parser::new(&tokens)
        .filter_map(|x| match x {
            Err(err) => {
                errors.push(err);
                None
            }
            Ok(t) => Some(t),
        })
        .collect::<Vec<_>>();
    let eval_result = dbg!(ast)
        .into_iter()
        .flat_map(|node| match ctx.eval(&node) {
            Ok(str) => Some(str),
            Err(err) => {
                errors.push(err);
                None
            }
        })
        .flatten()
        .collect::<Vec<String>>();
    dbg!(errors);
    println!("{:?}", eval_result);
}
