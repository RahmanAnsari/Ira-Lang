//! Error handling for the Ira language

use thiserror::Error;

pub type Result<T> = std::result::Result<T, IraError>;

#[derive(Error, Debug)]
pub enum IraError {
    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("Schema error: {message}")]
    SchemaError {
        message: String,
    },
    
    #[error("Validation error in field '{field}': {message}")]
    ValidationError {
        field: String,
        message: String,
    },
    
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: String,
        found: String,
    },
    
    #[error("Unknown schema: {schema_name}")]
    UnknownSchema {
        schema_name: String,
    },
    
    #[error("Unknown field: {field_name} in schema {schema_name}")]
    UnknownField {
        field_name: String,
        schema_name: String,
    },
    
    #[error("Required field missing: {field_name} in schema {schema_name}")]
    RequiredFieldMissing {
        field_name: String,
        schema_name: String,
    },
    
    #[error("Reference error: {target_schema}.{target_instance} not found")]
    ReferenceError {
        target_schema: String,
        target_instance: String,
    },
    
    #[error("Range validation failed: {value} is not in range {min}-{max}")]
    RangeValidationError {
        value: i32,
        min: i32,
        max: i32,
    },
    
    #[error("Choice validation failed: '{value}' is not one of {choices:?}")]
    ChoiceValidationError {
        value: String,
        choices: Vec<String>,
    },
    
    #[error("Compilation error: {message}")]
    CompilationError {
        message: String,
    },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl IraError {
    /// Create a parse error with location information
    pub fn parse_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::ParseError {
            line,
            column,
            message: message.into(),
        }
    }
    
    /// Create a schema error
    pub fn schema_error(message: impl Into<String>) -> Self {
        Self::SchemaError {
            message: message.into(),
        }
    }
    
    /// Create a validation error
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
    
    /// Create a type mismatch error
    pub fn type_mismatch(expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            found: found.into(),
        }
    }
    
    /// Create an unknown schema error
    pub fn unknown_schema(schema_name: impl Into<String>) -> Self {
        Self::UnknownSchema {
            schema_name: schema_name.into(),
        }
    }
    
    /// Create an unknown field error
    pub fn unknown_field(field_name: impl Into<String>, schema_name: impl Into<String>) -> Self {
        Self::UnknownField {
            field_name: field_name.into(),
            schema_name: schema_name.into(),
        }
    }
    
    /// Create a required field missing error
    pub fn required_field_missing(field_name: impl Into<String>, schema_name: impl Into<String>) -> Self {
        Self::RequiredFieldMissing {
            field_name: field_name.into(),
            schema_name: schema_name.into(),
        }
    }
    
    /// Create a reference error
    pub fn reference_error(target_schema: impl Into<String>, target_instance: impl Into<String>) -> Self {
        Self::ReferenceError {
            target_schema: target_schema.into(),
            target_instance: target_instance.into(),
        }
    }
    
    /// Create a range validation error
    pub fn range_validation_error(value: i32, min: i32, max: i32) -> Self {
        Self::RangeValidationError { value, min, max }
    }
    
    /// Create a choice validation error
    pub fn choice_validation_error(value: impl Into<String>, choices: Vec<String>) -> Self {
        Self::ChoiceValidationError {
            value: value.into(),
            choices,
        }
    }
    
    /// Create a compilation error
    pub fn compilation_error(message: impl Into<String>) -> Self {
        Self::CompilationError {
            message: message.into(),
        }
    }
}