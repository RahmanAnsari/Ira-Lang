//! Ira Reader (iread)
//! Query tool for reading and analyzing compiled .irac files

use ira_lang::runtime::BinaryReader;
use ira_lang::types::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

const VERSION: &str = "1.0.0";

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Handle version flag
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("iread (Ira Query Tool) {}", VERSION);
        return;
    }
    
    // Handle help flag
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return;
    }
    
    // Check for input file and query
    if args.len() < 3 {
        eprintln!("❌ Error: Missing input file or query");
        eprintln!("💡 Usage: iread <file.iracc> <query>");
        eprintln!("💡 Example: iread countries.iracc schema.list");
        eprintln!("💡 Run 'iread --help' for more information");
        process::exit(1);
    }
    
    let input_file = PathBuf::from(&args[1]);
    let query = args[2].clone();
    
    // Run query
    if let Err(e) = execute_query(input_file, query) {
        eprintln!("❌ Query failed: {}", e);
        process::exit(1);
    }
}

fn print_help() {
    println!("iread - Ira Data Query Tool");
    println!();
    println!("USAGE:");
    println!("    iread <input.iracc> <query>");
    println!();
    println!("QUERY EXAMPLES:");
    println!();
    println!("Schema Exploration:");
    println!("    iread countries.iracc schema.list                    # List schemas");
    println!("    iread countries.iracc schema.countries.count         # Count instances");
    println!("    iread countries.iracc schema.countries.list          # List country names");
    println!("    iread countries.iracc schema.countries.fields        # List field names");
    println!();
    println!("Data Access:");
    println!("    iread countries.iracc countries.India                # All fields for India");
    println!("    iread countries.iracc countries.India.YOUTH_RATING   # Specific field");
    println!("    iread countries.iracc countries.all.YOUTH_RATING     # Field for all countries");
    println!();
    println!("Filtering:");
    println!("    iread countries.iracc countries.filter.YOUTH_RATING.gt.80.list");
    println!("    iread countries.iracc countries.filter.CONTINENT.eq.ASIA.count");
    println!("    iread countries.iracc countries.filter.YOUTH_RATING.range.60,80.list");
    println!();
    println!("Sorting:");
    println!("    iread countries.iracc countries.sort.YOUTH_RATING.desc.list");
    println!("    iread countries.iracc countries.sort.YOUTH_RATING.desc.values");
    println!("    iread countries.iracc countries.sort.YOUTH_RATING.desc.top.5");
    println!();
    println!("Statistics:");
    println!("    iread countries.iracc countries.all.YOUTH_RATING.min");
    println!("    iread countries.iracc countries.all.YOUTH_RATING.max");
    println!("    iread countries.iracc countries.all.YOUTH_RATING.avg");
    println!("    iread countries.iracc countries.all.YOUTH_RATING.stats");
    println!();
    println!("The iread tool provides powerful query capabilities for analyzing");
    println!("football simulation data stored in compressed .iracc format.");
}

fn execute_query(input_file: PathBuf, query: String) -> Result<(), Box<dyn std::error::Error>> {
    // Validate input file
    validate_input_file(&input_file)?;
    
    // Load and parse binary data
    let data_sections = load_irac_file(&input_file)?;
    
    // Parse and execute query
    execute_parsed_query(&data_sections, &query)?;
    
    Ok(())
}

fn validate_input_file(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !input.exists() {
        return Err(format!("Input file not found: {}", input.display()).into());
    }
    
    match input.extension() {
        Some(ext) if ext == "irac" || ext == "iracc" => Ok(()),
        Some(ext) => Err(format!(
            "Invalid file extension '.{}'. Expected '.irac' or '.iracc' file.", 
            ext.to_string_lossy()
        ).into()),
        None => Err("Input file must have '.irac' or '.iracc' extension.".into()),
    }
}

fn load_irac_file(input_file: &PathBuf) -> Result<HashMap<SchemaType, SchemaData>, Box<dyn std::error::Error>> {
    let binary_data = fs::read(input_file)?;
    let mut reader = BinaryReader::new(binary_data)?;
    
    let _header = reader.read_header()?;
    let string_table = reader.read_string_table()?;
    let data_sections = reader.read_data_sections(&string_table)?;
    
    Ok(data_sections)
}

fn execute_parsed_query(data_sections: &HashMap<SchemaType, SchemaData>, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = query.split('.').collect();
    
    if parts.is_empty() {
        return Err("Empty query".into());
    }
    
    match parts[0] {
        "schema" => execute_schema_query(data_sections, &parts[1..])?,
        "countries" => execute_countries_query(data_sections, &parts[1..])?,
        "teams" => execute_teams_query(data_sections, &parts[1..])?,
        _ => return Err(format!("Unknown query type: {}", parts[0]).into()),
    }
    
    Ok(())
}

