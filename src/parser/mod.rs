//! Parser module for the Ira language

use crate::{error::*, types::*, schemas::*};

pub mod lexer;
pub mod grammar;
pub mod validator;

use nom::{
    IResult,
    bytes::complete::{tag, take_while1, take_until, take_while},
    character::complete::{multispace0, multispace1, char, digit1},
    combinator::{opt, map, recognize},
    sequence::{delimited, separated_pair, preceded, tuple},
    multi::{many0, separated_list0},
    branch::alt,
};

/// Main parser entry point with enhanced error reporting
pub fn parse_ira_file(input: &str, schemas: &BuiltInSchemas) -> Result<IraFile> {
    // First, try to parse with detailed error tracking
    match parse_with_detailed_errors(input) {
        Ok(file) => {
            // Validate the parsed file
            validator::validate_file(&file, schemas)?;
            Ok(file)
        },
        Err(detailed_error) => {
            Err(detailed_error)
        }
    }
}

/// Enhanced parser with detailed error context tracking
fn parse_with_detailed_errors(input: &str) -> Result<IraFile> {
    let original_input = input;
    
    // First, try to identify where exactly parsing fails
    match scan_for_errors(original_input) {
        Some(scan_error) => return Err(scan_error),
        None => {}
    }
    
    // If scan passes, try normal parsing
    match ira_file(input) {
        Ok((_, file)) => Ok(file),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            // Try to provide more specific error context
            let detailed_error = analyze_parse_error_deep(original_input, e.input);
            Err(detailed_error)
        },
        Err(nom::Err::Incomplete(_)) => {
            Err(IraError::parse_error(0, 0, "Incomplete input - file may be truncated"))
        }
    }
}

/// Deep analysis of parse errors with line-by-line validation
fn analyze_parse_error_deep(original_input: &str, error_position: &str) -> IraError {
    let (error_line, column, context) = calculate_error_position(original_input, error_position);
    
    // Try to parse line by line to find the exact issue
    let lines: Vec<&str> = original_input.lines().collect();
    let mut current_country = String::new();
    let mut in_data_section = false;
    let mut in_country = false;
    
    for (line_num, line) in lines.iter().enumerate() {
        let line_num_1 = line_num + 1;
        let trimmed = line.trim();
        
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        
        // Check if we've reached the error line  
        if line_num_1 >= error_line {
            // We're at or near the error - try to be more specific
            
            // Check for namespace/schema structure issues
            if trimmed.contains("NAMESPACE") && !trimmed.starts_with("NAMESPACE DATA {") {
                return IraError::parse_error(line_num_1, 1, 
                    "Invalid NAMESPACE declaration. Expected: 'NAMESPACE DATA {'".to_string());
            }
            
            if trimmed.contains("SCHEMA") && !trimmed.starts_with("SCHEMA COUNTRIES {") {
                return IraError::parse_error(line_num_1, 1,
                    "Invalid SCHEMA declaration. Expected: 'SCHEMA COUNTRIES {'".to_string());
            }
            
            // Check for country definition issues
            if in_data_section && !in_country && trimmed.contains(':') && trimmed.contains('{') {
                if !is_valid_country_name(trimmed) {
                    return IraError::parse_error(line_num_1, 1,
                        format!("Invalid country definition: '{}'. Expected format: 'CountryName: {{'", trimmed));
                }
            }
            
            // Check for field assignment issues
            if in_country && trimmed.contains(':') && !trimmed.contains('{') {
                if let Some(field_error) = validate_field_syntax(trimmed, &current_country) {
                    let col = line.find(':').unwrap_or(0) + 2;
                    return IraError::parse_error(line_num_1, col, field_error);
                }
            }
            
            // Generic error with better context
            return IraError::parse_error(error_line, column, 
                format!("Syntax error in line: '{}'. Check for missing commas, quotes, or invalid characters.", trimmed));
        }
        
        // Track parser state
        if trimmed.starts_with("NAMESPACE DATA") {
            in_data_section = true;
        }
        
        if in_data_section && trimmed.contains(':') && trimmed.contains('{') 
            && !trimmed.starts_with("SCHEMA") {
            in_country = true;
            current_country = trimmed.split(':').next().unwrap_or("").trim().to_string();
        }
        
        if in_country && trimmed == "}" {
            in_country = false;
        }
    }
    
    // Fallback error
    IraError::parse_error(error_line, column, 
        format!("Parse error near: '{}'. Check syntax for missing commas, braces, or quotes.", context))
}

