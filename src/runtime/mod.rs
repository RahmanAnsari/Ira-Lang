//! Runtime module for reading and working with compiled Ira binary data

use crate::{error::*, types::*};
use brotli::Decompressor;
use std::io::Read;

/// Binary data reader for Ira files
pub struct BinaryReader {
    data: Vec<u8>,
    position: usize,
}

/// Header information from binary file
#[derive(Debug, Clone)]
pub struct BinaryHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub string_table_size: u32,
    pub data_sections_offset: u32,
}

impl BinaryReader {
    /// Create a new reader from binary data (with automatic brotli decompression)
    pub fn new(data: Vec<u8>) -> Result<Self> {
        // Try to decompress as brotli first
        let decompressed_data = if Self::is_brotli_compressed(&data) {
            let mut decompressor = Decompressor::new(&data[..], 4096);
            let mut decompressed = Vec::new();
            decompressor.read_to_end(&mut decompressed)
                .map_err(|e| IraError::compilation_error(&format!("Brotli decompression failed: {}", e)))?;
            decompressed
        } else {
            data
        };
        
        Ok(Self { data: decompressed_data, position: 0 })
    }
    
    /// Check if data is brotli compressed by looking for magic bytes
    fn is_brotli_compressed(data: &[u8]) -> bool {
        // Brotli doesn't have fixed magic bytes, but we can try to decompress and see if it works
        // For now, we'll assume compressed files start differently than "IRAB"
        data.len() > 4 && &data[0..4] != b"IRAB"
    }
    
    /// Read header from binary data
    pub fn read_header(&mut self) -> Result<BinaryHeader> {
        if self.data.len() < 14 {
            return Err(IraError::compilation_error("Invalid binary file: too short"));
        }
        
        let magic = [
            self.data[0],
            self.data[1], 
            self.data[2],
            self.data[3],
        ];
        
        if &magic != b"IRAB" {
            return Err(IraError::compilation_error("Invalid binary file: wrong magic number"));
        }
        
        let version = u16::from_le_bytes([self.data[4], self.data[5]]);
        let string_table_size = u32::from_le_bytes([
            self.data[6], self.data[7], self.data[8], self.data[9]
        ]);
        let data_sections_offset = u32::from_le_bytes([
            self.data[10], self.data[11], self.data[12], self.data[13]
        ]);
        
        self.position = 14;
        
        Ok(BinaryHeader {
            magic,
            version,
            string_table_size,
            data_sections_offset,
        })
    }
    
    /// Read string table from binary data
    pub fn read_string_table(&mut self) -> Result<Vec<String>> {
        if self.position + 4 > self.data.len() {
            return Err(IraError::compilation_error("Unexpected end of file reading string table"));
        }
        
        let string_count = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        
        let mut strings = Vec::new();
        
        for _ in 0..string_count {
            if self.position + 4 > self.data.len() {
                return Err(IraError::compilation_error("Unexpected end of file reading string"));
            }
            
            let string_length = u32::from_le_bytes([
                self.data[self.position],
                self.data[self.position + 1],
                self.data[self.position + 2],
                self.data[self.position + 3],
            ]);
            self.position += 4;
            
            if self.position + string_length as usize > self.data.len() {
                return Err(IraError::compilation_error("Unexpected end of file reading string data"));
            }
            
            let string_bytes = &self.data[self.position..self.position + string_length as usize];
            let string = String::from_utf8(string_bytes.to_vec())
                .map_err(|_| IraError::compilation_error("Invalid UTF-8 in string table"))?;
            
            strings.push(string);
            self.position += string_length as usize;
        }
        
        Ok(strings)
    }
    
    /// Read all data sections
    pub fn read_data_sections(&mut self, string_table: &[String]) -> Result<std::collections::HashMap<SchemaType, SchemaData>> {
        let mut sections = std::collections::HashMap::new();
        
        while self.position < self.data.len() {
            let (schema_type, schema_data) = self.read_data_section(string_table)?;
            sections.insert(schema_type, schema_data);
        }
        
        Ok(sections)
    }
    
    /// Read a single data section
    fn read_data_section(&mut self, string_table: &[String]) -> Result<(SchemaType, SchemaData)> {
        if self.position + 8 > self.data.len() {
            return Err(IraError::compilation_error("Unexpected end of file reading data section"));
        }
        
        let schema_id = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        
        let schema_type = match schema_id {
            1 => SchemaType::Countries,
            2 => SchemaType::Teams,
            3 => SchemaType::Players,
            4 => SchemaType::Leagues,
            5 => SchemaType::Matches,
            6 => SchemaType::Stadiums,
            _ => return Err(IraError::compilation_error(&format!("Unknown schema ID: {}", schema_id))),
        };
        
        let instance_count = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        
        let mut instances = std::collections::HashMap::new();
        
        for i in 0..instance_count {
            let (name, instance) = self.read_instance_data(string_table, i)?;
            instances.insert(name, instance);
        }
        
        Ok((schema_type, SchemaData { instances }))
    }
    
