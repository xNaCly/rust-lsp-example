use crate::error::LspError;

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    line: usize,
    start: usize,
    end: usize,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Number(f64),
    String(String),
    Add,
    Subtract,
    Multipy,
    Divide,
    DelimitorLeft,
    DelimitorRight,
    /// indicates the end of the input
    EOF,
}

pub struct Lexer<'lexer> {
    pos: usize,
    line: usize,
    input: &'lexer [u8],
}

impl<'lexer> Lexer<'_> {
    pub fn new(input: &'lexer [u8]) -> Lexer<'lexer> {
        Lexer {
            pos: 0,
            line: 0,
            input,
        }
    }

    pub fn next(&mut self) -> Result<Token, LspError> {
        // skip whitespace
        while self
            .cur()
            .is_some_and(|char| matches!(char, ' ' | '\t' | '\n'))
        {
            self.advance();
        }

        let char = match self.cur() {
            Some(char) => char,
            None => return self.create_token(TokenType::EOF),
        };

        let tok = match char {
            '+' => self.create_token(TokenType::Add),
            '-' => self.create_token(TokenType::Subtract),
            '/' => self.create_token(TokenType::Divide),
            '*' => self.create_token(TokenType::Multipy),
            '(' => self.create_token(TokenType::DelimitorLeft),
            ')' => self.create_token(TokenType::DelimitorRight),
            // skip comments
            ';' if self.next_byte().is_some_and(|c| c == ';') => {
                while self.cur().is_some_and(|char| char != '\n') {
                    self.advance();
                }
                self.next()
            }
            '0'..'9' => {
                let start = self.pos;
                while self
                    .cur()
                    .is_some_and(|char| char.is_digit(10) || char == '.')
                {
                    self.advance();
                }
                let bytes = self.input.get(start..self.pos).unwrap_or_default().to_vec();
                let string = String::from_utf8(bytes).map_err(|err| {
                    self.create_error(format!("Failed to create string: {err}"), start)
                })?;
                let number = string.parse::<f64>().map_err(|err| {
                    self.create_error(format!("Failed to parse number: {err}"), start)
                })?;

                // we decrement one because we are at the last position of the integer, which the
                // self.advance at the bottom of the function skips past
                self.pos -= 1;

                Ok(Token {
                    token_type: TokenType::Number(number),
                    line: self.line,
                    start,
                    end: self.pos - 1,
                })
            }
            // strings ofc ofc
            '"' => {
                // skip "
                self.advance();
                let start = self.pos;
                while self.cur().is_some_and(|char| char != '"') {
                    self.advance();
                }
                let bytes = self.input.get(start..self.pos).unwrap_or_default().to_vec();
                let string = String::from_utf8(bytes).map_err(|err| {
                    self.create_error(format!("Failed to create string: {err}"), start)
                })?;
                let tok = Ok(Token {
                    token_type: TokenType::String(string),
                    line: self.line,
                    start,
                    end: self.pos,
                });
                if self.cur().is_none() {
                    Err(self.create_error("Unterminated string", start))
                } else {
                    tok
                }
            }
            cur @ _ => Err(self.create_error(format!("Unkown character '{cur}'"), self.pos)),
        };
        self.advance();
        return tok;
    }

    fn create_error(&self, message: impl Into<String>, start: usize) -> LspError {
        LspError::new(self.line, start, self.pos, message.into())
    }

    fn create_token(&self, token_type: TokenType) -> Result<Token, LspError> {
        Ok(Token {
            token_type,
            line: self.line,
            start: self.pos,
            end: self.pos,
        })
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn cur(&self) -> Option<char> {
        self.input.get(self.pos).map(|u| *u as char)
    }

    fn next_byte(&self) -> Option<char> {
        self.input.get(self.pos + 1).map(|u| *u as char)
    }
}
