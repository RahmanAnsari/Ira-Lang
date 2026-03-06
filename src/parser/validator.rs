//! Validation logic for parsed Ira files

use crate::{error::*, types::*, schemas::*};

/// Validate a complete Ira file
pub fn validate_file(file: &IraFile, schemas: &BuiltInSchemas) -> Result<()> {
    // Validate override namespace if present
    if let Some(override_ns) = &file.override_namespace {
        validate_override_namespace(override_ns, schemas)?;
    }
    
    // Validate data namespace
    validate_data_namespace(&file.data_namespace, schemas)?;
    
    Ok(())
}

/// Validate override namespace
fn validate_override_namespace(override_ns: &OverrideNamespace, schemas: &BuiltInSchemas) -> Result<()> {
    for (schema_type, schema_override) in &override_ns.schema_overrides {
        let schema_def = schemas.get_schema(schema_type);
        validate_schema_override(schema_override, schema_def)?;
    }
    Ok(())
}

/// Validate data namespace
fn validate_data_namespace(data_ns: &DataNamespace, schemas: &BuiltInSchemas) -> Result<()> {
    // First, validate each schema individually
    for (schema_type, schema_data) in &data_ns.schema_data {
        let schema_def = schemas.get_schema(schema_type);
        validate_schema_data(schema_data, schema_def, schema_type)?;
    }
    
    // Then, validate document-level constraints
    validate_document_structure(data_ns)?;
    
    Ok(())
}

/// Validate document-level structure constraints
fn validate_document_structure(data_ns: &DataNamespace) -> Result<()> {
    let has_countries = data_ns.schema_data.contains_key(&SchemaType::Countries);
    let has_leagues = data_ns.schema_data.contains_key(&SchemaType::Leagues);
    
    // Rule: If leagues exist, countries must exist
    if has_leagues && !has_countries {
        return Err(IraError::validation_error(
            "Document Structure",
            "Leagues require at least one Country to be defined. Add a Countries schema.".to_string()
        ));
    }
    
    // Rule: Validate league country references
    if has_leagues && has_countries {
        validate_league_country_references(data_ns)?;
    }
    
    Ok(())
}

/// Validate that all league countryId references point to existing countries
fn validate_league_country_references(data_ns: &DataNamespace) -> Result<()> {
    // Get all country IDs
    let countries = data_ns.schema_data.get(&SchemaType::Countries).unwrap();
    let mut country_ids = std::collections::HashSet::new();
    
    for instance in countries.instances.values() {
        if let Some(IraValue::UUID(country_id)) = instance.fields.get("ID") {
            country_ids.insert(country_id.clone());
        }
    }
    
    // Check all league country references
    if let Some(leagues) = data_ns.schema_data.get(&SchemaType::Leagues) {
        for (league_name, league_instance) in &leagues.instances {
            if let Some(IraValue::UUID(country_id)) = league_instance.fields.get("countryId") {
                if !country_ids.contains(country_id) {
                    return Err(IraError::validation_error(
                        &format!("League '{}'", league_name),
                        format!("Invalid countryId reference '{}'. No country with this ID exists.", country_id)
                    ));
                }
            }
        }
    }
    
    Ok(())
}

/// Validate schema override
fn validate_schema_override(schema_override: &SchemaOverride, schema_def: &SchemaDefinition) -> Result<()> {
    // Check that all overridden fields exist in the schema
    for field_name in schema_override.field_overrides.keys() {
        if !schema_def.fields.contains_key(field_name) {
            return Err(IraError::unknown_field(field_name, &schema_def.name));
        }
    }
    
    // Validate validation rules
    for rule in &schema_override.validation_rules {
        validate_validation_rule(rule, schema_def)?;
    }
    
    Ok(())
}

/// Validate schema data
fn validate_schema_data(schema_data: &SchemaData, schema_def: &SchemaDefinition, schema_type: &SchemaType) -> Result<()> {
    // Check for duplicate unique field values (like country codes)
    validate_unique_fields(schema_data, schema_type)?;
    
    for (instance_name, instance) in &schema_data.instances {
        validate_data_instance(instance, schema_def, schema_type, instance_name)?;
    }
    Ok(())
}

/// Validate unique fields to prevent duplicates
fn validate_unique_fields(schema_data: &SchemaData, schema_type: &SchemaType) -> Result<()> {
    match schema_type {
        SchemaType::Countries => {
            validate_country_unique_fields(schema_data)?;
        },
        SchemaType::Teams => {
            validate_team_unique_fields(schema_data)?;
        },
        _ => {
            // Other schemas don't have specific unique field requirements yet
        }
    }
    Ok(())
}

