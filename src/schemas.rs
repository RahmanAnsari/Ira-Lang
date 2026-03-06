//! Built-in schema definitions for football entities

use crate::types::*;
use std::collections::HashMap;

/// Container for all built-in schemas
#[derive(Debug, Clone)]
pub struct BuiltInSchemas {
    pub countries: SchemaDefinition,
    pub teams: SchemaDefinition,
    pub players: SchemaDefinition,
    pub leagues: SchemaDefinition,
    pub matches: SchemaDefinition,
    pub stadiums: SchemaDefinition,
}

/// Schema definition with field specifications
#[derive(Debug, Clone)]
pub struct SchemaDefinition {
    pub name: String,
    pub fields: HashMap<String, FieldDefinition>,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
}

/// Individual field definition
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    pub name: String,
    pub data_type: DataType,
    pub required: bool,
    pub description: Option<String>,
    pub default_value: Option<IraValue>,
}

impl BuiltInSchemas {
    /// Create new instance with all built-in schemas
    pub fn new() -> Self {
        Self {
            countries: Self::create_countries_schema(),
            teams: Self::create_teams_schema(),
            players: Self::create_players_schema(),
            leagues: Self::create_leagues_schema(),
            matches: Self::create_matches_schema(),
            stadiums: Self::create_stadiums_schema(),
        }
    }
    
    /// Get schema by type
    pub fn get_schema(&self, schema_type: &SchemaType) -> &SchemaDefinition {
        match schema_type {
            SchemaType::Countries => &self.countries,
            SchemaType::Teams => &self.teams,
            SchemaType::Players => &self.players,
            SchemaType::Leagues => &self.leagues,
            SchemaType::Matches => &self.matches,
            SchemaType::Stadiums => &self.stadiums,
        }
    }
    