fn execute_schema_query(data_sections: &HashMap<SchemaType, SchemaData>, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    if parts.is_empty() {
        return Err("Incomplete schema query".into());
    }
    
    match parts[0] {
        "list" => {
            // List all available schemas
            for schema_type in data_sections.keys() {
                println!("{:?}", schema_type);
            }
        },
        schema_name => {
            let schema_type = parse_schema_name(schema_name)?;
            
            if let Some(schema_data) = data_sections.get(&schema_type) {
                if parts.len() < 2 {
                    return Err("Incomplete schema query".into());
                }
                
                match parts[1] {
                    "count" => {
                        println!("{}", schema_data.instances.len());
                    },
                    "list" => {
                        for instance_name in schema_data.instances.keys() {
                            println!("{}", instance_name);
                        }
                    },
                    "fields" => {
                        // Get field names from first instance
                        if let Some((_, first_instance)) = schema_data.instances.iter().next() {
                            for field_name in first_instance.fields.keys() {
                                println!("{}", field_name);
                            }
                        }
                    },
                    _ => return Err(format!("Unknown schema operation: {}", parts[1]).into()),
                }
            } else {
                return Err(format!("Schema not found: {}", schema_name).into());
            }
        }
    }
    
    Ok(())
}

fn execute_countries_query(data_sections: &HashMap<SchemaType, SchemaData>, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let countries_data = data_sections.get(&SchemaType::Countries)
        .ok_or("Countries schema not found")?;
    
    if parts.is_empty() {
        return Err("Incomplete countries query".into());
    }
    
    match parts[0] {
        "all" => execute_all_countries_query(countries_data, &parts[1..])?,
        "filter" => execute_filter_query(countries_data, &parts[1..])?,
        "sort" => execute_sort_query(countries_data, &parts[1..])?,
        country_name => {
            if let Some(country_data) = countries_data.instances.get(country_name) {
                if parts.len() == 1 {
                    // Print all fields for this country
                    for (field_name, field_value) in &country_data.fields {
                        println!("{}: {}", field_name, format_value(field_value));
                    }
                } else {
                    // Print specific field(s)
                    let field_names: Vec<&str> = parts[1].split(',').collect();
                    for field_name in &field_names {
                        if let Some(value) = country_data.fields.get(*field_name) {
                            if field_names.len() == 1 {
                                println!("{}", format_value(value));
                            } else {
                                println!("{}: {}", field_name, format_value(value));
                            }
                        } else {
                            return Err(format!("Field not found: {}", field_name).into());
                        }
                    }
                }
            } else {
                return Err(format!("Country not found: {}", country_name).into());
            }
        }
    }
    
    Ok(())
}

fn execute_teams_query(_data_sections: &HashMap<SchemaType, SchemaData>, _parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    // Teams queries - similar structure to countries
    println!("Teams queries not yet implemented");
    Ok(())
}

fn execute_all_countries_query(countries_data: &SchemaData, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    if parts.is_empty() {
        return Err("Incomplete all countries query".into());
    }
    
    let field_name = parts[0];
    
    // Check if this is a statistics query
    if parts.len() > 1 {
        match parts[1] {
            "min" => {
                if let Some((country, value)) = find_min_value(countries_data, field_name)? {
                    println!("{}: {}", country, format_value(value));
                }
                return Ok(());
            },
            "max" => {
                if let Some((country, value)) = find_max_value(countries_data, field_name)? {
                    println!("{}: {}", country, format_value(value));
                }
                return Ok(());
            },
            "avg" => {
                if let Some(avg) = calculate_average(countries_data, field_name)? {
                    println!("{:.1}", avg);
                }
                return Ok(());
            },
            "stats" => {
                print_field_statistics(countries_data, field_name)?;
                return Ok(());
            },
            "unique" => {
                let unique_values = get_unique_values(countries_data, field_name)?;
                for value in unique_values {
                    println!("{}", value);
                }
                return Ok(());
            },
            _ => {}
        }
    }
    
    // Print field values for all countries
    for (country_name, country_data) in &countries_data.instances {
        if let Some(value) = country_data.fields.get(field_name) {
            println!("{}: {}", country_name, format_value(value));
        }
    }
    
    Ok(())
}

fn execute_filter_query(countries_data: &SchemaData, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    if parts.len() < 4 {
        return Err("Incomplete filter query".into());
    }
    
    let field_name = parts[0];
    let operator = parts[1];
    let value_str = parts[2];
    let operation = parts[3];
    
    let filtered_countries = filter_countries(countries_data, field_name, operator, value_str)?;
    
    match operation {
        "list" => {
            for country_name in &filtered_countries {
                println!("{}", country_name);
            }
        },
        "count" => {
            println!("{}", filtered_countries.len());
        },
        "values" => {
            for country_name in &filtered_countries {
                if let Some(country_data) = countries_data.instances.get(country_name) {
                    if let Some(value) = country_data.fields.get(field_name) {
                        println!("{}: {}", country_name, format_value(value));
                    }
                }
            }
        },
        _ => return Err(format!("Unknown filter operation: {}", operation).into()),
    }
    
    Ok(())
}