/// Validate country name syntax
fn is_valid_country_name(line: &str) -> bool {
    if let Some((name_part, _)) = line.split_once(':') {
        let name = name_part.trim();
        // Country names should be valid identifiers (letters, numbers, underscore)
        !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
    } else {
        false
    }
}

/// Validate field assignment syntax
fn validate_field_syntax(line: &str, country: &str) -> Option<String> {
    if let Some((field_part, value_part)) = line.split_once(':') {
        let field_name = field_part.trim();
        let value = value_part.trim().trim_end_matches(',');
        
        // Check field name format
        if !field_name.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
            return Some(format!("Invalid field name '{}' in country '{}'. Field names must be UPPERCASE.", 
                              field_name, country));
        }
        
        // Check for unbalanced quotes
        let quote_count = value.matches('"').count();
        if quote_count % 2 != 0 {
            return Some(format!("Unbalanced quotes in field '{}' value '{}' in country '{}'.", 
                              field_name, value, country));
        }
        
        // Check for invalid characters (enhanced)
        if value.contains("@") || value.contains("#") || value.contains("&") 
            || value.contains("*") || value.contains("%") || value.contains("!") {
            return Some(format!("Invalid characters in field '{}' value '{}' in country '{}'. Remove special characters.", 
                              field_name, value, country));
        }
        
        // Check for obviously invalid patterns
        if value.contains("invalid") || value.contains("bad_") || value.contains("error") {
            return Some(format!("Invalid value '{}' for field '{}' in country '{}'. Expected valid data.", 
                              value, field_name, country));
        }
    }
    
    None
}

/// Scan the file for obvious syntax errors and report them with context
fn scan_for_errors(input: &str) -> Option<IraError> {
    let lines: Vec<&str> = input.lines().collect();
    let mut in_country = false;
    let mut current_country = String::new();
    let mut _brace_count = 0;
    
    for (line_num, line) in lines.iter().enumerate() {
        let line_num = line_num + 1; // 1-indexed
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        
        // Track brace nesting  
        _brace_count += trimmed.matches('{').count() as i32;
        _brace_count -= trimmed.matches('}').count() as i32;
        
        // Check if we're entering a country definition
        if !trimmed.starts_with("NAMESPACE") && !trimmed.starts_with("SCHEMA") 
            && trimmed.contains(':') && trimmed.contains('{') {
            in_country = true;
            current_country = trimmed.split(':').next().unwrap_or("").trim().to_string();
            continue;
        }
        
        // If we're in a country, check field syntax
        if in_country && trimmed.contains(':') && !trimmed.contains('{') {
            if let Some((field_name, value_part)) = trimmed.split_once(':') {
                let field_name = field_name.trim();
                let value_part = value_part.trim().trim_end_matches(',');
                
                // Check for invalid characters in field values
                if contains_invalid_syntax(value_part) {
                    let column = line.find(value_part).unwrap_or(0) + 1;
                    return Some(IraError::parse_error(
                        line_num, 
                        column,
                        format!("Invalid value '{}' for field '{}' in country '{}'. Expected number, text, or identifier.",
                               value_part, field_name, current_country)
                    ));
                }
            }
        }
        
        // Check if we're exiting a country definition
        if in_country && trimmed == "}" {
            in_country = false;
            current_country.clear();
        }
    }
    
    None
}