    /// Create the Countries schema with all FM properties
    fn create_countries_schema() -> SchemaDefinition {
        let mut fields = HashMap::new();
        let mut required_fields = Vec::new();
        let mut optional_fields = Vec::new();
        
        // Helper macro to add fields
        macro_rules! add_field {
            ($name:literal, $type:expr, required) => {
                fields.insert($name.to_string(), FieldDefinition {
                    name: $name.to_string(),
                    data_type: $type,
                    required: true,
                    description: None,
                    default_value: None,
                });
                required_fields.push($name.to_string());
            };
            ($name:literal, $type:expr, optional) => {
                fields.insert($name.to_string(), FieldDefinition {
                    name: $name.to_string(),
                    data_type: $type,
                    required: false,
                    description: None,
                    default_value: None,
                });
                optional_fields.push($name.to_string());
            };
        }
        
        // Basic identification (required)
        add_field!("CODE", DataType::Text { max_length: Some(3) }, required);
        add_field!("NAME", DataType::Text { max_length: None }, required);
        add_field!("SHORT_CODE", DataType::Text { max_length: Some(2) }, required);
        
        // Geographic & demographic (required)
        add_field!("CONTINENT", DataType::Choice { 
            options: vec!["EUROPE".to_string(), "SOUTH_AMERICA".to_string(), 
                         "NORTH_AMERICA".to_string(), "AFRICA".to_string(), 
                         "ASIA".to_string(), "OCEANIA".to_string()] 
        }, required);
        add_field!("CAPITAL", DataType::Text { max_length: None }, required);
        add_field!("POPULATION", DataType::Number, required);
        add_field!("LAND_AREA", DataType::Number, required);
        add_field!("TIME_ZONE", DataType::TimeZone, required);
        
        // Economic (required)
        add_field!("GDP_PER_CAPITA", DataType::Money { currency: None }, required);
        add_field!("CURRENCY_CODE", DataType::Text { max_length: Some(3) }, required);
        add_field!("CURRENCY_SYMBOL", DataType::Text { max_length: Some(5) }, required);
        add_field!("AVERAGE_WAGE_LEVEL", DataType::Money { currency: None }, required);
        add_field!("ECONOMIC_STABILITY", DataType::Rating { min: 1, max: 100 }, required);
        
        // Language & culture
        add_field!("PRIMARY_LANGUAGE", DataType::Text { max_length: None }, required);
        add_field!("SECONDARY_LANGUAGES", DataType::Array { 
            element_type: Box::new(DataType::Text { max_length: None }), 
            max_size: Some(3) 
        }, optional);
        add_field!("FOOTBALL_CULTURE", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("FOOTBALL_HISTORY", DataType::Number, required);
        
        // Football development (required)
        add_field!("YOUTH_RATING", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("COACHING_LEVEL", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("FACILITIES_RATING", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("FOOTBALL_IMPORTANCE", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("JUNIOR_COACHING", DataType::Rating { min: 1, max: 100 }, required);
        
        // Coaching systems
        add_field!("COACHING_EDUCATION", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("LICENSED_COACHES", DataType::Number, optional);
        add_field!("COACHING_SCHOOLS", DataType::Number, optional);
        add_field!("TACTICAL_KNOWLEDGE", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("MODERN_METHODS", DataType::Rating { min: 1, max: 100 }, required);
        
        // Infrastructure
        add_field!("STADIUMS_OVER_10K", DataType::Number, optional);
        add_field!("STADIUMS_OVER_30K", DataType::Number, optional);
        add_field!("TRAINING_CENTERS", DataType::Number, required);
        add_field!("MEDICAL_SUPPORT", DataType::Rating { min: 1, max: 100 }, required);
        
        // Economic factors for clubs
        add_field!("AVERAGE_TRANSFER_BUDGET", DataType::Money { currency: None }, required);
        add_field!("AVERAGE_WAGE_BUDGET", DataType::Money { currency: None }, required);
        add_field!("SPONSORSHIP_LEVELS", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("TV_MONEY_DISTRIBUTION", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("TICKET_PRICE_LEVELS", DataType::Rating { min: 1, max: 100 }, required);
        
        // Talent production characteristics
        add_field!("TECHNICAL_TENDENCY", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("PHYSICAL_TENDENCY", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("MENTAL_TENDENCY", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("PACE_TENDENCY", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("CREATIVITY_TENDENCY", DataType::Rating { min: 1, max: 100 }, required);
        
        // Playing style tendencies
        add_field!("POSSESSION_STYLE", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("COUNTER_ATTACK_STYLE", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("DIRECT_STYLE", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("TECHNICAL_STYLE", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("PHYSICAL_STYLE", DataType::Rating { min: 1, max: 100 }, required);
        
        // Reputation
        add_field!("DOMESTIC_REPUTATION", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("CONTINENTAL_REPUTATION", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("WORLD_REPUTATION", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("CONTINENTAL_CONFEDERATION", DataType::Choice { 
            options: vec!["UEFA".to_string(), "CONMEBOL".to_string(), 
                         "CONCACAF".to_string(), "CAF".to_string(), 
                         "AFC".to_string(), "OFC".to_string()] 
        }, required);
        
        // Special characteristics (optional)
        add_field!("IS_FOOTBALL_MAJOR", DataType::Boolean, optional);
        add_field!("HAS_STRONG_DOMESTIC_LEAGUE", DataType::Boolean, optional);
        add_field!("IS_EMERGING_MARKET", DataType::Boolean, optional);
        add_field!("HAS_FOOTBALL_TRADITION", DataType::Boolean, optional);
        add_field!("IS_WEALTHY_NATION", DataType::Boolean, optional);
        
        SchemaDefinition {
            name: "COUNTRIES".to_string(),
            fields,
            required_fields,
            optional_fields,
        }
    }
    
    /// Create the Teams schema
    fn create_teams_schema() -> SchemaDefinition {
        let mut fields = HashMap::new();
        let mut required_fields = Vec::new();
        let mut optional_fields = Vec::new();
        
        macro_rules! add_field {
            ($name:literal, $type:expr, required) => {
                fields.insert($name.to_string(), FieldDefinition {
                    name: $name.to_string(),
                    data_type: $type,
                    required: true,
                    description: None,
                    default_value: None,
                });
                required_fields.push($name.to_string());
            };
            ($name:literal, $type:expr, optional) => {
                fields.insert($name.to_string(), FieldDefinition {
                    name: $name.to_string(),
                    data_type: $type,
                    required: false,
                    description: None,
                    default_value: None,
                });
                optional_fields.push($name.to_string());
            };
        }
        
        // Basic identification
        add_field!("CODE", DataType::Text { max_length: Some(3) }, required);
        add_field!("NAME", DataType::Text { max_length: None }, required);
        add_field!("SHORT_NAME", DataType::Text { max_length: Some(3) }, required);
        add_field!("NICKNAME", DataType::Text { max_length: None }, required);
        
        // Location
        add_field!("CITY", DataType::Text { max_length: None }, required);
        add_field!("COUNTRY", DataType::Reference { schema: SchemaType::Countries }, required);
        add_field!("FOUNDED", DataType::Year, required);
        
        // Reputation
        add_field!("NATIONAL_REPUTATION", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("CONTINENTAL_REPUTATION", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("WORLD_REPUTATION", DataType::Rating { min: 1, max: 100 }, required);
        
        // Finances
        add_field!("TRANSFER_BUDGET", DataType::Money { currency: None }, required);
        add_field!("WAGE_BUDGET", DataType::Money { currency: None }, required);
        add_field!("DEBT", DataType::Money { currency: None }, optional);
        add_field!("ANNUAL_REVENUE", DataType::Money { currency: None }, required);
        add_field!("CLUB_VALUE", DataType::Money { currency: None }, required);
        
        // Facilities
        add_field!("TRAINING_FACILITIES", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("YOUTH_FACILITIES", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("MEDICAL_FACILITIES", DataType::Rating { min: 1, max: 100 }, required);
        
        // Stadium
        add_field!("STADIUM_NAME", DataType::Text { max_length: None }, required);
        add_field!("STADIUM_CAPACITY", DataType::Number, required);
        add_field!("STADIUM_CONDITION", DataType::Rating { min: 1, max: 100 }, required);
        
        // Board & fans
        add_field!("BOARD_CONFIDENCE", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("FAN_LOYALTY", DataType::Rating { min: 1, max: 100 }, required);
        
        SchemaDefinition {
            name: "TEAMS".to_string(),
            fields,
            required_fields,
            optional_fields,
        }
    }
    
    // Placeholder implementations for other schemas
    fn create_players_schema() -> SchemaDefinition {
        SchemaDefinition {
            name: "PLAYERS".to_string(),
            fields: HashMap::new(),
            required_fields: Vec::new(),
            optional_fields: Vec::new(),
        }
    }
    
    fn create_leagues_schema() -> SchemaDefinition {
        SchemaDefinition {
            name: "LEAGUES".to_string(),
            fields: HashMap::new(),
            required_fields: Vec::new(),
            optional_fields: Vec::new(),
        }
    }
    
    fn create_matches_schema() -> SchemaDefinition {
        SchemaDefinition {
            name: "MATCHES".to_string(),
            fields: HashMap::new(),
            required_fields: Vec::new(),
            optional_fields: Vec::new(),
        }
    }
    
    fn create_stadiums_schema() -> SchemaDefinition {
        SchemaDefinition {
            name: "STADIUMS".to_string(),
            fields: HashMap::new(),
            required_fields: Vec::new(),
            optional_fields: Vec::new(),
        }
    }
}

impl Default for BuiltInSchemas {
    fn default() -> Self {
        Self::new()
    }
}