fn execute_sort_query(countries_data: &SchemaData, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    if parts.len() < 3 {
        return Err("Incomplete sort query".into());
    }
    
    let field_name = parts[0];
    let sort_order = parts[1]; // "asc" or "desc"
    let operation = parts[2]; // "list", "values", "top"
    
    let sorted_countries = sort_countries(countries_data, field_name, sort_order == "desc")?;
    
    let countries_to_show = if parts.len() > 3 && parts[2] == "top" {
        let limit = parts[3].parse::<usize>().unwrap_or(10);
        sorted_countries.into_iter().take(limit).collect()
    } else {
        sorted_countries
    };
    
    match operation {
        "list" | "top" => {
            for country_name in &countries_to_show {
                println!("{}", country_name);
            }
        },
        "values" => {
            for country_name in &countries_to_show {
                if let Some(country_data) = countries_data.instances.get(country_name) {
                    if let Some(value) = country_data.fields.get(field_name) {
                        println!("{}: {}", country_name, format_value(value));
                    }
                }
            }
        },
        _ => return Err(format!("Unknown sort operation: {}", operation).into()),
    }
    
    Ok(())
}

fn parse_schema_name(name: &str) -> Result<SchemaType, Box<dyn std::error::Error>> {
    match name.to_lowercase().as_str() {
        "countries" => Ok(SchemaType::Countries),
        "teams" => Ok(SchemaType::Teams),
        "players" => Ok(SchemaType::Players),
        "leagues" => Ok(SchemaType::Leagues),
        "matches" => Ok(SchemaType::Matches),
        "stadiums" => Ok(SchemaType::Stadiums),
        _ => Err(format!("Unknown schema: {}", name).into()),
    }
}

fn format_value(value: &IraValue) -> String {
    match value {
        IraValue::Text(s) => s.clone(),
        IraValue::Number(n) => n.to_string(),
        IraValue::Integer(i) => i.to_string(),
        IraValue::Boolean(b) => b.to_string(),
        IraValue::Rating(r) => r.to_string(),
        IraValue::Year(y) => y.to_string(),
        IraValue::Money { amount, .. } => amount.to_string(),
        IraValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        },
        IraValue::Choice(c) => c.clone(),
        IraValue::Reference { instance, .. } => instance.clone(),
        IraValue::TimeZone(tz) => tz.format(),
    }
}

fn filter_countries(
    countries_data: &SchemaData, 
    field_name: &str, 
    operator: &str, 
    value_str: &str
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut filtered = Vec::new();
    
    for (country_name, country_data) in &countries_data.instances {
        if let Some(field_value) = country_data.fields.get(field_name) {
            let matches = match operator {
                "gt" => compare_values(field_value, value_str, |a, b| a > b)?,
                "lt" => compare_values(field_value, value_str, |a, b| a < b)?,
                "eq" => compare_values_exact(field_value, value_str)?,
                "gte" => compare_values(field_value, value_str, |a, b| a >= b)?,
                "lte" => compare_values(field_value, value_str, |a, b| a <= b)?,
                "range" => {
                    let range_parts: Vec<&str> = value_str.split(',').collect();
                    if range_parts.len() != 2 {
                        return Err("Range filter requires two values separated by comma".into());
                    }
                    let min_val = range_parts[0].parse::<f64>()?;
                    let max_val = range_parts[1].parse::<f64>()?;
                    check_in_range(field_value, min_val, max_val)?
                },
                _ => return Err(format!("Unknown operator: {}", operator).into()),
            };
            
            if matches {
                filtered.push(country_name.clone());
            }
        }
    }
    
    Ok(filtered)
}

fn compare_values<F>(field_value: &IraValue, value_str: &str, op: F) -> Result<bool, Box<dyn std::error::Error>>
where
    F: Fn(f64, f64) -> bool,
{
    let field_num = extract_numeric_value(field_value)?;
    let compare_num = value_str.parse::<f64>()?;
    Ok(op(field_num, compare_num))
}

fn compare_values_exact(field_value: &IraValue, value_str: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match field_value {
        IraValue::Text(s) | IraValue::Choice(s) => Ok(s == value_str),
        IraValue::Number(n) => Ok(*n == value_str.parse::<f64>()?),
        IraValue::Integer(i) => Ok(*i == value_str.parse::<i64>()?),
        IraValue::Rating(r) => Ok(*r == value_str.parse::<u8>()?),
        IraValue::Boolean(b) => Ok(*b == value_str.parse::<bool>()?),
        _ => Ok(false),
    }
}