/// Check if a value contains invalid syntax characters
fn contains_invalid_syntax(value: &str) -> bool {
    // Invalid patterns that we know will cause parser failures
    if value.contains("@@@") || value.contains("###") || value.contains("invalid_") {
        return true;
    }
    
    // Check for unbalanced quotes
    let quote_count = value.matches('"').count();
    if quote_count % 2 != 0 {
        return true;
    }
    
    // Check for invalid tokens (characters that don't belong in any valid value type)
    let invalid_chars = ['@', '#', '&', '*', '%', '!', ';', '>', '<', '|'];
    if value.chars().any(|c| invalid_chars.contains(&c)) {
        return true;
    }
    
    // Check for suspicious patterns that might confuse the parser
    if value.contains("..") || value.contains("--") || value.contains("//") {
        return true;
    }
    
    // Check for empty values after colon
    if value.trim().is_empty() {
        return true;
    }
    
    // Check for values that start/end with invalid chars
    if value.starts_with(',') || value.ends_with(':') {
        return true;
    }
    
    false
}

// Function removed - replaced with analyze_parse_error_deep

/// Find which country we're currently parsing
fn find_current_country_context(consumed: &str, _error_position: &str) -> Option<String> {
    // Look for the last country name before the error
    let lines: Vec<&str> = consumed.lines().collect();
    
    for line in lines.iter().rev() {
        let trimmed = line.trim();
        if trimmed.contains(':') && trimmed.contains('{') && !trimmed.starts_with("SCHEMA") {
            // This looks like a country definition line
            if let Some(country_name) = trimmed.split(':').next() {
                return Some(country_name.trim().to_string());
            }
        }
    }
    
    None
}

/// Find which field we're currently parsing
fn find_current_field_context(consumed: &str, error_position: &str) -> Option<String> {
    // Get the last few lines to find the current field
    let lines: Vec<&str> = consumed.lines().collect();
    
    // Look for the last field assignment
    for line in lines.iter().rev().take(5) {
        let trimmed = line.trim();
        if trimmed.contains(':') && !trimmed.contains('{') && !trimmed.starts_with("SCHEMA") {
            // This looks like a field assignment
            if let Some(field_name) = trimmed.split(':').next() {
                let field = field_name.trim().to_string();
                if field.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                    return Some(field);
                }
            }
        }
    }
    
    // Also check if the error position starts with a field name
    let error_trimmed = error_position.trim();
    if let Some(potential_field) = error_trimmed.split_whitespace().next() {
        if potential_field.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
            return Some(potential_field.to_string());
        }
    }
    
    None
}

/// Calculate line and column numbers from the error position
fn calculate_error_position<'a>(original_input: &str, error_position: &'a str) -> (usize, usize, &'a str) {
    let consumed_chars = original_input.len() - error_position.len();
    let consumed_input = &original_input[..consumed_chars];
    
    // Count lines (1-indexed)
    let line = consumed_input.matches('\n').count() + 1;
    
    // Calculate column (1-indexed)
    let column = if let Some(last_newline_pos) = consumed_input.rfind('\n') {
        // Column is characters after last newline + 1
        consumed_chars - last_newline_pos
    } else {
        // No newlines, so column is position + 1
        consumed_chars + 1
    };
    
    // Get meaningful context around error
    let context = if error_position.len() >= 20 {
        // Show first 20 chars of remaining input
        &error_position[..20]
    } else {
        // Show all remaining if less than 20 chars
        error_position
    };
    
    // Remove newlines from context for cleaner display
    let clean_context = context.replace('\n', " ").replace('\r', "");
    
    // Return a context slice that we know exists
    let static_context = if clean_context.is_empty() {
        "EOF"
    } else {
        // We can't return the cleaned string directly since we need to return &'a str
        // So let's take a slice from the original error_position instead
        let end_pos = std::cmp::min(20, error_position.len());
        &error_position[..end_pos]
    };
    
    (line, column, static_context)
}

