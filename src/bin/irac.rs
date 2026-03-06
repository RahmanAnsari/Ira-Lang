//! Ira Compiler (irac)
//! Simple compiler for Ira language files - validates and compiles .ira to .bin

use ira_lang::IraLanguage;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

const VERSION: &str = "1.0.0";

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Handle version flag
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("irac (Ira Compiler) {}", VERSION);
        return;
    }
    
    // Handle help flag
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return;
    }
    
    // Check for input file
    if args.len() < 2 {
        eprintln!("❌ Error: No input file specified");
        eprintln!("💡 Usage: irac <file.ira>");
        eprintln!("💡 Run 'irac --help' for more information");
        process::exit(1);
    }
    
    let input_file = PathBuf::from(&args[1]);
    
    // Parse optional output file
    let output_file = if args.len() > 2 {
        Some(PathBuf::from(&args[2]))
    } else {
        None
    };
    
    // Run compiler
    if let Err(e) = compile_file(input_file, output_file) {
        eprintln!("❌ Compilation failed: {}", e);
        process::exit(1);
    }
}

fn print_help() {
    println!("irac - Ira Language Compiler");
    println!();
    println!("USAGE:");
    println!("    irac <input.ira> [output.iracc]");
    println!();
    println!("ARGUMENTS:");
    println!("    <input.ira>     Source file to compile (must have .ira extension)");
    println!("    [output.iracc]  Optional output file (defaults to input with .iracc extension)");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help      Show this help message");
    println!("    -v, --version   Show version information");
    println!();
    println!("EXAMPLES:");
    println!("    irac countries.ira                # Compiles to countries.iracc");
    println!("    irac teams.ira data/teams.iracc   # Compiles to data/teams.iracc");
    println!();
    println!("The Ira compiler validates syntax, checks schema conformance, and");
    println!("generates optimized .iracc binary output with 90%+ compression via brotli.");
}

fn compile_file(input: PathBuf, output: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Validate input file
    validate_input_file(&input)?;
    
    // Step 2: Read source file
    println!("📖 Reading {}...", input.display());
    let source = fs::read_to_string(&input).map_err(|e| {
        format!("Failed to read input file '{}': {}", input.display(), e)
    })?;
    
    if source.trim().is_empty() {
        return Err("Input file is empty".into());
    }
    
    // Step 3: Initialize compiler
    let ira = IraLanguage::new();
    
    // Step 4: Parse and validate
    println!("🔍 Validating syntax and schema...");
    let ast = match ira.parse(&source) {
        Ok(parsed) => {
            println!("✅ Syntax validation passed");
            
            // Show what we found
            let schema_count = parsed.data_namespace.schema_data.len();
            if schema_count == 0 {
                return Err("No data schemas found in file. Expected at least one SCHEMA in DATA namespace.".into());
            }
            
            let mut total_instances = 0;
            for schema_data in parsed.data_namespace.schema_data.values() {
                total_instances += schema_data.instances.len();
            }
            
            if total_instances == 0 {
                return Err("No data instances found. Each schema must have at least one data entry.".into());
            }
            
            println!("✅ Found {} schema(s) with {} data instance(s)", schema_count, total_instances);
            
            // Show override info if present
            if let Some(override_ns) = &parsed.override_namespace {
                println!("🔧 Applied {} schema override(s)", override_ns.schema_overrides.len());
            }
            
            parsed
        },
        Err(e) => {
            println!("❌ Validation failed");
            show_detailed_error(&e, &source, &input)?;
            return Err(e.into());
        }
    };
    
    // Step 5: Compile to compressed binary
    println!("⚙️  Compiling to .iracc format...");
    let binary_data = match ira.compile(&ast) {
        Ok(data) => {
            println!("✅ Binary compilation with brotli compression successful");
            data
        },
        Err(e) => {
            println!("❌ Binary compilation failed");
            eprintln!("💡 Error: {}", e);
            return Err(e.into());
        }
    };
    
    // Step 6: Determine output file
    let output_path = determine_output_path(&input, output)?;
    
    // Step 7: Write output
    println!("💾 Writing to {}...", output_path.display());
    fs::write(&output_path, &binary_data).map_err(|e| {
        format!("Failed to write output file '{}': {}", output_path.display(), e)
    })?;
    
    // Step 8: Show success summary
    show_compilation_summary(&source, &binary_data, &input, &output_path);
    
    Ok(())
}