    /// Read instance data
    fn read_instance_data(&mut self, string_table: &[String], _index: u32) -> Result<(String, DataInstance)> {
        if self.position + 8 > self.data.len() {
            return Err(IraError::compilation_error("Unexpected end of file reading instance"));
        }
        
        // Read instance name from string table
        let name_index = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        
        let instance_name = string_table.get(name_index as usize)
            .ok_or_else(|| IraError::compilation_error("Invalid string table index for instance name"))?
            .clone();
        
        let field_count = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        
        let mut fields = std::collections::HashMap::new();
        
        for _ in 0..field_count {
            let (field_name, value) = self.read_field_data(string_table)?;
            fields.insert(field_name, value);
        }
        
        Ok((instance_name, DataInstance { fields }))
    }
    
    /// Read field data
    fn read_field_data(&mut self, string_table: &[String]) -> Result<(String, IraValue)> {
        if self.position + 5 > self.data.len() {
            return Err(IraError::compilation_error("Unexpected end of file reading field"));
        }
        
        let name_index = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        
        let field_name = string_table.get(name_index as usize)
            .ok_or_else(|| IraError::compilation_error("Invalid string table index"))?
            .clone();
        
        let value_type = self.data[self.position];
        self.position += 1;
        
        let value = match value_type {
            1 => { // Text
                let index = self.read_u32()?;
                let text = string_table.get(index as usize)
                    .ok_or_else(|| IraError::compilation_error("Invalid string table index"))?
                    .clone();
                IraValue::Text(text)
            },
            2 => { // Number
                let num = self.read_f64()?;
                IraValue::Number(num)
            },
            3 => { // Integer
                let int = self.read_i64()?;
                IraValue::Integer(int)
            },
            4 => { // Boolean
                let bool_val = self.data[self.position] != 0;
                self.position += 1;
                IraValue::Boolean(bool_val)
            },
            5 => { // Money
                let amount = self.read_f64()?;
                let currency_code = self.data[self.position];
                self.position += 1;
                let currency = match currency_code {
                    1 => CurrencyType::USD,
                    2 => CurrencyType::EUR,
                    3 => CurrencyType::GBP,
                    4 => CurrencyType::INR,
                    _ => CurrencyType::Local,
                };
                IraValue::Money { amount, currency }
            },
            6 => { // Rating
                let rating = self.data[self.position];
                self.position += 1;
                IraValue::Rating(rating)
            },
            7 => { // Year
                let year = self.read_u16()?;
                IraValue::Year(year)
            },
            8 => { // Reference
                let schema_id = self.data[self.position];
                self.position += 1;
                let schema = match schema_id {
                    1 => SchemaType::Countries,
                    2 => SchemaType::Teams,
                    3 => SchemaType::Players,
                    4 => SchemaType::Leagues,
                    5 => SchemaType::Matches,
                    6 => SchemaType::Stadiums,
                    _ => SchemaType::Countries, // fallback
                };
                let instance_index = self.read_u32()?;
                let instance = string_table.get(instance_index as usize)
                    .ok_or_else(|| IraError::compilation_error("Invalid string table index"))?
                    .clone();
                IraValue::Reference { schema, instance }
            },
            9 => { // Array
                let array_length = self.read_u32()?;
                let mut elements = Vec::new();
                for _ in 0..array_length {
                    let element = self.read_value_only(string_table)?;
                    elements.push(element);
                }
                IraValue::Array(elements)
            },
            10 => { // Choice
                let choice_index = self.read_u32()?;
                let choice = string_table.get(choice_index as usize)
                    .ok_or_else(|| IraError::compilation_error("Invalid string table index"))?
                    .clone();
                IraValue::Choice(choice)
            },
            11 => { // TimeZone
                let hours = self.data[self.position] as i8;
                self.position += 1;
                let minutes = self.data[self.position];
                self.position += 1;
                let tz = TimeZone::new(hours, minutes)
                    .map_err(|e| IraError::compilation_error(&e))?;
                IraValue::TimeZone(tz)
            },
            12 => { // UUID
                let uuid_index = self.read_u32()?;
                let uuid_str = string_table.get(uuid_index as usize)
                    .ok_or_else(|| IraError::compilation_error("Invalid string table index"))?
                    .clone();
                IraValue::UUID(uuid_str)
            },
            _ => {
                return Err(IraError::compilation_error(&format!("Unknown value type: {}", value_type)));
            }
        };
        
        Ok((field_name, value))
    }
    