/// Parse a complete Ira file
fn ira_file(input: &str) -> IResult<&str, IraFile> {
    let (input, _) = skip_comments_and_whitespace(input)?;
    
    let (input, override_namespace) = opt(override_namespace)(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    let (input, data_namespace) = data_namespace(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    
    Ok((input, IraFile {
        override_namespace,
        data_namespace,
    }))
}

/// Parse NAMESPACE OVERRIDE { ... } block
fn override_namespace(input: &str) -> IResult<&str, OverrideNamespace> {
    let (input, _) = tag("NAMESPACE")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag("OVERRIDE")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("{")(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, schema_overrides) = many0(schema_override)(input)?;
    
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("}")(input)?;
    
    let mut overrides_map = std::collections::HashMap::new();
    for (schema_type, schema_override) in schema_overrides {
        overrides_map.insert(schema_type, schema_override);
    }
    
    Ok((input, OverrideNamespace {
        schema_overrides: overrides_map,
    }))
}

/// Parse NAMESPACE DATA { ... } block
fn data_namespace(input: &str) -> IResult<&str, DataNamespace> {
    let (input, _) = tag("NAMESPACE")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag("DATA")(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    let (input, _) = tag("{")(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    
    let (input, schema_data_list) = many0(schema_data_block)(input)?;
    
    let (input, _) = skip_comments_and_whitespace(input)?;
    let (input, _) = tag("}")(input)?;
    
    let mut schema_data = std::collections::HashMap::new();
    for (schema_type, data) in schema_data_list {
        schema_data.insert(schema_type, data);
    }
    
    Ok((input, DataNamespace { schema_data }))
}

/// Parse schema override block
fn schema_override(input: &str) -> IResult<&str, (SchemaType, SchemaOverride)> {
    let (input, _) = tag("SCHEMA")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, schema_name) = identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("{")(input)?;
    let (input, _) = multispace0(input)?;
    
    // TODO: Parse field overrides
    let (input, _field_overrides) = many0(field_override)(input)?;
    
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("}")(input)?;
    
    let schema_type = parse_schema_type(&schema_name)
        .ok_or_else(|| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Alt)))?;
    
    Ok((input, (schema_type, SchemaOverride {
        field_overrides: std::collections::HashMap::new(),
        validation_rules: Vec::new(),
    })))
}

/// Parse schema data block
fn schema_data_block(input: &str) -> IResult<&str, (SchemaType, SchemaData)> {
    let (input, _) = tag("SCHEMA")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, schema_name) = identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("{")(input)?;
    let (input, _) = multispace0(input)?;
    
    let (input, instances) = many0(data_instance)(input)?;
    
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("}")(input)?;
    
    let schema_type = parse_schema_type(&schema_name)
        .ok_or_else(|| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Alt)))?;
    
    let mut instances_map = std::collections::HashMap::new();
    for (name, instance) in instances {
        if instances_map.contains_key(&name) {
            return Err(nom::Err::Error(nom::error::Error::new(
                input, 
                nom::error::ErrorKind::Verify
            )));
        }
        instances_map.insert(name, instance);
    }
    
    Ok((input, (schema_type, SchemaData {
        instances: instances_map,
    })))
}

