//! Compiler module for generating binary output

use crate::{error::*, types::*};
use brotli::CompressorReader;
use std::io::Read;

/// Compile an Ira AST to binary format with brotli compression
pub fn compile_to_binary(file: &IraFile, config: &LanguageConfig) -> Result<Vec<u8>> {
    let mut binary_output = Vec::new();
    
    // Write binary header
    write_header(&mut binary_output, config)?;
    
    // Write string table
    let string_table = build_string_table(file)?;
    write_string_table(&mut binary_output, &string_table)?;
    
    // Write schema data
    write_data_sections(&mut binary_output, file, &string_table)?;
    
    // Apply brotli compression (level 11 = maximum compression)
    let mut compressor = CompressorReader::new(&binary_output[..], 4096, 11, 22);
    let mut compressed_output = Vec::new();
    compressor.read_to_end(&mut compressed_output)
        .map_err(|e| IraError::compilation_error(&format!("Brotli compression failed: {}", e)))?;
    
    Ok(compressed_output)
}

/// Write binary header
fn write_header(output: &mut Vec<u8>, config: &LanguageConfig) -> Result<()> {
    // Magic number "IRAB" (Ira Binary)
    output.extend_from_slice(b"IRAB");
    
    // Version
    output.extend_from_slice(&config.binary_format_version.to_le_bytes());
    
    // Placeholder for string table size (will be updated)
    output.extend_from_slice(&0u32.to_le_bytes());
    
    // Placeholder for data sections offset (will be updated)
    output.extend_from_slice(&0u32.to_le_bytes());
    
    Ok(())
}

/// Build string table from file
fn build_string_table(file: &IraFile) -> Result<Vec<String>> {
    let mut strings = std::collections::HashSet::new();
    
    // Collect all strings from data
    for schema_data in file.data_namespace.schema_data.values() {
        // Add instance names to string table
        for instance_name in schema_data.instances.keys() {
            strings.insert(instance_name.clone());
        }
        
        // Add field names to string table
        for instance in schema_data.instances.values() {
            for field_name in instance.fields.keys() {
                strings.insert(field_name.clone());
            }
            for value in instance.fields.values() {
                collect_strings_from_value(value, &mut strings);
            }
        }
    }
    
    Ok(strings.into_iter().collect())
}

/// Collect strings from a value recursively
fn collect_strings_from_value(value: &IraValue, strings: &mut std::collections::HashSet<String>) {
    match value {
        IraValue::Text(s) => {
            strings.insert(s.clone());
        },
        IraValue::Choice(s) => {
            strings.insert(s.clone());
        },
        IraValue::Reference { instance, .. } => {
            strings.insert(instance.clone());
        },
        IraValue::Array(arr) => {
            for item in arr {
                collect_strings_from_value(item, strings);
            }
        },
        IraValue::UUID(uuid_str) => {
            strings.insert(uuid_str.clone());
        },
        _ => {}, // Other types don't contain strings
    }
}

/// Write string table to output
fn write_string_table(output: &mut Vec<u8>, strings: &[String]) -> Result<()> {
    // Write string count
    output.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    
    // Write each string
    for string in strings {
        let bytes = string.as_bytes();
        output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        output.extend_from_slice(bytes);
    }
    
    Ok(())
}

/// Write data sections
fn write_data_sections(output: &mut Vec<u8>, file: &IraFile, string_table: &[String]) -> Result<()> {
    for (schema_type, schema_data) in &file.data_namespace.schema_data {
        write_data_section(output, schema_type, schema_data, string_table)?;
    }
    Ok(())
}

/// Write a single data section
fn write_data_section(
    output: &mut Vec<u8>, 
    schema_type: &SchemaType, 
    schema_data: &SchemaData,
    string_table: &[String]
) -> Result<()> {
    // Write schema type ID
    let schema_id = match schema_type {
        SchemaType::Countries => 1u32,
        SchemaType::Teams => 2u32,
        SchemaType::Players => 3u32,
        SchemaType::Leagues => 4u32,
        SchemaType::Matches => 5u32,
        SchemaType::Stadiums => 6u32,
    };
    output.extend_from_slice(&schema_id.to_le_bytes());
    
    // Write instance count
    output.extend_from_slice(&(schema_data.instances.len() as u32).to_le_bytes());
    
    // Write instances
    for (instance_name, instance) in &schema_data.instances {
        write_instance_data(output, instance_name, instance, string_table)?;
    }
    
    Ok(())
}

