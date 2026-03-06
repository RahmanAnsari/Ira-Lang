//! Additional lexer utilities for the parser

// This module provides additional lexer functionality
// that complements the main lexer in src/lexer/mod.rs

use crate::lexer::Token;

/// Token stream for parser consumption
pub struct TokenStream {
    tokens: Vec<Token>,
    position: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }
    
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    
    pub fn advance(&mut self) -> Option<&Token> {
        if self.position < self.tokens.len() {
            let token = &self.tokens[self.position];
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }
    
    pub fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
}