//! Ira Language CLI
//! Command-line interface for the Ira language compiler

use clap::{Parser, Subcommand};
use ira_lang::{IraLanguage, runtime::BinaryReader};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ira")]
#[command(about = "Ira Language - Domain-specific language for football simulation data")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile an .ira file to binary format
    Compile {
        /// Input .ira file (must have .ira extension)
        #[arg(value_name = "FILE.ira")]
        input: PathBuf,
        
        /// Output file (defaults to input with .bin extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Validate an .ira file syntax
    Validate {
        /// Input .ira file (must have .ira extension)
        #[arg(value_name = "FILE.ira")]
        input: PathBuf,
        
        /// Verbose validation output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Show information about an .ira file
    Info {
        /// Input .ira file (must have .ira extension)
        #[arg(value_name = "FILE.ira")]
        input: PathBuf,
        
        /// Show detailed statistics
        #[arg(long)]
        stats: bool,
    },
    
    /// Read and display contents of a compiled .bin file
    Read {
        /// Input .bin file (must have .bin extension)
        #[arg(value_name = "FILE.bin")]
        input: PathBuf,
        
        /// Output format (json, yaml, or text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Create a new .ira file from template
    New {
        /// Name of the new file
        #[arg(value_name = "NAME")]
        name: String,
        
        /// Template type (basic, football, country, team)
        #[arg(short, long, default_value = "basic")]
        template: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Validate that input file has .ira extension
fn validate_ira_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(extension) = path.extension() {
        if extension != "ira" {
            return Err(format!(
                "Invalid file extension '{}'. Ira language only accepts '.ira' files.",
                extension.to_string_lossy()
            ).into());
        }
    } else {
        return Err("File must have '.ira' extension.".into());
    }
    
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()).into());
    }
    
    Ok(())
}

/// Validate that input file has .bin extension (for compiled files)
fn validate_bin_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(extension) = path.extension() {
        if extension != "bin" {
            return Err(format!(
                "Invalid file extension '{}'. Expected '.bin' file for compiled Ira data.",
                extension.to_string_lossy()
            ).into());
        }
    } else {
        return Err("File must have '.bin' extension.".into());
    }
    
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()).into());
    }
    
    Ok(())
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Compile { input, output, verbose } => {
            compile_command(input, output, verbose)
        },
        Commands::Validate { input, verbose } => {
            validate_command(input, verbose)
        },
        Commands::Info { input, stats } => {
            info_command(input, stats)
        },
        Commands::Read { input, format } => {
            read_command(input, format)
        },
        Commands::New { name, template } => {
            new_command(name, template)
        },
    }
}

fn compile_command(input: PathBuf, output: Option<PathBuf>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Validate input file extension
    validate_ira_file(&input)?;
    
    if verbose {
        println!("🚀 Compiling {}...", input.display());
    }
    
    // Read source file
    let source = fs::read_to_string(&input)?;
    
    // Initialize Ira language
    let ira = IraLanguage::new();
    
    // Parse and compile with detailed error reporting
    let binary_data = match ira.parse_and_compile(&source) {
        Ok(data) => {
            if verbose {
                println!("✅ Parsing successful");
                println!("✅ Validation passed");
                println!("✅ Binary compilation completed");
            }
            data
        },
        Err(e) => {
            eprintln!("❌ Compilation failed for {}", input.display());
            eprintln!("💡 Error details: {}", e);
            
            // Provide helpful hints based on error type
            let error_str = format!("{}", e);
            if error_str.contains("Parse error") {
                eprintln!("💡 Check your syntax - ensure proper NAMESPACE and SCHEMA structure");
            } else if error_str.contains("Unknown field") {
                eprintln!("💡 Check field names - all fields must be UPPERCASE (CODE, NAME, etc.)");
            } else if error_str.contains("Required field missing") {
                eprintln!("💡 Ensure all required fields are present in your data instances");
            } else if error_str.contains("Unknown schema") {
                eprintln!("💡 Valid schemas are: COUNTRIES, TEAMS, PLAYERS, LEAGUES, MATCHES, STADIUMS");
            }
            
            return Err(e.into());
        }
    };
    
    // Determine output file - always use .bin extension
    let output_path = if let Some(out_path) = output {
        // If user specified output, ensure it has .bin extension
        if let Some(extension) = out_path.extension() {
            if extension != "bin" {
                println!("⚠️  Warning: Output file should have '.bin' extension. Adding '.bin'.");
                let mut corrected_path = out_path.clone();
                corrected_path.set_extension("bin");
                corrected_path
            } else {
                out_path
            }
        } else {
            let mut corrected_path = out_path.clone();
            corrected_path.set_extension("bin");
            corrected_path
        }
    } else {
        // Default: input file name with .bin extension
        let mut path = input.clone();
        path.set_extension("bin");
        path
    };
    
    // Write binary output
    fs::write(&output_path, binary_data)?;
    
    if verbose {
        let input_size = source.len();
        let output_size = fs::metadata(&output_path)?.len();
        let compression_ratio = (input_size as f64) / (output_size as f64);
        
        println!("✅ Compilation successful!");
        println!("📊 Input size:  {} bytes", input_size);
        println!("📊 Output size: {} bytes", output_size);
        println!("🔥 Compression: {:.1}x smaller", compression_ratio);
    } else {
        println!("✅ Compiled to {}", output_path.display());
    }
    
    Ok(())
}