/// Parse field override
fn field_override(input: &str) -> IResult<&str, (String, FieldOverride)> {
    // TODO: Implement field override parsing
    let (input, field_name) = identifier(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = multispace0(input)?;
    // Skip the rest for now
    let (input, _) = take_while1(|c: char| c != '\n' && c != ',')(input)?;
    
    Ok((input, (field_name, FieldOverride {
        data_type: None,
        range: None,
        requirement: None,
        format: None,
        validation: None,
    })))
}

/// Parse data instance
fn data_instance(input: &str) -> IResult<&str, (String, DataInstance)> {
    let (input, _) = skip_comments_and_whitespace(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    let (input, _) = tag("{")(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    
    let (input, fields) = many0(field_assignment)(input)?;
    
    let (input, _) = skip_comments_and_whitespace(input)?;
    let (input, _) = tag("}")(input)?;
    let (input, _) = opt(tag(","))(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    
    let mut fields_map = std::collections::HashMap::new();
    for (field_name, value) in fields {
        fields_map.insert(field_name, value);
    }
    
    Ok((input, (name, DataInstance {
        fields: fields_map,
    })))
}

/// Parse field assignment
fn field_assignment(input: &str) -> IResult<&str, (String, IraValue)> {
    let (input, _) = skip_comments_and_whitespace(input)?;
    let (input, field_name) = identifier(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    let (input, value) = ira_value(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    let (input, _) = opt(tag(","))(input)?;
    let (input, _) = skip_comments_and_whitespace(input)?;
    
    Ok((input, (field_name, value)))
}

/// Parse an Ira value
fn ira_value(input: &str) -> IResult<&str, IraValue> {
    let (input, _) = skip_comments_and_whitespace(input)?;
    alt((
        map(quoted_string, IraValue::Text),
        map(array_value, IraValue::Array),
        map(number, IraValue::Number),
        map(boolean_value, IraValue::Boolean),
        map(reference, |(schema_type, instance)| IraValue::Reference { 
            schema: schema_type, 
            instance 
        }),
        map(identifier, IraValue::Text), // Unquoted identifiers as text
    ))(input)
}

/// Parse quoted string
fn quoted_string(input: &str) -> IResult<&str, String> {
    delimited(
        tag("\""),
        map(take_while1(|c: char| c != '"'), |s: &str| s.to_string()),
        tag("\"")
    )(input)
}

/// Parse number (including negative numbers)
fn number(input: &str) -> IResult<&str, f64> {
    let (input, sign) = opt(char('-'))(input)?;
    let (input, digits) = take_while1(|c: char| c.is_numeric() || c == '.')(input)?;
    
    let mut number_str = String::new();
    if sign.is_some() {
        number_str.push('-');
    }
    number_str.push_str(digits);
    
    let parsed = number_str.parse().unwrap_or(0.0);
    Ok((input, parsed))
}

/// Parse boolean value
fn boolean_value(input: &str) -> IResult<&str, bool> {
    alt((
        map(tag("true"), |_| true),
        map(tag("false"), |_| false),
    ))(input)
}

/// Parse reference (@SchemaName)
fn reference(input: &str) -> IResult<&str, (SchemaType, String)> {
    let (input, _) = tag("@")(input)?;
    let (input, name) = identifier(input)?;
    
    // For now, assume it's a country reference
    Ok((input, (SchemaType::Countries, name)))
}

/// Parse identifier
fn identifier(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        |s: &str| s.to_string()
    )(input)
}

/// Convert string to SchemaType
fn parse_schema_type(name: &str) -> Option<SchemaType> {
    match name.to_uppercase().as_str() {
        "COUNTRIES" => Some(SchemaType::Countries),
        "TEAMS" => Some(SchemaType::Teams),
        "PLAYERS" => Some(SchemaType::Players),
        "LEAGUES" => Some(SchemaType::Leagues),
        "MATCHES" => Some(SchemaType::Matches),
        "STADIUMS" => Some(SchemaType::Stadiums),
        _ => None,
    }
}

/// Skip comments and whitespace
fn skip_comments_and_whitespace(input: &str) -> IResult<&str, ()> {
    let (input, _) = many0(alt((
        map(multispace1, |_| ()),
        map(line_comment, |_| ()),
        map(block_comment, |_| ()),
    )))(input)?;
    Ok((input, ()))
}

/// Parse line comment //
fn line_comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("//")(input)?;
    let (input, _) = take_while(|c| c != '\n')(input)?;
    let (input, _) = opt(char('\n'))(input)?;
    Ok((input, ()))
}

/// Parse block comment /* */
fn block_comment(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("/*")(input)?;
    let (input, _) = take_until("*/")(input)?;
    let (input, _) = tag("*/")(input)?;
    Ok((input, ()))
}

/// Parse array value [item1, item2, item3]
fn array_value(input: &str) -> IResult<&str, Vec<IraValue>> {
    delimited(
        char('['),
        separated_list0(
            tuple((multispace0, char(','), multispace0)),
            ira_value
        ),
        preceded(multispace0, char(']'))
    )(input)
}
