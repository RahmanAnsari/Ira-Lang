//! Grammar definitions and parsing utilities

/// Grammar rules for the Ira language
pub struct Grammar {
    // Grammar rules will be defined here
}

impl Grammar {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Validate that a field name follows Ira naming conventions
    pub fn is_valid_field_name(name: &str) -> bool {
        // Field names must be uppercase with underscores
        name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
            && !name.is_empty()
            && !name.starts_with('_')
            && !name.ends_with('_')
    }
    
    /// Validate that a schema name follows conventions
    pub fn is_valid_schema_name(name: &str) -> bool {
        matches!(name.to_uppercase().as_str(), 
                "COUNTRIES" | "TEAMS" | "PLAYERS" | "LEAGUES" | "MATCHES" | "STADIUMS")
    }
    
    /// Validate that an identifier follows conventions
    pub fn is_valid_identifier(name: &str) -> bool {
        !name.is_empty()
            && name.chars().next().unwrap().is_alphabetic()
            && name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }
}

impl Default for Grammar {
    fn default() -> Self {
        Self::new()
    }
}