fn validate_command(input: PathBuf, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Validate input file extension
    validate_ira_file(&input)?;
    
    if verbose {
        println!("🔍 Validating {}...", input.display());
    }
    
    // Read source file
    let source = fs::read_to_string(&input)?;
    
    // Initialize Ira language
    let ira = IraLanguage::new();
    
    // Parse (this includes validation) with detailed error reporting
    let ast = match ira.parse(&source) {
        Ok(parsed) => {
            println!("✅ Syntax validation passed!");
            
            if verbose {
                // Count schemas and data for verbose output
                let schema_count = parsed.data_namespace.schema_data.len();
                let mut total_instances = 0;
                let mut total_fields = 0;
                
                for schema_data in parsed.data_namespace.schema_data.values() {
                    total_instances += schema_data.instances.len();
                    for instance in schema_data.instances.values() {
                        total_fields += instance.fields.len();
                    }
                }
                
                println!("📊 Found {} schemas with {} instances and {} total fields", 
                         schema_count, total_instances, total_fields);
                
                if let Some(override_ns) = &parsed.override_namespace {
                    println!("🔧 Found {} schema overrides", override_ns.schema_overrides.len());
                }
            }
            
            parsed
        },
        Err(e) => {
            eprintln!("❌ Validation failed for {}", input.display());
            eprintln!("💡 Error details: {}", e);
            
            // Provide helpful hints
            let error_str = format!("{}", e);
            if error_str.contains("Parse error") {
                eprintln!("💡 Syntax error - check your NAMESPACE and SCHEMA structure");
            } else if error_str.contains("Unknown field") {
                eprintln!("💡 Invalid field name - ensure fields are UPPERCASE (CODE, NAME, etc.)");
            } else if error_str.contains("Required field missing") {
                eprintln!("💡 Missing required fields - check schema requirements");
            }
            
            return Err(e.into());
        }
    };
    
    if verbose {
        println!("📊 File size: {} bytes", source.len());
        println!("📊 Lines: {}", source.lines().count());
        
        // Show schema breakdown
        for (schema_type, schema_data) in &ast.data_namespace.schema_data {
            println!("📋 {:?}: {} instances", schema_type, schema_data.instances.len());
        }
    }
    
    Ok(())
}

fn info_command(input: PathBuf, stats: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Validate input file extension
    validate_ira_file(&input)?;
    
    println!("📄 File: {}", input.display());
    
    // Read source file
    let source = fs::read_to_string(&input)?;
    
    // Initialize Ira language
    let ira = IraLanguage::new();
    
    // Parse file
    let ast = ira.parse(&source)?;
    
    // Basic info
    println!("📏 Size: {} bytes", source.len());
    println!("📊 Lines: {}", source.lines().count());
    
    // Count schemas and instances
    for (schema_type, schema_data) in &ast.data_namespace.schema_data {
        
        println!("📋 {} schema: {} instances", 
                 format!("{:?}", schema_type), 
                 schema_data.instances.len());
    }
    
    if let Some(override_ns) = &ast.override_namespace {
        println!("🔧 Overrides: {} schema overrides", override_ns.schema_overrides.len());
    }
    
    if stats {
        // Compile to get compression estimates
        let binary_data = ira.compile(&ast)?;
        let compression_ratio = (source.len() as f64) / (binary_data.len() as f64);
        
        println!("\n📈 Statistics:");
        println!("  Binary size: {} bytes", binary_data.len());
        println!("  Compression: {:.1}x smaller", compression_ratio);
        println!("  Saved: {} bytes ({:.1}%)", 
                 source.len() - binary_data.len(),
                 (1.0 - (binary_data.len() as f64 / source.len() as f64)) * 100.0);
    }
    
    Ok(())
}