fn validate_input_file(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Check if file exists
    if !input.exists() {
        return Err(format!("Input file not found: {}", input.display()).into());
    }
    
    // Check file extension
    match input.extension() {
        Some(ext) if ext == "ira" => Ok(()),
        Some(ext) => Err(format!(
            "Invalid file extension '.{}'. Ira compiler only accepts '.ira' files.", 
            ext.to_string_lossy()
        ).into()),
        None => Err("Input file must have '.ira' extension.".into()),
    }
}

fn determine_output_path(input: &PathBuf, output: Option<PathBuf>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(out_path) = output {
        // User specified output file
        if let Some(parent) = out_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create output directory '{}': {}", parent.display(), e)
                })?;
            }
        }
        
        // Ensure .iracc extension
        if let Some(ext) = out_path.extension() {
            if ext != "iracc" {
                println!("⚠️  Warning: Output file should have '.iracc' extension for compressed compiled Ira data");
            }
        }
        
        Ok(out_path)
    } else {
        // Default: input filename with .iracc extension (compressed binary)
        let mut path = input.clone();
        path.set_extension("iracc");
        Ok(path)
    }
}

fn show_detailed_error(
    error: &ira_lang::IraError, 
    _source: &str, 
    file_path: &PathBuf
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!();
    eprintln!("🐛 Detailed Error Report:");
    eprintln!("   File: {}", file_path.display());
    eprintln!("   Error: {}", error);
    eprintln!();
    
    let error_str = format!("{}", error);
    
    // Parse error hints
    if error_str.contains("Parse error") || error_str.contains("Unexpected token") {
        eprintln!("💡 Syntax Error Hints:");
        eprintln!("   • Check your NAMESPACE and SCHEMA structure");
        eprintln!("   • Ensure proper use of braces {{ }} and commas");
        eprintln!("   • Verify all strings are quoted: \"value\"");
        eprintln!("   • Check for typos in keywords (NAMESPACE, DATA, SCHEMA)");
    }
    else if error_str.contains("Unknown field") {
        eprintln!("💡 Field Error Hints:");
        eprintln!("   • All field names must be UPPERCASE (CODE, NAME, etc.)");
        eprintln!("   • Check spelling of field names");
        eprintln!("   • Ensure field exists in the schema definition");
    }
    else if error_str.contains("Required field missing") {
        eprintln!("💡 Missing Field Hints:");
        eprintln!("   • Check that all required fields are present");
        eprintln!("   • Common required fields: CODE, NAME for countries");
        eprintln!("   • Review schema documentation for field requirements");
    }
    else if error_str.contains("Invalid value") || error_str.contains("type mismatch") {
        eprintln!("💡 Value Error Hints:");
        eprintln!("   • Check data types: numbers should be unquoted, strings quoted");
        eprintln!("   • rating() values must be within valid ranges");
        eprintln!("   • choice() values must be from predefined lists");
        eprintln!("   • Arrays use square brackets: [\"value1\", \"value2\"]");
    }
    else if error_str.contains("Unknown schema") {
        eprintln!("💡 Schema Error Hints:");
        eprintln!("   • Valid schemas: COUNTRIES, TEAMS, PLAYERS, LEAGUES, MATCHES, STADIUMS");
        eprintln!("   • Check spelling and capitalization of schema names");
    }
    
    eprintln!();
    eprintln!("📚 For complete syntax help, visit: https://docs.ira-lang.dev");
    eprintln!();
    
    Ok(())
}

fn show_compilation_summary(
    source: &str, 
    binary_data: &[u8], 
    input_path: &PathBuf, 
    output_path: &PathBuf
) {
    let input_size = source.len();
    let output_size = binary_data.len();
    let compression_ratio = if output_size > 0 {
        (input_size as f64) / (output_size as f64)
    } else {
        1.0
    };
    let bytes_saved = if input_size > output_size {
        input_size - output_size
    } else {
        0
    };
    let percent_saved = if input_size > 0 {
        (bytes_saved as f64 / input_size as f64) * 100.0
    } else {
        0.0
    };
    
    println!();
    println!("🎉 Compilation Successful!");
    println!();
    println!("📊 Compression Summary:");
    println!("   Input:  {} → {} bytes", input_path.display(), format_bytes(input_size));
    println!("   Output: {} → {} bytes", output_path.display(), format_bytes(output_size));
    println!("   Saved:  {} bytes ({:.1}%)", format_bytes(bytes_saved), percent_saved);
    println!("   Ratio:  {:.1}x smaller", compression_ratio);
    println!();
    
    if compression_ratio > 4.0 {
        println!("🔥 Excellent compression achieved!");
    } else if compression_ratio > 2.0 {
        println!("✨ Good compression ratio!");
    }
}

fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}