/// Validate unique fields for countries
fn validate_country_unique_fields(schema_data: &SchemaData) -> Result<()> {
    let mut seen_codes = std::collections::HashMap::new();
    let mut seen_names = std::collections::HashMap::new();
    
    for (instance_name, instance) in &schema_data.instances {
        // Check for duplicate country codes
        if let Some(code_value) = instance.fields.get("CODE") {
            if let IraValue::Text(code) = code_value {
                if let Some(existing_country) = seen_codes.insert(code.clone(), instance_name.clone()) {
                    return Err(IraError::validation_error(
                        "CODE",
                        format!("Duplicate country code '{}' found in '{}' and '{}'", code, existing_country, instance_name)
                    ));
                }
            }
        }
        
        // Check for duplicate country names
        if let Some(name_value) = instance.fields.get("NAME") {
            if let IraValue::Text(name) = name_value {
                if let Some(existing_country) = seen_names.insert(name.clone(), instance_name.clone()) {
                    return Err(IraError::validation_error(
                        "NAME",
                        format!("Duplicate country name '{}' found in '{}' and '{}'", name, existing_country, instance_name)
                    ));
                }
            }
        }
    }
    
    Ok(())
}

/// Validate unique fields for teams
fn validate_team_unique_fields(_schema_data: &SchemaData) -> Result<()> {
    // TODO: Add team-specific unique field validation
    Ok(())
}

/// Validate a data instance
fn validate_data_instance(
    instance: &DataInstance, 
    schema_def: &SchemaDefinition, 
    _schema_type: &SchemaType,
    _instance_name: &str
) -> Result<()> {
    // Check required fields are present
    for required_field in &schema_def.required_fields {
        if !instance.fields.contains_key(required_field) {
            return Err(IraError::required_field_missing(required_field, &schema_def.name));
        }
    }
    
    // Validate each field
    for (field_name, field_value) in &instance.fields {
        if let Some(field_def) = schema_def.fields.get(field_name) {
            validate_field_value(field_value, &field_def.data_type, field_name)?;
        } else {
            return Err(IraError::unknown_field(field_name, &schema_def.name));
        }
    }
    
    Ok(())
}

