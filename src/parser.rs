use std::collections::HashMap;

use crate::{
    error::LspError,
    lexer::{Token, TokenType},
};

#[derive(Default)]
pub struct Context {
    pub variables: HashMap<String, Node>,
}

#[derive(Debug)]
pub enum Node {
    /// Number represtents all numbers, can be floating point or whole
    Number(f64),
    /// String is a rust native String and holds all bytes between ""
    String(String),
    Ident(String),
    List(Vec<Node>),
    // Emitted for unknown elements and as a stop
    Null,
}

pub struct Parser<'parser> {
    pos: usize,
    tokens: &'parser [Token],
}

impl<'parser> Iterator for Parser<'parser> {
    type Item = Result<Node, LspError>;

    fn next(&mut self) -> Option<Self::Item> {
        let ast = self.parse();
        if let Ok(Node::Null) = ast {
            None
        } else {
            Some(ast)
        }
    }
}

impl<'parser> Parser<'parser> {
    pub fn new(tokens: &'parser [Token]) -> Parser<'parser> {
        Parser { pos: 0, tokens }
    }

    fn err(&mut self, msg: String) -> Result<Node, LspError> {
        let (line, start, end) = self
            .cur()
            .map(|t| (t.line, t.start, t.end))
            .unwrap_or_else(|| (0, 0, 0));
        self.advance();
        Err(LspError::new(line, start, end, msg))
    }

    fn parse(&mut self) -> Result<Node, LspError> {
        match self.cur() {
            Some(tok) => match &tok.token_type {
                TokenType::EOF => Ok(Node::Null),
                TokenType::Number(_) | TokenType::String(_) | TokenType::Ident(_) => self.atom(),
                TokenType::DelimitorLeft => self.list(),
                t @ _ => self.err(format!("Unexpected {:?}, wanted Atom or List", t)),
            },
            None => Ok(Node::Null),
        }
    }

    fn list(&mut self) -> Result<Node, LspError> {
        self.consume(TokenType::DelimitorLeft)?;

        let tok = match self.cur() {
            Some(tok) => tok,
            None => return Err(LspError::new(0, 0, 0, "Unexpected EOF, wanted List".into())),
        };

        let mut children = vec![];

        while self
            .cur()
            .map(|e| &e.token_type)
            .is_some_and(|t| t != &TokenType::DelimitorRight && t != &TokenType::EOF)
        {
            children.push(self.parse()?);
        }

        let bin = Node::List(children);

        self.consume(TokenType::DelimitorRight)?;
        Ok(bin)
    }

    fn atom(&mut self) -> Result<Node, LspError> {
        let tok = match self.cur() {
            Some(tok) => tok,
            None => return Err(LspError::new(0, 0, 0, "Unexpected EOF, wanted Atom".into())),
        };

        let t = match tok {
            Token {
                token_type: TokenType::Number(num),
                ..
            } => Ok(Node::Number(*num)),
            Token {
                token_type: TokenType::String(str),
                ..
            } => Ok(Node::String(str.into())),
            Token {
                token_type: TokenType::Ident(str),
                ..
            } => Ok(Node::Ident(str.into())),
            Token {
                token_type: TokenType::DelimitorLeft,
                ..
            } => self.parse(),
            _ => Err(LspError::new(
                tok.line,
                tok.start,
                tok.end,
                format!("Wanted Atom or DelimitorLeft, got {:?}", tok.token_type),
            )),
        };

        self.advance();
        t
    }

    fn consume(&mut self, token_type: TokenType) -> Result<Node, LspError> {
        let tok = match self.cur() {
            Some(tok) => tok,
            _ => return self.err(format!("Unexpected EOF in place of {:?}", token_type)),
        };
        let r = if tok.token_type == token_type {
            Ok(Node::Null)
        } else {
            self.err(format!(
                "Unexpected {:?} in place of {:?}",
                token_type, tok.token_type,
            ))
        };
        self.advance();
        r
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn cur(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
}