/// Write instance data
fn write_instance_data(
    output: &mut Vec<u8>, 
    instance_name: &String,
    instance: &DataInstance,
    string_table: &[String]
) -> Result<()> {
    // Write instance name as string table index
    let name_index = string_table.iter()
        .position(|s| s == instance_name)
        .unwrap_or(0) as u32;
    output.extend_from_slice(&name_index.to_le_bytes());
    
    // Write field count
    output.extend_from_slice(&(instance.fields.len() as u32).to_le_bytes());
    
    // Write each field
    for (field_name, value) in &instance.fields {
        // Write field name as string table index
        let name_index = string_table.iter()
            .position(|s| s == field_name)
            .unwrap_or(0) as u32;
        output.extend_from_slice(&name_index.to_le_bytes());
        
        // Write value
        write_value_data(output, value, string_table)?;
    }
    
    Ok(())
}

/// Write value data
fn write_value_data(
    output: &mut Vec<u8>, 
    value: &IraValue,
    string_table: &[String]
) -> Result<()> {
    // Write value type
    let value_type = match value {
        IraValue::Text(_) => 1u8,
        IraValue::Number(_) => 2u8,
        IraValue::Integer(_) => 3u8,
        IraValue::Boolean(_) => 4u8,
        IraValue::Money { .. } => 5u8,
        IraValue::Rating(_) => 6u8,
        IraValue::Year(_) => 7u8,
        IraValue::Reference { .. } => 8u8,
        IraValue::Array(_) => 9u8,
        IraValue::Choice(_) => 10u8,
        IraValue::TimeZone(_) => 11u8,
        IraValue::UUID(_) => 12u8,
    };
    output.push(value_type);
    
    // Write value data
    match value {
        IraValue::Text(s) => {
            let index = string_table.iter()
                .position(|st| st == s)
                .unwrap_or(0) as u32;
            output.extend_from_slice(&index.to_le_bytes());
        },
        IraValue::Number(n) => {
            output.extend_from_slice(&n.to_le_bytes());
        },
        IraValue::Integer(i) => {
            output.extend_from_slice(&i.to_le_bytes());
        },
        IraValue::Boolean(b) => {
            output.push(if *b { 1 } else { 0 });
        },
        IraValue::Money { amount, currency } => {
            output.extend_from_slice(&amount.to_le_bytes());
            let currency_code = match currency {
                CurrencyType::USD => 1u8,
                CurrencyType::EUR => 2u8,
                CurrencyType::GBP => 3u8,
                CurrencyType::INR => 4u8,
                CurrencyType::Local => 0u8,
            };
            output.push(currency_code);
        },
        IraValue::Rating(r) => {
            output.push(*r);
        },
        IraValue::Year(y) => {
            output.extend_from_slice(&y.to_le_bytes());
        },
        IraValue::Reference { schema, instance } => {
            // Write schema type ID
            let schema_id = match schema {
                SchemaType::Countries => 1u8,
                SchemaType::Teams => 2u8,
                SchemaType::Players => 3u8,
                SchemaType::Leagues => 4u8,
                SchemaType::Matches => 5u8,
                SchemaType::Stadiums => 6u8,
            };
            output.push(schema_id);
            
            // Write instance name as string index
            let index = string_table.iter()
                .position(|s| s == instance)
                .unwrap_or(0) as u32;
            output.extend_from_slice(&index.to_le_bytes());
        },
        IraValue::Array(arr) => {
            // Write array length
            output.extend_from_slice(&(arr.len() as u32).to_le_bytes());
            
            // Write each element
            for element in arr {
                write_value_data(output, element, string_table)?;
            }
        },
        IraValue::Choice(choice) => {
            // Write choice as string index
            let index = string_table.iter()
                .position(|s| s == choice)
                .unwrap_or(0) as u32;
            output.extend_from_slice(&index.to_le_bytes());
        },
        IraValue::TimeZone(tz) => {
            // Write timezone as hours (i8) and minutes (u8)
            output.push(tz.hours as u8);
            output.push(tz.minutes);
        },
        IraValue::UUID(uuid_str) => {
            // Write UUID as string index
            let index = string_table.iter()
                .position(|s| s == uuid_str)
                .unwrap_or(0) as u32;
            output.extend_from_slice(&index.to_le_bytes());
        },
    }
    
    Ok(())
}