/// Validate a field value against its expected type
fn validate_field_value(value: &IraValue, expected_type: &DataType, field_name: &str) -> Result<()> {
    match (value, expected_type) {
        (IraValue::Text(s), DataType::Text { max_length }) => {
            if let Some(max_len) = max_length {
                let char_count = s.chars().count();
                if char_count > *max_len {
                    return Err(IraError::validation_error(
                        field_name,
                        format!("Text length {} characters exceeds maximum {}", char_count, max_len)
                    ));
                }
            }
        },
        
        (IraValue::Number(_), DataType::Number) => {
            // Numbers are always valid
        },
        
        // Handle numbers that should be ratings
        (IraValue::Number(n), DataType::Rating { min, max }) => {
            let rating_value = *n as u8;
            if rating_value < *min || rating_value > *max {
                return Err(IraError::range_validation_error(*n as i32, *min as i32, *max as i32));
            }
        },
        
        (IraValue::Rating(r), DataType::Rating { min, max }) => {
            if *r < *min || *r > *max {
                return Err(IraError::range_validation_error(*r as i32, *min as i32, *max as i32));
            }
        },
        
        (IraValue::Boolean(_), DataType::Boolean) => {
            // Booleans are always valid
        },
        
        (IraValue::Choice(choice), DataType::Choice { options }) => {
            if !options.contains(choice) {
                return Err(IraError::choice_validation_error(choice, options.clone()));
            }
        },
        
        (IraValue::Year(year), DataType::Year) => {
            // Basic year validation
            if *year < 1800 || *year > 2100 {
                return Err(IraError::validation_error(
                    field_name,
                    format!("Year {} is outside valid range 1800-2100", year)
                ));
            }
        },
        
        // Handle numbers that should be years  
        (IraValue::Number(n), DataType::Year) => {
            let year_value = *n as u16;
            if year_value < 1800 || year_value > 2100 {
                return Err(IraError::validation_error(
                    field_name,
                    format!("Year {} is outside valid range 1800-2100", year_value)
                ));
            }
        },
        
        // Handle numbers that should be money
        (IraValue::Number(n), DataType::Money { .. }) => {
            if *n < 0.0 {
                return Err(IraError::validation_error(
                    field_name,
                    "Money values cannot be negative".to_string()
                ));
            }
        },
        
        (IraValue::Money { .. }, DataType::Money { .. }) => {
            // Money validation could check for negative values, etc.
        },
        
        // Handle arrays
        (IraValue::Array(arr), DataType::Array { element_type, max_size }) => {
            if let Some(max) = max_size {
                if arr.len() > *max {
                    return Err(IraError::validation_error(
                        field_name,
                        format!("Array length {} exceeds maximum {}", arr.len(), max)
                    ));
                }
            }
            
            // Validate each element in the array
            for (i, element) in arr.iter().enumerate() {
                validate_field_value(element, element_type, &format!("{}[{}]", field_name, i))?;
            }
        },
        
        // Handle ranges (numbers that should be in a range)
        (IraValue::Number(n), DataType::Range { min, max }) => {
            let value = *n as i32;
            if value < *min || value > *max {
                return Err(IraError::range_validation_error(value, *min, *max));
            }
        },
        
        // Handle unquoted identifiers as choices
        (IraValue::Text(choice), DataType::Choice { options }) => {
            if !options.contains(choice) {
                return Err(IraError::choice_validation_error(choice, options.clone()));
            }
        },
        
        // Handle TimeZone values
        (IraValue::TimeZone(_), DataType::TimeZone) => {
            // TimeZone values are already validated in their constructor
        },
        
        // Allow numbers to be treated as timezones (backwards compatibility)
        (IraValue::Number(n), DataType::TimeZone) => {
            if let Err(e) = crate::types::TimeZone::from_decimal(*n) {
                return Err(IraError::validation_error(field_name, e));
            }
        },
        
        // Handle UUID values (including Country IDs, League IDs, Stadium IDs, and Team IDs)
        (IraValue::UUID(id_str), DataType::UUID) => {
            use crate::types::IraIdValidator;
            
            // Check if it's a Country ID, League ID, Stadium ID, Team ID, or regular UUID
            let is_valid = if id_str.starts_with("COUN") {
                IraIdValidator::is_valid_country_id(id_str)
            } else if id_str.starts_with("LEAG") {
                IraIdValidator::is_valid_league_id(id_str)
            } else if id_str.starts_with("STAD") {
                IraIdValidator::is_valid_stadium_id(id_str)
            } else if id_str.starts_with("TEM") {
                IraIdValidator::is_valid_team_id(id_str)
            } else {
                IraIdValidator::is_valid_uuid(id_str)
            };
            
            if !is_valid {
                let expected_format = if field_name == "ID" || field_name == "leagueId" || field_name == "stadiumId" || field_name == "teamId" {
                    // For schema ID fields, determine expected format by prefix
                    if id_str.starts_with("COUN") || field_name == "countryId" {
                        "COUN1000-1999 (e.g., COUN1232)"
                    } else if id_str.starts_with("LEAG") || field_name == "leagueId" {
                        "LEAG10000-19999 (e.g., LEAG12345)"
                    } else if id_str.starts_with("STAD") || field_name == "stadiumId" {
                        "STAD13000-13999 (e.g., STAD13001)"
                    } else if id_str.starts_with("TEM") || field_name == "teamId" {
                        "TEM10000-19999 (e.g., TEM14001)"
                    } else {
                        "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                    }
                } else {
                    // For other UUIDs, expect standard format
                    "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                };
                
                return Err(IraError::validation_error(
                    field_name,
                    format!("Invalid ID format: '{}'. Expected format: {}", id_str, expected_format)
                ));
            }
        },
        
        _ => {
            return Err(IraError::type_mismatch(
                format!("{:?}", expected_type),
                format!("{:?}", value.data_type())
            ));
        }
    }
    
    Ok(())
}

/// Validate a validation rule
fn validate_validation_rule(rule: &ValidationRule, schema_def: &SchemaDefinition) -> Result<()> {
    match rule {
        ValidationRule::FieldComparison { field1, field2, .. } => {
            if !schema_def.fields.contains_key(field1) {
                return Err(IraError::unknown_field(field1, &schema_def.name));
            }
            if !schema_def.fields.contains_key(field2) {
                return Err(IraError::unknown_field(field2, &schema_def.name));
            }
        },
        
        ValidationRule::ConditionalRequired { condition_field, required_fields, .. } => {
            if !schema_def.fields.contains_key(condition_field) {
                return Err(IraError::unknown_field(condition_field, &schema_def.name));
            }
            for req_field in required_fields {
                if !schema_def.fields.contains_key(req_field) {
                    return Err(IraError::unknown_field(req_field, &schema_def.name));
                }
            }
        }
    }
    
    Ok(())
}