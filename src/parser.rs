use crate::{
    error::LspError,
    lexer::{Token, TokenType},
};

#[derive(Debug)]
pub enum Atom {
    /// Number represtents all numbers, can be floating point or whole
    Number(f64),
    /// String is a rust native String and holds all bytes between ""
    String(String),
    Ident(String),
}

#[derive(Debug)]
pub enum Ast {
    Atom(Atom),
    List { op: TokenType, children: Vec<Ast> },
    Unknown,
}

pub struct Parser<'parser> {
    pos: usize,
    tokens: &'parser [Token],
}

impl<'parser> Iterator for Parser<'parser> {
    type Item = Result<Ast, LspError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.tokens.len() {
            None
        } else {
            Some(self.next())
        }
    }
}

impl<'parser> Parser<'parser> {
    pub fn new(tokens: &'parser [Token]) -> Parser<'parser> {
        Parser { pos: 0, tokens }
    }

    fn next(&mut self) -> Result<Ast, LspError> {
        self.parse()
    }

    fn parse(&mut self) -> Result<Ast, LspError> {
        match self.cur() {
            Some(tok) => match tok.token_type.clone() {
                TokenType::Number(_) | TokenType::String(_) => {
                    let r = self.literal();
                    self.advance();
                    r
                }
                TokenType::DelimitorLeft => self.binary(),
                t @ _ => {
                    self.advance();
                    Err(LspError::new(
                        0,
                        0,
                        0,
                        format!("Unexpected {:?}, wanted Number, String or DelimitorLeft", t),
                    ))
                }
            },
            None => Ok(Ast::Unknown),
        }
    }

    fn binary(&mut self) -> Result<Ast, LspError> {
        self.consume(TokenType::DelimitorLeft)?;

        let tok = match self.cur() {
            Some(tok) => tok,
            None => {
                return Err(LspError::new(
                    0,
                    0,
                    0,
                    "Unexpected EOF, wanted Binary".into(),
                ))
            }
        };

        match tok.token_type {
            TokenType::Add | TokenType::Subtract | TokenType::Divide | TokenType::Multipy => (),
            _ => {
                return dbg!(Err(LspError::new(
                    tok.line,
                    tok.start,
                    tok.end,
                    format!(
                        "Wanted Add, Subtract, Divide or Multipy, got {:?}",
                        tok.token_type
                    ),
                )))
            }
        };

        let token_type = tok.token_type.clone();

        self.advance();

        let mut children = vec![];

        while self
            .cur()
            .is_some_and(|tok| tok.token_type != TokenType::DelimitorRight)
        {
            children.push(self.parse()?);
        }

        let bin = Ast::List {
            op: token_type,
            children,
        };

        self.consume(TokenType::DelimitorRight)?;
        Ok(bin)
    }

    fn literal(&mut self) -> Result<Ast, LspError> {
        let tok = match self.cur() {
            Some(tok) => tok,
            None => {
                return Err(LspError::new(
                    0,
                    0,
                    0,
                    "Unexpected EOF, wanted Binary".into(),
                ))
            }
        };

        match tok {
            Token {
                token_type: TokenType::Number(num),
                ..
            } => Ok(Ast::Atom(Atom::Number(*num))),
            Token {
                token_type: TokenType::String(str),
                ..
            } => Ok(Ast::Atom(Atom::String(str.into()))),
            Token {
                token_type: TokenType::DelimitorLeft,
                ..
            } => self.parse(),
            _ => Err(LspError::new(
                tok.line,
                tok.start,
                tok.end,
                format!(
                    "Wanted String, Number or DelimitorLeft, got {:?}",
                    tok.token_type
                ),
            )),
        }
    }

    fn consume(&mut self, token_type: TokenType) -> Result<(), LspError> {
        let tok = match self.cur() {
            Some(tok) => tok,
            _ => {
                return Err(LspError::new(
                    0,
                    0,
                    0,
                    "Unexpected EOF, wanted Binary".into(),
                ))
            }
        };
        let r = if tok.token_type == token_type {
            Ok(())
        } else {
            Err(LspError::new(
                tok.line,
                tok.start,
                tok.end,
                format!("Wanted {:?}, but got {:?}", token_type, tok.token_type,),
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