fn check_in_range(field_value: &IraValue, min_val: f64, max_val: f64) -> Result<bool, Box<dyn std::error::Error>> {
    let field_num = extract_numeric_value(field_value)?;
    Ok(field_num >= min_val && field_num <= max_val)
}

fn extract_numeric_value(value: &IraValue) -> Result<f64, Box<dyn std::error::Error>> {
    match value {
        IraValue::Number(n) => Ok(*n),
        IraValue::Integer(i) => Ok(*i as f64),
        IraValue::Rating(r) => Ok(*r as f64),
        IraValue::Year(y) => Ok(*y as f64),
        IraValue::Money { amount, .. } => Ok(*amount),
        _ => Err("Field is not numeric".into()),
    }
}

fn sort_countries(
    countries_data: &SchemaData, 
    field_name: &str, 
    descending: bool
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut country_values: Vec<(String, f64)> = Vec::new();
    
    for (country_name, country_data) in &countries_data.instances {
        if let Some(field_value) = country_data.fields.get(field_name) {
            if let Ok(numeric_value) = extract_numeric_value(field_value) {
                country_values.push((country_name.clone(), numeric_value));
            }
        }
    }
    
    country_values.sort_by(|a, b| {
        if descending {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        }
    });
    
    Ok(country_values.into_iter().map(|(name, _)| name).collect())
}

fn find_min_value<'a>(countries_data: &'a SchemaData, field_name: &str) -> Result<Option<(String, &'a IraValue)>, Box<dyn std::error::Error>> {
    let mut min_country = None;
    let mut min_value = f64::INFINITY;
    
    for (country_name, country_data) in &countries_data.instances {
        if let Some(field_value) = country_data.fields.get(field_name) {
            if let Ok(numeric_value) = extract_numeric_value(field_value) {
                if numeric_value < min_value {
                    min_value = numeric_value;
                    min_country = Some((country_name.clone(), field_value));
                }
            }
        }
    }
    
    Ok(min_country)
}

fn find_max_value<'a>(countries_data: &'a SchemaData, field_name: &str) -> Result<Option<(String, &'a IraValue)>, Box<dyn std::error::Error>> {
    let mut max_country = None;
    let mut max_value = f64::NEG_INFINITY;
    
    for (country_name, country_data) in &countries_data.instances {
        if let Some(field_value) = country_data.fields.get(field_name) {
            if let Ok(numeric_value) = extract_numeric_value(field_value) {
                if numeric_value > max_value {
                    max_value = numeric_value;
                    max_country = Some((country_name.clone(), field_value));
                }
            }
        }
    }
    
    Ok(max_country)
}

fn calculate_average(countries_data: &SchemaData, field_name: &str) -> Result<Option<f64>, Box<dyn std::error::Error>> {
    let mut sum = 0.0;
    let mut count = 0;
    
    for (_, country_data) in &countries_data.instances {
        if let Some(field_value) = country_data.fields.get(field_name) {
            if let Ok(numeric_value) = extract_numeric_value(field_value) {
                sum += numeric_value;
                count += 1;
            }
        }
    }
    
    if count > 0 {
        Ok(Some(sum / count as f64))
    } else {
        Ok(None)
    }
}

fn print_field_statistics(countries_data: &SchemaData, field_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut values = Vec::new();
    
    for (_, country_data) in &countries_data.instances {
        if let Some(field_value) = country_data.fields.get(field_name) {
            if let Ok(numeric_value) = extract_numeric_value(field_value) {
                values.push(numeric_value);
            }
        }
    }
    
    if values.is_empty() {
        println!("No numeric values found for field: {}", field_name);
        return Ok(());
    }
    
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    let count = values.len();
    let min = values[0];
    let max = values[count - 1];
    let sum: f64 = values.iter().sum();
    let avg = sum / count as f64;
    let median = if count % 2 == 0 {
        (values[count / 2 - 1] + values[count / 2]) / 2.0
    } else {
        values[count / 2]
    };
    
    println!("Count: {}, Min: {:.1}, Max: {:.1}, Avg: {:.1}, Median: {:.1}", 
             count, min, max, avg, median);
    
    Ok(())
}

fn get_unique_values(countries_data: &SchemaData, field_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut unique_values = std::collections::HashSet::new();
    
    for (_, country_data) in &countries_data.instances {
        if let Some(field_value) = country_data.fields.get(field_name) {
            unique_values.insert(format_value(field_value));
        }
    }
    
    let mut result: Vec<String> = unique_values.into_iter().collect();
    result.sort();
    Ok(result)
}