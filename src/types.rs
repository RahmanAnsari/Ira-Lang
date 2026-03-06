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
    UUID,
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
    UUID(String),  // Store as string for now (could be uuid::Uuid later)
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

/// ID validation utilities for Ira entities
pub struct IraIdValidator;

impl IraIdValidator {
    /// Check if a string is a valid Country ID format (COUN1000-1999)
    pub fn is_valid_country_id(id_str: &str) -> bool {
        if !id_str.starts_with("COUN") {
            return false;
        }
        
        if id_str.len() != 8 {  // "COUN" + 4 digits = 8 chars
            return false;
        }
        
        let number_part = &id_str[4..];
        if let Ok(number) = number_part.parse::<u16>() {
            number >= 1000 && number <= 1999
        } else {
            false
        }
    }
    
    /// Check if a string is a valid League ID format (LEAG10000-19999)
    pub fn is_valid_league_id(id_str: &str) -> bool {
        if !id_str.starts_with("LEAG") {
            return false;
        }
        
        if id_str.len() != 9 {  // "LEAG" + 5 digits = 9 chars
            return false;
        }
        
        let number_part = &id_str[4..];
        if let Ok(number) = number_part.parse::<u32>() {
            number >= 10000 && number <= 19999
        } else {
            false
        }
    }
    
    /// Check if a string is a valid UUID format (for other entities)
    pub fn is_valid_uuid(uuid_str: &str) -> bool {
        // Basic UUID format validation: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        if uuid_str.len() != 36 {
            return false;
        }
        
        let parts: Vec<&str> = uuid_str.split('-').collect();
        if parts.len() != 5 {
            return false;
        }
        
        // Check each part length: 8-4-4-4-12
        if parts[0].len() != 8 || parts[1].len() != 4 || 
           parts[2].len() != 4 || parts[3].len() != 4 || parts[4].len() != 12 {
            return false;
        }
        
        // Check all characters are valid hex
        for part in parts {
            if !part.chars().all(|c| c.is_ascii_hexdigit()) {
                return false;
            }
        }
        
        true
    }
    
    /// Generate a new Country ID (COUN1000-1999)
    pub fn generate_country_id() -> String {
        let mut rng_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        // Generate random number in range 1000-1999
        let number = 1000 + ((rng_seed % 1000) as u16);
        format!("COUN{}", number)
    }
    
    /// Generate a new League ID (LEAG10000-19999)
    pub fn generate_league_id() -> String {
        let mut rng_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        // Generate random number in range 10000-19999
        let number = 10000 + ((rng_seed % 10000) as u32);
        format!("LEAG{}", number)
    }
    
    /// Generate a new UUID v4 (for other entities)
    pub fn generate_uuid() -> String {
        let mut rng_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        // Simple random UUID v4 generation
        let mut uuid = String::with_capacity(36);
        
        for i in 0..32 {
            if i == 8 || i == 12 || i == 16 || i == 20 {
                uuid.push('-');
            }
            
            // Very simple random hex digit
            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let hex_char = format!("{:x}", (rng_seed >> 28) & 0xf);
            uuid.push_str(&hex_char);
        }
        
        // Set version (4) and variant bits
        uuid.replace_range(14..15, "4");  // Version 4
        let variant_chars = ['8', '9', 'a', 'b'];
        let variant_idx = ((rng_seed >> 4) & 3) as usize;
        uuid.replace_range(19..20, &variant_chars[variant_idx].to_string());
        
        uuid
    }
    
    /// Check if a string is a valid Stadium ID format (STAD13000-13999)
    pub fn is_valid_stadium_id(id_str: &str) -> bool {
        if !id_str.starts_with("STAD") {
            return false;
        }
        
        if id_str.len() != 9 {  // "STAD" + 5 digits = 9 chars
            return false;
        }
        
        let number_part = &id_str[4..];
        if let Ok(number) = number_part.parse::<u32>() {
            number >= 13000 && number <= 13999
        } else {
            false
        }
    }
    
    /// Generate a new Stadium ID (STAD13000-13999)
    pub fn generate_stadium_id() -> String {
        let mut rng_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        // Generate random number in range 13000-13999
        let number = 13000 + ((rng_seed % 1000) as u32);
        format!("STAD{}", number)
    }
    
    /// Check if a string is a valid Team ID format (TEM10000-19999)
    pub fn is_valid_team_id(id_str: &str) -> bool {
        if !id_str.starts_with("TEM") {
            return false;
        }
        
        if id_str.len() != 8 {  // "TEM" + 5 digits = 8 chars
            return false;
        }
        
        let number_part = &id_str[3..];
        if let Ok(number) = number_part.parse::<u32>() {
            number >= 10000 && number <= 19999
        } else {
            false
        }
    }
    
    /// Generate a new Team ID (TEM10000-19999)
    pub fn generate_team_id() -> String {
        let mut rng_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        // Generate random number in range 10000-19999
        let number = 10000 + ((rng_seed % 10000) as u32);
        format!("TEM{}", number)
    }
    
    /// Validate any ID based on context (determines if it's Country ID, League ID, Stadium ID, Team ID, or UUID)
    pub fn validate_id(id_str: &str, schema_type: &SchemaType) -> bool {
        match schema_type {
            SchemaType::Countries => Self::is_valid_country_id(id_str),
            SchemaType::Leagues => Self::is_valid_league_id(id_str),
            SchemaType::Stadiums => Self::is_valid_stadium_id(id_str),
            SchemaType::Teams => Self::is_valid_team_id(id_str),
            _ => Self::is_valid_uuid(id_str),
        }
    }
    
    /// Convert UUID string to canonical format (lowercase with dashes)
    pub fn normalize_uuid(uuid_str: &str) -> Result<String, String> {
        let cleaned = uuid_str.replace(&['-', ' '], "").to_lowercase();
        
        if cleaned.len() != 32 || !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("Invalid UUID format: {}", uuid_str));
        }
        
        // Insert dashes at correct positions
        Ok(format!("{}-{}-{}-{}-{}", 
            &cleaned[0..8],
            &cleaned[8..12], 
            &cleaned[12..16],
            &cleaned[16..20],
            &cleaned[20..32]
        ))
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
            IraValue::UUID(_) => DataType::UUID,
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
            IraValue::UUID(uuid_str) => {
                // Store UUID as 16-byte binary format
                uuid_str.as_bytes().to_vec()  // Simple string storage for now
            },
            _ => vec![], // TODO: Implement for remaining complex types
        }
    }
}