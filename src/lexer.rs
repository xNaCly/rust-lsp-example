use crate::{error::LspError, parser::TokenContext};

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Number(f64),
    String(String),
    Ident(String),
    DelimitorLeft,
    DelimitorRight,
    EOF,
}

pub struct Lexer<'lexer> {
    pos: usize,
    line_pos: usize,
    line: usize,
    input: &'lexer [u8],
}

impl<'lexer> Iterator for Lexer<'lexer> {
    type Item = Result<Token, LspError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            None
        } else {
            Some(self.next())
        }
    }
}

impl<'lexer> Lexer<'_> {
    pub fn new(input: &'lexer [u8]) -> Lexer<'lexer> {
        Lexer {
            pos: 0,
            line: 0,
            line_pos: 0,
            input,
        }
    }

    fn next(&mut self) -> Result<Token, LspError> {
        // skip whitespace
        while self.cur().is_some_and(|c| matches!(c, ' ' | '\t' | '\n')) {
            self.advance();
        }

        // skip comments
        if self.cur().is_some_and(|char| char == ';') {
            while self.cur().is_some_and(|char| char != '\n') {
                self.advance();
            }
            return self.next();
        }

        if self.pos >= self.input.len() {
            return self.create_token(TokenType::EOF);
        }

        let char = match self.cur() {
            Some(char) => char,
            None => return Err(self.err("Unexpected end of input", self.line_pos)),
        };

        let tok = match char {
            '(' => self.create_token(TokenType::DelimitorLeft),
            ')' => self.create_token(TokenType::DelimitorRight),
            '0'..='9' => {
                let line_start = self.line_pos - 1;
                let start = self.pos;
                while self
                    .cur()
                    .is_some_and(|char| char.is_digit(10) || char == '.')
                {
                    self.advance();
                }

                let bytes = self.input.get(start..self.pos).unwrap_or_default().to_vec();
                let string = String::from_utf8(bytes).map_err(|err| {
                    self.err(format!("Failed to create string: {err}"), line_start)
                })?;
                let number = string.parse::<f64>().map_err(|err| {
                    self.err(format!("Failed to parse number: {err}"), line_start)
                })?;

                return Ok(Token {
                    token_type: TokenType::Number(number),
                    line: self.line,
                    start: line_start,
                    end: self.line_pos - 2,
                });
            }
            'a'..='z' | 'A'..='Z' => {
                let line_start = self.line_pos - 1;
                let start = self.pos;
                while self
                    .cur()
                    .is_some_and(|char| matches!(char, 'a'..='z' | 'A'..='Z' | '_' | '0'..'9'))
                {
                    self.advance();
                }
                let bytes = self.input.get(start..self.pos).unwrap_or_default().to_vec();
                let string = String::from_utf8(bytes).map_err(|err| {
                    self.err(format!("Failed to create string: {err}"), line_start)
                })?;
                return Ok(Token {
                    token_type: TokenType::Ident(string),
                    line: self.line,
                    start: line_start,
                    end: self.line_pos - 2,
                });
            }
            // strings ofc ofc
            '"' => {
                let line_start = self.line_pos - 1;
                // skip "
                self.advance();
                let start = self.pos;
                while self.cur().is_some_and(|char| char != '"' && char != '\n') {
                    self.advance();
                }

                if self.cur().is_some_and(|char| char == '\n') {
                    return Err(self.err("Unterminated string", line_start));
                }

                let bytes = self.input.get(start..self.pos).unwrap_or_default().to_vec();
                let string = String::from_utf8(bytes).map_err(|err| {
                    self.err(format!("Failed to create string: {err}"), line_start)
                })?;
                let tok = Ok(Token {
                    token_type: TokenType::String(string),
                    line: self.line,
                    start: line_start,
                    end: self.line_pos - 1,
                });
                if self.cur().is_none() {
                    Err(self.err("Unterminated string", line_start))
                } else {
                    tok
                }
            }
            cur @ _ => Err(self.err(format!("Unkown character '{cur}'"), self.line_pos)),
        };
        self.advance();
        return tok;
    }

    fn err(&self, message: impl Into<String>, start: usize) -> LspError {
        LspError::with_context(
            TokenContext {
                line: self.line,
                start,
                end: self.line_pos - 1,
            },
            message.into(),
        )
    }

    fn create_token(&self, token_type: TokenType) -> Result<Token, LspError> {
        Ok(Token {
            token_type,
            line: self.line,
            start: self.line_pos,
            end: self.line_pos,
        })
    }

    fn advance(&mut self) {
        if self.cur().is_some_and(|c| c == '\n') {
            self.line_pos = 0;
            self.line += 1;
        }
        self.pos += 1;
        self.line_pos += 1;
    }

    fn cur(&self) -> Option<char> {
        self.input.get(self.pos).map(|u| *u as char)
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos + 1).map(|u| *u as char)
    }
}
