//! Core type definitions for the Ira language

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main Ira file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IraFile {
    pub override_namespace: Option<OverrideNamespace>,
    pub data_namespace: DataNamespace,
}

/// NAMESPACE OVERRIDE {} block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideNamespace {
    pub schema_overrides: HashMap<SchemaType, SchemaOverride>,
}

/// NAMESPACE DATA {} block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataNamespace {
    pub schema_data: HashMap<SchemaType, SchemaData>,
}

/// Built-in schema types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchemaType {
    Countries,
    Teams,
    Players,
    Leagues,
    Matches,
    Stadiums,
}

/// Schema override configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaOverride {
    pub field_overrides: HashMap<String, FieldOverride>,
    pub validation_rules: Vec<ValidationRule>,
}

/// Individual field override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOverride {
    pub data_type: Option<DataType>,
    pub range: Option<Range>,
    pub requirement: Option<FieldRequirement>,
    pub format: Option<DisplayFormat>,
    pub validation: Option<FieldValidation>,
}

/// Time zone representation with hours and minutes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeZone {
    pub hours: i8,      // -12 to +14
    pub minutes: u8,    // 0, 15, 30, 45
}

impl TimeZone {
    /// Create a new TimeZone with validation
    pub fn new(hours: i8, minutes: u8) -> Result<Self, String> {
        if hours < -12 || hours > 14 {
            return Err(format!("Hours must be between -12 and +14, got {}", hours));
        }
        if ![0, 15, 30, 45].contains(&minutes) {
            return Err(format!("Minutes must be 0, 15, 30, or 45, got {}", minutes));
        }
        Ok(Self { hours, minutes })
    }
    
    /// Convert to decimal hours (e.g., +5:30 -> 5.5)
    pub fn to_decimal(&self) -> f64 {
        self.hours as f64 + (self.minutes as f64 / 60.0)
    }
    
    /// Convert from decimal hours (e.g., 5.5 -> +5:30)
    pub fn from_decimal(decimal: f64) -> Result<Self, String> {
        let hours = decimal.trunc() as i8;
        let fraction = decimal.fract();
        
        let minutes = match (fraction * 60.0).round() as u8 {
            0 => 0,
            15 => 15,
            30 => 30,
            45 => 45,
            _ => return Err(format!("Invalid timezone fraction, must be .00, .25, .50, or .75, got {}", fraction)),
        };
        
        Self::new(hours, minutes)
    }
    
    /// Format as string (e.g., "+05:30", "-03:00")
    pub fn format(&self) -> String {
        format!("{:+03}:{:02}", self.hours, self.minutes)
    }
}

/// Data type definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Text { max_length: Option<usize> },
    Number,
    Money { currency: Option<CurrencyType> },
    Rating { min: u8, max: u8 },
    Range { min: i32, max: i32 },
    Year,
    Boolean,
    Choice { options: Vec<String> },
    Reference { schema: SchemaType },
    Array { element_type: Box<DataType>, max_size: Option<usize> },
    TimeZone,
}

/// Numeric range definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Range {
    pub min: i32,
    pub max: i32,
}

/// Field requirement level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldRequirement {
    Required,
    Optional,
}

/// Display format options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisplayFormat {
    Millions,        // 1.4M instead of 1400000
    Billions,        // 1.4B instead of 1400000000
    Percentage,      // 75% instead of 0.75
    Currency { code: String },
    LocalCurrency,   // Use local currency from context
}

/// Field validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldValidation {
    Min(i32),
    Max(i32),
    MaxRelativeTo(String),  // field_name
    MinRelativeTo(String),  // field_name
}

/// Global validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    FieldComparison {
        field1: String,
        operator: ComparisonOperator,
        field2: String,
    },
    ConditionalRequired {
        condition_field: String,
        condition_value: IraValue,
        required_fields: Vec<String>,
    },
}

/// Comparison operators for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

/// Currency type definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CurrencyType {
    USD,
    EUR,
    GBP,
    INR,
    Local,  // Use currency from context
}

/// Schema data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaData {
    pub instances: HashMap<String, DataInstance>,
}

/// Individual data instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataInstance {
    pub fields: HashMap<String, IraValue>,
}

/// Value types in Ira
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IraValue {
    Text(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Money { amount: f64, currency: CurrencyType },
    Rating(u8),
    Year(u16),
    Reference { schema: SchemaType, instance: String },
    Array(Vec<IraValue>),
    Choice(String),
    TimeZone(TimeZone),
}

/// Language configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub default_rating_range: Range,
    pub default_reputation_range: Range,
    pub binary_format_version: u16,
    pub string_compression: bool,
    pub data_compression: CompressionType,
}

/// Compression types for binary output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    LZ4,
    Gzip,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            default_rating_range: Range { min: 1, max: 100 },
            default_reputation_range: Range { min: 1, max: 100 },
            binary_format_version: 1,
            string_compression: true,
            data_compression: CompressionType::None,
        }
    }
}

impl IraValue {
    /// Get the data type of this value
    pub fn data_type(&self) -> DataType {
        match self {
            IraValue::Text(s) => DataType::Text { max_length: Some(s.len()) },
            IraValue::Number(_) => DataType::Number,
            IraValue::Integer(_) => DataType::Number,
            IraValue::Boolean(_) => DataType::Boolean,
            IraValue::Money { currency, .. } => DataType::Money { currency: Some(currency.clone()) },
            IraValue::Rating(_) => DataType::Rating { min: 1, max: 100 },
            IraValue::Year(_) => DataType::Year,
            IraValue::Reference { schema, .. } => DataType::Reference { schema: schema.clone() },
            IraValue::Array(arr) => {
                let element_type = if let Some(first) = arr.first() {
                    Box::new(first.data_type())
                } else {
                    Box::new(DataType::Text { max_length: None })
                };
                DataType::Array { element_type, max_size: Some(arr.len()) }
            },
            IraValue::Choice(_) => DataType::Choice { options: vec![] },
            IraValue::TimeZone(_) => DataType::TimeZone,
        }
    }
    
    /// Convert to binary representation
    pub fn to_binary(&self) -> Vec<u8> {
        match self {
            IraValue::Text(s) => {
                let mut bytes = vec![];
                let str_bytes = s.as_bytes();
                bytes.extend_from_slice(&(str_bytes.len() as u32).to_le_bytes());
                bytes.extend_from_slice(str_bytes);
                bytes
            },
            IraValue::Number(n) => n.to_le_bytes().to_vec(),
            IraValue::Integer(i) => i.to_le_bytes().to_vec(),
            IraValue::Boolean(b) => vec![if *b { 1 } else { 0 }],
            IraValue::Money { amount, .. } => amount.to_le_bytes().to_vec(),
            IraValue::Rating(r) => vec![*r],
            IraValue::Year(y) => y.to_le_bytes().to_vec(),
            IraValue::TimeZone(tz) => {
                let mut bytes = vec![];
                bytes.push(tz.hours as u8);
                bytes.push(tz.minutes);
                bytes
            },
            _ => vec![], // TODO: Implement for remaining complex types
        }
    }
}