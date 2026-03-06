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
        add_field!("ID", DataType::UUID, required);  // Primary UUID identifier
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
    
    /// Create the Teams schema with comprehensive football team properties
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
        
        // Core Identity (8 properties)
        add_field!("teamId", DataType::UUID, required);
        add_field!("countryId", DataType::UUID, required); // References Countries.ID
        add_field!("leagueId", DataType::UUID, required);  // References Leagues.ID
        add_field!("stadiumId", DataType::UUID, required); // References Stadiums.ID
        add_field!("name", DataType::Text { max_length: None }, required);
        add_field!("shortName", DataType::Text { max_length: Some(4) }, required);
        add_field!("nickname", DataType::Text { max_length: None }, optional);
        add_field!("foundedYear", DataType::Year, required);
        add_field!("city", DataType::Text { max_length: None }, required);
        add_field!("primaryColor", DataType::Text { max_length: Some(20) }, required);
        
        // Financial Management (12 properties)
        add_field!("transferBudget", DataType::Money { currency: None }, required);
        add_field!("wageBudget", DataType::Money { currency: None }, required);
        add_field!("bankBalance", DataType::Money { currency: None }, required);
        add_field!("debt", DataType::Money { currency: None }, optional);
        add_field!("revenue", DataType::Money { currency: None }, required);
        add_field!("ticketPrices", DataType::Rating { min: 1, max: 10 }, required);
        add_field!("commercialDeals", DataType::Money { currency: None }, required);
        add_field!("tvMoney", DataType::Money { currency: None }, required);
        add_field!("luxuryTax", DataType::Money { currency: None }, optional);
        add_field!("bankruptcyRisk", DataType::Rating { min: 0, max: 100 }, required);
        add_field!("sugarDaddy", DataType::Boolean, optional);
        add_field!("profitabilityTarget", DataType::Money { currency: None }, optional);
        
        // Playing Style & Tactics (15 properties)
        add_field!("preferredFormation", DataType::Text { max_length: Some(10) }, required);
        add_field!("playingPhilosophy", DataType::Choice { 
            options: vec!["possession".to_string(), "counter_attack".to_string(), 
                         "direct".to_string(), "tiki_taka".to_string(), "long_ball".to_string()] 
        }, required);
        add_field!("attackingMentality", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("defensiveMentality", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("possessionStyle", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("counterAttackSpeed", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("pressing", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("creativity", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("directness", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("width", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("tempo", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("setPlayerFocus", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("teamCohesion", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("tacticalFamiliarity", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("mentalStrength", DataType::Rating { min: 1, max: 100 }, required);
        
        // Infrastructure & Facilities (5 properties - stadium removed, referenced)
        add_field!("trainingFacilities", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("youthAcademy", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("medicalFacilities", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("scoutingNetwork", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("dataAnalytics", DataType::Rating { min: 1, max: 100 }, required);
        
        // Performance & Reputation (10 properties)
        add_field!("overallRating", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("domesticReputation", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("continentalReputation", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("worldReputation", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("currentForm", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("homeAdvantage", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("awayPerformance", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("bigGameMentality", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("consistency", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("clutchFactor", DataType::Rating { min: 1, max: 100 }, required);
        
        // Squad & Management (7 properties)
        add_field!("squadSize", DataType::Range { min: 16, max: 35 }, required);
        add_field!("averageAge", DataType::Number, required);
        add_field!("foreignPlayerCount", DataType::Number, required);
        add_field!("homegrownPlayers", DataType::Number, required);
        add_field!("injuryProneness", DataType::Rating { min: 0, max: 100 }, required);
        add_field!("squadDepth", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("teamSpirit", DataType::Rating { min: 1, max: 100 }, required);
        
        // Competition & Historical (5 properties)
        add_field!("currentLeagueTier", DataType::Rating { min: 1, max: 10 }, required);
        add_field!("euroCompetitionSpots", DataType::Range { min: 0, max: 5 }, optional);
        add_field!("rivalryIntensity", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("trophyHistory", DataType::Number, optional);
        add_field!("managerExpectations", DataType::Rating { min: 1, max: 100 }, required);
        
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
    
    /// Create the Leagues schema with all 50 simulation properties
    fn create_leagues_schema() -> SchemaDefinition {
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
        
        // Core Identity & References (1-10)
        add_field!("leagueId", DataType::UUID, required);  // LEAG10000-19999
        add_field!("countryId", DataType::UUID, required); // REFERENCES Countries.ID
        add_field!("name", DataType::Text { max_length: None }, required);
        add_field!("shortName", DataType::Text { max_length: Some(10) }, required);
        add_field!("foundedYear", DataType::Year, required);
        add_field!("tier", DataType::Rating { min: 1, max: 10 }, required);
        add_field!("totalTeams", DataType::Range { min: 4, max: 50 }, required);
        add_field!("currentSeason", DataType::Number, required);
        add_field!("status", DataType::Choice { 
            options: vec!["active".to_string(), "suspended".to_string(), "defunct".to_string()] 
        }, required);
        add_field!("leaguePrestige", DataType::Rating { min: 1, max: 100 }, required);
        
        // Financial Simulation (11-20)
        add_field!("salaryCapAmount", DataType::Money { currency: None }, required);
        add_field!("salaryFloorAmount", DataType::Money { currency: None }, required);
        add_field!("revenueSharing", DataType::Rating { min: 0, max: 100 }, required);
        add_field!("tvRevenue", DataType::Money { currency: None }, required);
        add_field!("sponsorshipRevenue", DataType::Money { currency: None }, required);
        add_field!("ticketRevenueBase", DataType::Money { currency: None }, required);
        add_field!("transferTaxRate", DataType::Rating { min: 0, max: 100 }, required);
        add_field!("luxuryTaxThreshold", DataType::Money { currency: None }, required);
        add_field!("bankruptcyProtection", DataType::Boolean, required);
        add_field!("prizeMoneyTotal", DataType::Money { currency: None }, required);
        
        // Competition Structure (21-30)
        add_field!("regularSeasonGames", DataType::Number, required);
        add_field!("playoffTeams", DataType::Number, required);
        add_field!("playoffFormat", DataType::Choice { 
            options: vec!["single".to_string(), "double".to_string(), "bracket".to_string(), "round_robin".to_string()] 
        }, required);
        add_field!("promotionSlots", DataType::Range { min: 0, max: 5 }, required);
        add_field!("relegationSlots", DataType::Range { min: 0, max: 5 }, required);
        add_field!("conferenceCount", DataType::Range { min: 1, max: 8 }, required);
        add_field!("scheduleBalance", DataType::Rating { min: 0, max: 100 }, required);
        add_field!("seasonStartWeek", DataType::Range { min: 1, max: 52 }, required);
        add_field!("seasonLength", DataType::Number, required);
        add_field!("breakWeeks", DataType::Array { 
            element_type: Box::new(DataType::Number), 
            max_size: Some(10) 
        }, optional);
        
        // Gameplay Rules (31-40)
        add_field!("maxForeignPlayers", DataType::Number, required);
        add_field!("minHomegrownPlayers", DataType::Number, required);
        add_field!("substitutionRules", DataType::Number, required);
        add_field!("transferWindowStart", DataType::Range { min: 1, max: 52 }, required);
        add_field!("transferWindowEnd", DataType::Range { min: 1, max: 52 }, required);
        add_field!("contractLengthMax", DataType::Number, required);
        add_field!("ageLimitMin", DataType::Number, required);
        add_field!("ageLimitMax", DataType::Number, optional);
        add_field!("disciplinaryPoints", DataType::Number, required);
        add_field!("matchOfficials", DataType::Choice { 
            options: vec!["assigned".to_string(), "neutral".to_string(), "random".to_string()] 
        }, required);
        
        // Performance Tracking (41-50)
        add_field!("championshipPoints", DataType::Number, required);
        add_field!("tiebreakMethod", DataType::Choice { 
            options: vec!["head_to_head".to_string(), "goal_difference".to_string(), "goals_scored".to_string()] 
        }, required);
        add_field!("playerStatsTracked", DataType::Array { 
            element_type: Box::new(DataType::Text { max_length: None }), 
            max_size: Some(20) 
        }, optional);
        add_field!("teamStatsTracked", DataType::Array { 
            element_type: Box::new(DataType::Text { max_length: None }), 
            max_size: Some(15) 
        }, optional);
        add_field!("attendanceImpact", DataType::Rating { min: 0, max: 100 }, required);
        add_field!("homeAdvantage", DataType::Rating { min: 0, max: 100 }, required);
        add_field!("weatherEffects", DataType::Boolean, required);
        add_field!("injuryRate", DataType::Rating { min: 0, max: 100 }, required);
        add_field!("retirementAge", DataType::Number, required);
        add_field!("youthDevelopment", DataType::Rating { min: 0, max: 100 }, required);
        
        SchemaDefinition {
            name: "LEAGUES".to_string(),
            fields,
            required_fields,
            optional_fields,
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
    
    /// Create the Stadiums schema with comprehensive venue properties
    fn create_stadiums_schema() -> SchemaDefinition {
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
        
        // Core Identity (8 properties)
        add_field!("stadiumId", DataType::UUID, required);
        add_field!("countryId", DataType::UUID, required); // References Countries.ID
        add_field!("name", DataType::Text { max_length: None }, required);
        add_field!("nickname", DataType::Text { max_length: None }, optional);
        add_field!("city", DataType::Text { max_length: None }, required);
        add_field!("country", DataType::Text { max_length: None }, required);
        add_field!("openedYear", DataType::Year, required);
        add_field!("lastRenovation", DataType::Year, optional);
        add_field!("ownership", DataType::Choice { 
            options: vec!["club".to_string(), "council".to_string(), "private".to_string(), "government".to_string()] 
        }, required);
        
        // Capacity & Seating (7 properties)
        add_field!("totalCapacity", DataType::Number, required);
        add_field!("seatedCapacity", DataType::Number, required);
        add_field!("standingCapacity", DataType::Number, optional);
        add_field!("executiveBoxes", DataType::Number, optional);
        add_field!("disabledAccess", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("awaySectionSize", DataType::Number, optional);
        add_field!("capacityUtilization", DataType::Rating { min: 0, max: 100 }, optional);
        
        // Physical Infrastructure (10 properties)
        add_field!("pitchLength", DataType::Number, required);
        add_field!("pitchWidth", DataType::Number, required);
        add_field!("pitchQuality", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("pitchType", DataType::Choice { 
            options: vec!["grass".to_string(), "artificial".to_string(), "hybrid".to_string()] 
        }, required);
        add_field!("undersoilHeating", DataType::Boolean, optional);
        add_field!("roofCoverage", DataType::Rating { min: 0, max: 100 }, optional);
        add_field!("floodlightQuality", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("drainageSystem", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("structuralCondition", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("expansionPotential", DataType::Boolean, optional);
        
        // Atmosphere & Environment (8 properties)
        add_field!("homeAdvantage", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("crowdNoise", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("atmosphereRating", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("acousticDesign", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("supporterCulture", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("rivalryIntensity", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("weatherExposure", DataType::Rating { min: 0, max: 100 }, optional);
        add_field!("intimidationFactor", DataType::Rating { min: 1, max: 100 }, optional);
        
        // Facilities & Amenities (9 properties)
        add_field!("corporateFacilities", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("mediaFacilities", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("conferenceFacilities", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("restaurantQuality", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("parkingSpaces", DataType::Number, optional);
        add_field!("publicTransport", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("securitySystems", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("medicalFacilities", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("playerTunnels", DataType::Rating { min: 1, max: 100 }, optional);
        
        // Commercial & Financial (6 properties)
        add_field!("namingRights", DataType::Money { currency: None }, optional);
        add_field!("annualRunningCosts", DataType::Money { currency: None }, optional);
        add_field!("ticketPriceLevel", DataType::Rating { min: 1, max: 10 }, optional);
        add_field!("concessionRevenue", DataType::Money { currency: None }, optional);
        add_field!("eventHostingCapacity", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("broadcastValue", DataType::Rating { min: 1, max: 100 }, optional);
        
        // Technical & Safety (4 properties)
        add_field!("safetyRating", DataType::Rating { min: 1, max: 100 }, required);
        add_field!("emergencyExits", DataType::Number, optional);
        add_field!("cameraSystem", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("communicationSystems", DataType::Rating { min: 1, max: 100 }, optional);
        
        // Performance Impact (3 properties)
        add_field!("weatherResistance", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("pitchAdvantage", DataType::Rating { min: 1, max: 100 }, optional);
        add_field!("fatigueImpact", DataType::Rating { min: 0, max: 100 }, optional);
        
        SchemaDefinition {
            name: "STADIUMS".to_string(),
            fields,
            required_fields,
            optional_fields,
        }
    }
}

impl Default for BuiltInSchemas {
    fn default() -> Self {
        Self::new()
    }
}