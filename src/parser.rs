use crate::{
    error::LspError,
    lexer::{Token, TokenType},
};

#[derive(Debug)]
pub enum Ast {
    Number(f64),
    String(String),
    Binary {
        op: TokenType,
        left: Box<Ast>,
        right: Box<Ast>,
    },
    Unknown,
}

pub struct Parser<'parser> {
    pos: usize,
    tokens: &'parser [Token],
}

impl<'parser> Parser<'parser> {
    pub fn new(tokens: &'parser [Token]) -> Parser<'parser> {
        Parser { pos: 0, tokens }
    }

    pub fn next(&mut self) -> Result<Ast, LspError> {
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

        let bin = Ast::Binary {
            op: token_type,
            left: Box::new(self.parse()?),
            right: Box::new(self.parse()?),
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
            } => Ok(Ast::Number(*num)),
            Token {
                token_type: TokenType::String(str),
                ..
            } => Ok(Ast::String(str.into())),
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
