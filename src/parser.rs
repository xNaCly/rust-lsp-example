use std::collections::HashMap;

use crate::{
    error::LspError,
    lexer::{Token, TokenType},
};

#[derive(Default, Clone, Debug)]
pub struct TokenContext {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

impl From<&Token> for TokenContext {
    fn from(t: &Token) -> Self {
        let &Token {
            line, start, end, ..
        } = t;
        Self { line, start, end }
    }
}

impl From<&TokenContext> for TokenContext {
    fn from(ctx: &TokenContext) -> Self {
        let &TokenContext {
            line, start, end, ..
        } = ctx;
        Self { line, start, end }
    }
}

#[derive(Default)]
pub struct Context {
    pub variables: HashMap<String, Node>,
}

#[derive(Debug, Clone)]
pub enum Node {
    /// Number represtents all numbers, can be floating point or whole
    Number {
        ctx: TokenContext,
        val: f64,
    },
    /// String is a rust native String and holds all bytes between ""
    String {
        ctx: TokenContext,
        val: String,
    },
    Ident {
        ctx: TokenContext,
        val: String,
    },
    List(Vec<Node>),
    Var {
        ctx: TokenContext,
        ident: String,
        value: Box<Node>,
    },
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
        let ctx = self.cur().map(|t| t.into()).unwrap_or_default();
        self.advance();
        Err(LspError::with_context(ctx, msg))
    }

    fn parse(&mut self) -> Result<Node, LspError> {
        match self.cur() {
            Some(tok) => match &tok.token_type {
                TokenType::EOF => Ok(Node::Null),
                TokenType::Number(_) | TokenType::String(_) | TokenType::Ident(_) => self.atom(),
                TokenType::DelimitorLeft => self.list_like(),
                t @ _ => self.err(format!("Unexpected {:?}, wanted Atom or List", t)),
            },
            None => Ok(Node::Null),
        }
    }

    fn variable(&mut self) -> Result<Node, LspError> {
        self.advance(); // skip TokenType::Ident("let")
        let ctx: TokenContext;
        let ident = if let Some(Token {
            token_type: TokenType::Ident(ident),
            ..
        }) = self.cur()
        {
            ctx = self.cur().map(|n| n.into()).unwrap();
            ident.clone()
        } else {
            return self.err(format!(
                "Unexpected {:?}, wanted an Identifier as the variable name",
                self.cur().map(|e| &e.token_type)
            ));
        };
        // skipping the ident
        self.advance();
        let value = self.parse()?;
        Ok(Node::Var {
            ctx,
            ident,
            value: Box::new(value),
        })
    }

    fn list_like(&mut self) -> Result<Node, LspError> {
        self.consume(TokenType::DelimitorLeft)?;

        let r = match &self.cur() {
            Some(Token {
                token_type: TokenType::Ident(name),
                ..
            }) => match name.as_str() {
                "let" => self.variable(),
                _ => self.list(),
            },
            _ => self.list(),
        };

        self.consume(TokenType::DelimitorRight)?;
        r
    }

    fn list(&mut self) -> Result<Node, LspError> {
        let mut children = vec![];
        while self
            .cur()
            .map(|e| &e.token_type)
            .is_some_and(|t| *t != TokenType::DelimitorRight && *t != TokenType::EOF)
        {
            children.push(self.parse()?);
        }

        Ok(Node::List(children))
    }

    fn atom(&mut self) -> Result<Node, LspError> {
        let tok = match self.cur() {
            Some(tok) => tok,
            None => {
                return Err(LspError::with_context(
                    TokenContext::default(),
                    "Unexpected EOF, wanted Atom".into(),
                ))
            }
        };

        let t = match tok {
            Token {
                token_type: TokenType::Number(num),
                ..
            } => Ok(Node::Number {
                val: *num,
                ctx: tok.into(),
            }),
            Token {
                token_type: TokenType::String(str),
                ..
            } => Ok(Node::String {
                val: str.into(),
                ctx: tok.into(),
            }),
            Token {
                token_type: TokenType::Ident(str),
                ..
            } => Ok(Node::Ident {
                val: str.into(),
                ctx: tok.into(),
            }),
            Token {
                token_type: TokenType::DelimitorLeft,
                ..
            } => self.parse(),
            _ => Err(LspError::with_context(
                tok.into(),
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
