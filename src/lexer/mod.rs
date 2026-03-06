//! Lexer module for tokenizing Ira source code

use crate::error::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Namespace,
    Override,
    Data,
    Schema,
    
    // Identifiers and literals
    Identifier(String),
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    
    // Symbols
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    At,           // @
    
    // Special
    Newline,
    Whitespace,
    Comment(String),
    
    // End of file
    Eof,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            match self.next_token() {
                Ok(token) => {
                    if !matches!(token, Token::Whitespace | Token::Comment(_)) {
                        tokens.push(token);
                    }
                },
                Err(e) => return Err(e),
            }
        }
        
        tokens.push(Token::Eof);
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        
        if self.is_at_end() {
            return Ok(Token::Eof);
        }
        
        let ch = self.current_char();
        
        match ch {
            '{' => {
                self.advance();
                Ok(Token::LeftBrace)
            },
            '}' => {
                self.advance();
                Ok(Token::RightBrace)
            },
            ':' => {
                self.advance();
                Ok(Token::Colon)
            },
            ',' => {
                self.advance();
                Ok(Token::Comma)
            },
            '@' => {
                self.advance();
                Ok(Token::At)
            },
            '"' => self.string_literal(),
            '\n' => {
                self.advance();
                self.line += 1;
                self.column = 1;
                Ok(Token::Newline)
            },
            '/' if self.peek() == Some('/') => self.line_comment(),
            c if c.is_alphabetic() || c == '_' => self.identifier_or_keyword(),
            c if c.is_numeric() => self.number_literal(),
            _ => {
                self.advance();
                Ok(Token::Whitespace)
            }
        }
    }
    
    fn string_literal(&mut self) -> Result<Token> {
        self.advance(); // consume opening quote
        let start = self.position;
        
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }
        
        if self.is_at_end() {
            return Err(IraError::parse_error(self.line, self.column, "Unterminated string"));
        }
        
        let value = self.input[start..self.position].to_string();
        self.advance(); // consume closing quote
        
        Ok(Token::String(value))
    }
    
    fn line_comment(&mut self) -> Result<Token> {
        let start = self.position;
        
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
        
        let comment = self.input[start..self.position].to_string();
        Ok(Token::Comment(comment))
    }
    
    fn identifier_or_keyword(&mut self) -> Result<Token> {
        let start = self.position;
        
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        let value = &self.input[start..self.position];
        
        let token = match value.to_uppercase().as_str() {
            "NAMESPACE" => Token::Namespace,
            "OVERRIDE" => Token::Override,
            "DATA" => Token::Data,
            "SCHEMA" => Token::Schema,
            "TRUE" => Token::Boolean(true),
            "FALSE" => Token::Boolean(false),
            _ => Token::Identifier(value.to_string()),
        };
        
        Ok(token)
    }
    
    fn number_literal(&mut self) -> Result<Token> {
        let start = self.position;
        let mut has_dot = false;
        
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_numeric() {
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }
        
        let value_str = &self.input[start..self.position];
        
        if has_dot {
            let value: f64 = value_str.parse()
                .map_err(|_| IraError::parse_error(self.line, self.column, "Invalid number"))?;
            Ok(Token::Number(value))
        } else {
            let value: i64 = value_str.parse()
                .map_err(|_| IraError::parse_error(self.line, self.column, "Invalid integer"))?;
            Ok(Token::Integer(value))
        }
    }
    
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                },
                _ => break,
            }
        }
    }
    
    fn current_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }
    
    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }
    
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
            self.column += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}