fn read_command(input: PathBuf, format: String) -> Result<(), Box<dyn std::error::Error>> {
    // Validate input file extension
    validate_bin_file(&input)?;
    
    println!("📂 Reading {}...", input.display());
    
    // Read binary file
    let binary_data = fs::read(&input)?;
    
    // Parse binary data
    let mut reader = BinaryReader::new(binary_data)?;
    let header = reader.read_header()?;
    let string_table = reader.read_string_table()?;
    let data_sections = reader.read_data_sections(&string_table)?;
    
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&data_sections)?);
        },
        "text" | _ => {
            println!("📊 Binary File Info:");
            println!("  Version: {}", header.version);
            println!("  Strings: {}", string_table.len());
            println!("  Schemas: {}", data_sections.len());
            
            for (schema_type, schema_data) in data_sections {
                println!("\n📋 {:?} Schema:", schema_type);
                println!("  Instances: {}", schema_data.instances.len());
                
                for (instance_name, instance) in schema_data.instances.iter().take(3) {
                    println!("    {}: {} fields", instance_name, instance.fields.len());
                }
                
                if schema_data.instances.len() > 3 {
                    println!("    ... and {} more", schema_data.instances.len() - 3);
                }
            }
        }
    }
    
    Ok(())
}

fn new_command(name: String, template: String) -> Result<(), Box<dyn std::error::Error>> {
    // Always ensure .ira extension
    let filename = if name.ends_with(".ira") {
        name
    } else if name.contains('.') {
        // User specified different extension - warn and force .ira
        println!("⚠️  Warning: Ira files must have '.ira' extension. Changing to '.ira'.");
        let base_name = name.split('.').next().unwrap_or(&name);
        format!("{}.ira", base_name)
    } else {
        format!("{}.ira", name)
    };
    
    if std::path::Path::new(&filename).exists() {
        return Err(format!("File already exists: {}", filename).into());
    }
    
    let template_content = match template.as_str() {
        "basic" => basic_template(),
        "football" => football_template(),
        "country" => country_template(),
        "team" => team_template(),
        _ => return Err(format!("Unknown template: {}", template).into()),
    };
    
    fs::write(&filename, template_content)?;
    println!("✅ Created {} from {} template", filename, template);
    
    Ok(())
}

fn basic_template() -> String {
    r#"/**
 * Basic Ira File Template
 * Generated by Ira CLI
 */

NAMESPACE DATA {
    SCHEMA COUNTRIES {
        Example_Country: {
            CODE: EXA,
            NAME: "Example Country",
            SHORT_CODE: EX,
            CONTINENT: EUROPE,
            CAPITAL: "Example City",
            POPULATION: 10000000,
            LAND_AREA: 100000,
            TIME_ZONE: 0,
            GDP_PER_CAPITA: 25000,
            CURRENCY_CODE: EUR,
            CURRENCY_SYMBOL: €,
            AVERAGE_WAGE_LEVEL: 30000,
            ECONOMIC_STABILITY: 80,
            PRIMARY_LANGUAGE: "English",
            FOOTBALL_CULTURE: 70,
            FOOTBALL_HISTORY: 100,
            YOUTH_RATING: 60,
            COACHING_LEVEL: 65,
            FACILITIES_RATING: 70,
            FOOTBALL_IMPORTANCE: 75,
            JUNIOR_COACHING: 65,
        }
    }
}
"#.to_string()
}

fn football_template() -> String {
    r#"/**
 * Football Data Template
 * Generated by Ira CLI
 */

NAMESPACE DATA {
    SCHEMA COUNTRIES {
        // Add your country data here
    }
    
    SCHEMA TEAMS {
        // Add your team data here
    }
}
"#.to_string()
}

fn country_template() -> String {
    r#"/**
 * Country Data Template
 * Generated by Ira CLI
 */

NAMESPACE DATA {
    SCHEMA COUNTRIES {
        // Define your countries here
    }
}
"#.to_string()
}

fn team_template() -> String {
    r#"/**
 * Team Data Template
 * Generated by Ira CLI
 */

NAMESPACE DATA {
    SCHEMA TEAMS {
        // Define your teams here
    }
}
"#.to_string()
}