//! # Ira Language
//! 
//! A domain-specific language for football simulation data storage.
//! 
//! Ira provides ultra-compact binary serialization with built-in schemas
//! for football entities like countries, teams, players, and leagues.

pub mod lexer;
pub mod parser;
pub mod compiler;
pub mod runtime;
pub mod schemas;
pub mod types;
pub mod error;

pub use error::{IraError, Result};
pub use types::*;
pub use schemas::*;

/// Main entry point for the Ira language
pub struct IraLanguage {
    pub schemas: BuiltInSchemas,
    pub config: LanguageConfig,
}

impl IraLanguage {
    /// Create a new Ira language instance with default configuration
    pub fn new() -> Self {
        Self {
            schemas: BuiltInSchemas::new(),
            config: LanguageConfig::default(),
        }
    }
    
    /// Parse an Ira file and return the AST
    pub fn parse(&self, source: &str) -> Result<IraFile> {
        parser::parse_ira_file(source, &self.schemas)
    }
    
    /// Compile an Ira file to binary format
    pub fn compile(&self, ast: &IraFile) -> Result<Vec<u8>> {
        compiler::compile_to_binary(ast, &self.config)
    }
    
    /// Parse and compile in one step
    pub fn parse_and_compile(&self, source: &str) -> Result<Vec<u8>> {
        let ast = self.parse(source)?;
        self.compile(&ast)
    }
}

impl Default for IraLanguage {
    fn default() -> Self {
        Self::new()
    }
}