    /// Read u32 from current position
    fn read_u32(&mut self) -> Result<u32> {
        if self.position + 4 > self.data.len() {
            return Err(IraError::compilation_error("Unexpected end of file"));
        }
        
        let value = u32::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
        ]);
        self.position += 4;
        
        Ok(value)
    }
    
    /// Read u16 from current position
    fn read_u16(&mut self) -> Result<u16> {
        if self.position + 2 > self.data.len() {
            return Err(IraError::compilation_error("Unexpected end of file"));
        }
        
        let value = u16::from_le_bytes([
            self.data[self.position],
            self.data[self.position + 1],
        ]);
        self.position += 2;
        
        Ok(value)
    }
    
    /// Read f64 from current position
    fn read_f64(&mut self) -> Result<f64> {
        if self.position + 8 > self.data.len() {
            return Err(IraError::compilation_error("Unexpected end of file"));
        }
        
        let bytes = [
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
            self.data[self.position + 4],
            self.data[self.position + 5],
            self.data[self.position + 6],
            self.data[self.position + 7],
        ];
        self.position += 8;
        
        Ok(f64::from_le_bytes(bytes))
    }
    
    /// Read i64 from current position
    fn read_i64(&mut self) -> Result<i64> {
        if self.position + 8 > self.data.len() {
            return Err(IraError::compilation_error("Unexpected end of file"));
        }
        
        let bytes = [
            self.data[self.position],
            self.data[self.position + 1],
            self.data[self.position + 2],
            self.data[self.position + 3],
            self.data[self.position + 4],
            self.data[self.position + 5],
            self.data[self.position + 6],
            self.data[self.position + 7],
        ];
        self.position += 8;
        
        Ok(i64::from_le_bytes(bytes))
    }
    
    /// Read just a value (without field name) for array elements
    fn read_value_only(&mut self, string_table: &[String]) -> Result<IraValue> {
        if self.position + 1 > self.data.len() {
            return Err(IraError::compilation_error("Unexpected end of file reading value"));
        }
        
        let value_type = self.data[self.position];
        self.position += 1;
        
        let value = match value_type {
            1 => { // Text
                let index = self.read_u32()?;
                let text = string_table.get(index as usize)
                    .ok_or_else(|| IraError::compilation_error("Invalid string table index"))?
                    .clone();
                IraValue::Text(text)
            },
            2 => { // Number
                let num = self.read_f64()?;
                IraValue::Number(num)
            },
            3 => { // Integer
                let int = self.read_i64()?;
                IraValue::Integer(int)
            },
            4 => { // Boolean
                let bool_val = self.data[self.position] != 0;
                self.position += 1;
                IraValue::Boolean(bool_val)
            },
            5 => { // Money
                let amount = self.read_f64()?;
                let currency_code = self.data[self.position];
                self.position += 1;
                let currency = match currency_code {
                    1 => CurrencyType::USD,
                    2 => CurrencyType::EUR,
                    3 => CurrencyType::GBP,
                    4 => CurrencyType::INR,
                    _ => CurrencyType::Local,
                };
                IraValue::Money { amount, currency }
            },
            6 => { // Rating
                let rating = self.data[self.position];
                self.position += 1;
                IraValue::Rating(rating)
            },
            7 => { // Year
                let year = self.read_u16()?;
                IraValue::Year(year)
            },
            8 => { // Reference
                let schema_id = self.data[self.position];
                self.position += 1;
                let schema = match schema_id {
                    1 => SchemaType::Countries,
                    2 => SchemaType::Teams,
                    3 => SchemaType::Players,
                    4 => SchemaType::Leagues,
                    5 => SchemaType::Matches,
                    6 => SchemaType::Stadiums,
                    _ => SchemaType::Countries, // fallback
                };
                let instance_index = self.read_u32()?;
                let instance = string_table.get(instance_index as usize)
                    .ok_or_else(|| IraError::compilation_error("Invalid string table index"))?
                    .clone();
                IraValue::Reference { schema, instance }
            },
            10 => { // Choice  
                let choice_index = self.read_u32()?;
                let choice = string_table.get(choice_index as usize)
                    .ok_or_else(|| IraError::compilation_error("Invalid string table index"))?
                    .clone();
                IraValue::Choice(choice)
            },
            11 => { // TimeZone
                let hours = self.data[self.position] as i8;
                self.position += 1;
                let minutes = self.data[self.position];
                self.position += 1;
                let tz = TimeZone::new(hours, minutes)
                    .map_err(|e| IraError::compilation_error(&e))?;
                IraValue::TimeZone(tz)
            },
            12 => { // UUID
                let uuid_index = self.read_u32()?;
                let uuid_str = string_table.get(uuid_index as usize)
                    .ok_or_else(|| IraError::compilation_error("Invalid string table index"))?
                    .clone();
                IraValue::UUID(uuid_str)
            },
            _ => {
                return Err(IraError::compilation_error(&format!("Unknown value type: {}", value_type)));
            }
        };
        
        Ok(value)
    }
}