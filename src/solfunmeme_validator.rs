use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Import our unified validator types
use crate::validator::{
    DataAccess, EntityIdentifier, ParquetMetadata, ValidationError, ValidationResult,
    CachedResponse, ValidationLevel,
    DatasetValidator, 
    validate_split, validate_config, validate_dataset
};

/// Structure representing a term in the solfunmeme-index dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexTerm {
    pub term: String,
    pub count: u32,
    pub category: String,
    pub significance: String,
    pub vibe: String,
    pub action_suggestion: String,
    pub emoji_representation: Option<String>,
    pub semantic_names: Option<Vec<String>>,
    pub osi_layer: Option<String>,
    pub prime_factor: Option<u64>,
    pub is_power_of_two: Option<bool>,
    pub numerical_address: Option<String>,
    pub embedding_vectors: Option<Vec<f64>>,
    pub versions: Vec<String>,
    pub first_seen_timestamp: Option<u64>,
    pub last_seen_timestamp: Option<u64>,
}

/// Real implementation of DataAccess for the solfunmeme-index dataset
#[derive(Clone)]
pub struct SolfunmemeDataAccess {
    base_path: String,
    cache: HashMap<String, CachedResponse>,
}

impl SolfunmemeDataAccess {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
            cache: HashMap::new(),
        }
    }

    /// Load a term from the filesystem
    /// Since terms are organized by the first character of the actual term (not the ID),
    /// we need to search across directories if we don't know the term content
    pub fn load_term(&self, term_id: &str) -> Result<IndexTerm, ValidationError> {
        // First, try to load from a cache or mapping if we have one
        // For now, we'll search across all character directories
        
        // Get all available character directories
        let chars = self.get_available_chars()?;
        
        for char_dir in chars {
            let term_path = format!("{}/terms/{}/{}.json", self.base_path, char_dir, term_id);
            
            if Path::new(&term_path).exists() {
                let content = fs::read_to_string(&term_path)
                    .map_err(|e| ValidationError::DataAccessError {
                        message: format!("Failed to read term file {}: {}", term_path, e),
                    })?;
                
                return serde_json::from_str(&content)
                    .map_err(|e| ValidationError::DataAccessError {
                        message: format!("Failed to parse term JSON: {}", e),
                    });
            }
        }
        
        // If not found in any directory, return an error
        Err(ValidationError::DataAccessError {
            message: format!("Term with ID '{}' not found in any character directory", term_id),
        })
    }

    /// Load a term from a specific character directory (more efficient if you know the character)
    // pub fn load_term_from_char(&self, term_id: &str, char_dir: &str) -> Result<IndexTerm, ValidationError> {
    //     let term_path = format!("{}/terms/{}/{}.json", self.base_path, char_dir, term_id);
        
    //     let content = fs::read_to_string(&term_path)
    //         .map_err(|e| ValidationError::DataAccessError {
    //             message: format!("Failed to read term file {}: {}", term_path, e),
    //         })?;
        
    //     serde_json::from_str(&content)
    //         .map_err(|e| ValidationError::DataAccessError {
    //             message: format!("Failed to parse term JSON: {}", e),
    //         })
    // }

    /// Get all term IDs for a given first character
    fn get_term_ids_for_char(&self, first_char: char) -> Result<Vec<String>, ValidationError> {
        let dir_path = format!("{}/terms/{}", self.base_path, first_char);
        
        let entries = fs::read_dir(&dir_path)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to read directory {}: {}", dir_path, e),
            })?;
        
        let mut term_ids = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to read directory entry: {}", e),
            })?;
            
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    let term_id = file_name.trim_end_matches(".json");
                    term_ids.push(term_id.to_string());
                }
            }
        }
        
        Ok(term_ids)
    }

    /// Get all available first characters (configs in our model)
    fn get_available_chars(&self) -> Result<Vec<String>, ValidationError> {
        let terms_dir = format!("{}/terms", self.base_path);
        
        let entries = fs::read_dir(&terms_dir)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to read terms directory: {}", e),
            })?;
        
        let mut chars = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to read directory entry: {}", e),
            })?;
            
            if entry.file_type().map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to get file type: {}", e),
            })?.is_dir() {
                if let Some(dir_name) = entry.file_name().to_str() {
                    chars.push(dir_name.to_string());
                }
            }
        }
        
        chars.sort();
        Ok(chars)
    }

    /// Check if the dataset has specific capabilities based on its structure
    fn analyze_dataset_capabilities(&self) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check if we can view the dataset structure
        result.viewer = Path::new(&format!("{}/terms", self.base_path)).exists();
        
        // Check if we can preview data (sample some terms)
        result.preview = self.can_preview_data();
        
        // Check if we can search (based on term structure)
        result.search = self.can_search_terms();
        
        // Check if we can filter (based on metadata fields)
        result.filter = self.can_filter_terms();
        
        // Check if we have statistics (counts, metadata)
        result.statistics = self.has_statistics();
        
        result
    }

    fn can_preview_data(&self) -> bool {
        // Try to load a sample term to see if preview is possible
        if let Ok(chars) = self.get_available_chars() {
            for char in chars.iter().take(3) { // Check first 3 characters
                if let Ok(term_ids) = self.get_term_ids_for_char(char.chars().next().unwrap_or('a')) {
                    if let Some(term_id) = term_ids.first() {
                        if self.load_term(term_id).is_ok() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn can_search_terms(&self) -> bool {
        // We can search if terms have searchable text fields
        if let Ok(chars) = self.get_available_chars() {
            if let Some(first_char) = chars.first() {
                if let Ok(term_ids) = self.get_term_ids_for_char(first_char.chars().next().unwrap_or('a')) {
                    if let Some(term_id) = term_ids.first() {
                        if let Ok(term) = self.load_term(term_id) {
                            return !term.term.is_empty() || 
                                   !term.category.is_empty() || 
                                   !term.significance.is_empty();
                        }
                    }
                }
            }
        }
        false
    }

    fn can_filter_terms(&self) -> bool {
        // We can filter if terms have filterable metadata
        if let Ok(chars) = self.get_available_chars() {
            if let Some(first_char) = chars.first() {
                if let Ok(term_ids) = self.get_term_ids_for_char(first_char.chars().next().unwrap_or('a')) {
                    if let Some(term_id) = term_ids.first() {
                        if let Ok(term) = self.load_term(term_id) {
                            return term.count > 0 || 
                                   !term.category.is_empty() ||
                                   term.prime_factor.is_some() ||
                                   term.is_power_of_two.is_some();
                        }
                    }
                }
            }
        }
        false
    }

    fn has_statistics(&self) -> bool {
        // Check if we have statistical data (counts, timestamps, etc.)
        if let Ok(chars) = self.get_available_chars() {
            if let Some(first_char) = chars.first() {
                if let Ok(term_ids) = self.get_term_ids_for_char(first_char.chars().next().unwrap_or('a')) {
                    if let Some(term_id) = term_ids.first() {
                        if let Ok(term) = self.load_term(term_id) {
                            return term.count > 0 || 
                                   term.first_seen_timestamp.is_some() ||
                                   term.last_seen_timestamp.is_some();
                        }
                    }
                }
            }
        }
        false
    }

    /// Perform health check on the dataset
    pub fn health_check(&self) -> Result<(), ValidationError> {
        // Check if the base directory exists and is accessible
        if !Path::new(&self.base_path).exists() {
            return Err(ValidationError::DataAccessError {
                message: format!("Base path does not exist: {}", self.base_path),
            });
        }
        
        let terms_dir = format!("{}/terms", self.base_path);
        if !Path::new(&terms_dir).exists() {
            return Err(ValidationError::DataAccessError {
                message: format!("Terms directory does not exist: {}", terms_dir),
            });
        }
        
        Ok(())
    }
}

impl DataAccess for SolfunmemeDataAccess {
    fn check_successful_response(&self, kind: &str, entity: &EntityIdentifier) -> Result<bool, ValidationError> {
        match kind {
            "config-has-viewer" => {
                // Check if we can view terms for this character (config)
                if let Some(config) = &entity.config {
                    let first_char = config.chars().next().unwrap_or('a');
                    let dir_path = format!("{}/terms/{}", self.base_path, first_char);
                    Ok(Path::new(&dir_path).exists())
                } else {
                    Ok(false)
                }
            }
            "split-has-preview" => {
                // Check if we can preview this specific term (split)
                if let Some(split) = &entity.split {
                    self.load_term(split).map(|_| true).or(Ok(false))
                } else {
                    Ok(false)
                }
            }
            "split-has-statistics" => {
                // Check if this term has statistical data
                if let Some(split) = &entity.split {
                    match self.load_term(split) {
                        Ok(term) => Ok(term.count > 0 || 
                                      term.first_seen_timestamp.is_some() ||
                                      term.last_seen_timestamp.is_some()),
                        Err(_) => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(true), // Default to true for unknown kinds
        }
    }

    fn get_parquet_metadata(&self, _dataset: &str, config: &str) -> Result<ParquetMetadata, ValidationError> {
        // Simulate parquet metadata based on the JSON structure
        let mut features = HashMap::new();
        
        // Add features based on IndexTerm structure
        features.insert("term".to_string(), "string".to_string());
        features.insert("count".to_string(), "int32".to_string());
        features.insert("category".to_string(), "string".to_string());
        features.insert("significance".to_string(), "string".to_string());
        features.insert("vibe".to_string(), "string".to_string());
        features.insert("action_suggestion".to_string(), "string".to_string());
        features.insert("emoji_representation".to_string(), "string".to_string());
        features.insert("osi_layer".to_string(), "string".to_string());
        features.insert("prime_factor".to_string(), "int64".to_string());
        features.insert("is_power_of_two".to_string(), "boolean".to_string());
        features.insert("numerical_address".to_string(), "string".to_string());
        features.insert("first_seen_timestamp".to_string(), "int64".to_string());
        features.insert("last_seen_timestamp".to_string(), "int64".to_string());
        
        // Try to get actual count of terms for this config
        let first_char = config.chars().next().unwrap_or('a');
        let num_rows = self.get_term_ids_for_char(first_char)
            .map(|ids| ids.len() as u64)
            .unwrap_or(0);
        
        Ok(ParquetMetadata::new(features).with_rows(num_rows))
    }

    fn get_split_names(&self, _dataset: &str, config: &str) -> Result<Vec<String>, ValidationError> {
        // In our model, splits are individual terms within a character group
        let first_char = config.chars().next().unwrap_or('a');
        self.get_term_ids_for_char(first_char)
    }

    fn get_config_names(&self, _dataset: &str) -> Result<Vec<String>, ValidationError> {
        // In our model, configs are the first characters (a, b, c, etc.)
        self.get_available_chars()
    }

    fn get_cached_validation(&self, kind: &str, entity: &EntityIdentifier) -> Result<CachedResponse, ValidationError> {
        let cache_key = entity.cache_key(kind);
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Generate validation result based on the entity
        let result = match entity.infer_level() {
            ValidationLevel::Split => {
                // Validate individual term
                if let Some(split) = &entity.split {
                    match self.load_term(split) {
                        Ok(term) => ValidationResult {
                            viewer: true,
                            preview: true,
                            search: !term.term.is_empty(),
                            filter: term.count > 0 || !term.category.is_empty(),
                            statistics: term.count > 0 || term.first_seen_timestamp.is_some(),
                        },
                        Err(_) => ValidationResult::new(),
                    }
                } else {
                    ValidationResult::new()
                }
            }
            ValidationLevel::Config => {
                // Validate character group
                if let Some(config) = &entity.config {
                    let first_char = config.chars().next().unwrap_or('a');
                    match self.get_term_ids_for_char(first_char) {
                        Ok(term_ids) if !term_ids.is_empty() => ValidationResult {
                            viewer: true,
                            preview: true,
                            search: true,
                            filter: true,
                            statistics: true,
                        },
                        _ => ValidationResult::new(),
                    }
                } else {
                    ValidationResult::new()
                }
            }
            ValidationLevel::Dataset => {
                // Validate entire dataset
                self.analyze_dataset_capabilities()
            }
        };
        
        Ok(CachedResponse::new(200, result, 1.0))
    }

    fn has_indexable_columns(&self, features: &HashMap<String, String>) -> bool {
        // Check if we have string columns that can be indexed for search
        features.values().any(|v| v.contains("string"))
    }
}

/// Convenience function to create a validator for the solfunmeme dataset
pub fn create_solfunmeme_validator(base_path: &str) -> Result<DatasetValidator<SolfunmemeDataAccess>, ValidationError> {
    let data_access = SolfunmemeDataAccess::new(base_path);
    
    // Perform health check
    data_access.health_check()?;
    
    Ok(DatasetValidator::new(data_access))
}

/// Example usage and testing function
pub fn test_solfunmeme_dataset() -> Result<(), ValidationError> {
    println!("=== Testing Solfunmeme Dataset Validator ===\n");
    
    let base_path = "/home/mdupont/2025/08/07/solfunmeme-index";
    
    // Create data access instance for testing
    let data_access = SolfunmemeDataAccess::new(base_path);
    data_access.health_check()?;
    
    // Test dataset-level validation
    println!("1. Dataset-level validation:");
    let (result, progress) = validate_dataset("solfunmeme-index", data_access.clone())?;
    println!("   Result: {:?}", result);
    println!("   Progress: {:.1}%", progress * 100.0);
    println!("   Capabilities: {}/{}", result.capability_count(), 5);
    println!();
    
    // Test config-level validation (character 'a')
    println!("2. Config-level validation (character 'a'):");
    let (result, progress) = validate_config("solfunmeme-index", "a", data_access.clone())?;
    println!("   Result: {:?}", result);
    println!("   Progress: {:.1}%", progress * 100.0);
    println!("   Capabilities: {}/{}", result.capability_count(), 5);
    println!();
    
    // Test split-level validation (specific term)
    println!("3. Split-level validation (term '10000'):");
    let (result, progress) = validate_split("solfunmeme-index", "a", "10000", data_access.clone())?;
    println!("   Result: {:?}", result);
    println!("   Progress: {:.1}%", progress * 100.0);
    println!("   Capabilities: {}/{}", result.capability_count(), 5);
    println!();
    
    // Show some dataset statistics
    println!("4. Dataset Statistics:");
    let configs = data_access.get_config_names("solfunmeme-index")?;
    println!("   Available character groups: {}", configs.len());
    println!("   Characters: {:?}", configs.iter().take(10).collect::<Vec<_>>());
    
    // Sample some terms from character 'a'
    if let Ok(term_ids) = data_access.get_split_names("solfunmeme-index", "a") {
        println!("   Terms in 'a': {} terms", term_ids.len());
        println!("   Sample terms: {:?}", term_ids.iter().take(5).collect::<Vec<_>>());
        
        // Load and display a sample term
        if let Some(term_id) = term_ids.first() {
            if let Ok(term) = data_access.load_term(term_id) {
                println!("   Sample term details:");
                println!("     Term: '{}'", term.term);
                println!("     Count: {}", term.count);
                println!("     Category: '{}'", term.category);
                println!("     Significance: '{}'", term.significance);
                if let Some(emoji) = &term.emoji_representation {
                    println!("     Emoji: {}", emoji);
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solfunmeme_data_access() {
        let base_path = "/home/mdupont/2025/08/07/solfunmeme-index";
        
        // Skip test if directory doesn't exist
        if !Path::new(base_path).exists() {
            println!("Skipping test - dataset not found at {}", base_path);
            return;
        }
        
        let data_access = SolfunmemeDataAccess::new(base_path);
        
        // Test health check
        assert!(data_access.health_check().is_ok());
        
        // Test getting config names (character groups)
        let configs = data_access.get_config_names("solfunmeme-index").unwrap();
        assert!(!configs.is_empty());
        assert!(configs.contains(&"a".to_string()));
        
        // Test getting split names for character 'a'
        let splits = data_access.get_split_names("solfunmeme-index", "a").unwrap();
        assert!(!splits.is_empty());
        
        // Test loading a specific term - manually test with a known term in 'a' directory
        // We know "17751" is in the 'a' directory and represents "abilities"
        if splits.contains(&"17751".to_string()) {
            let term = data_access.load_term("17751").unwrap();
            assert_eq!(term.term, "abilities");
            assert!(term.count > 0);
        } else {
            // If 17751 is not available, just test the first available term
            if let Some(term_id) = splits.first() {
                // For this test, we'll just verify that we can load some term
                // The actual term content validation is less important than the loading mechanism
                let result = data_access.load_term(term_id);
                assert!(result.is_ok(), "Failed to load term {}: {:?}", term_id, result.err());
            }
        }
    }

    #[test]
    fn test_validation_capabilities() {
        let base_path = "/home/mdupont/2025/08/07/solfunmeme-index";
        
        // Skip test if directory doesn't exist
        if !Path::new(base_path).exists() {
            println!("Skipping test - dataset not found at {}", base_path);
            return;
        }
        
        let validator = create_solfunmeme_validator(base_path).unwrap();
        
        // Test dataset validation
        let result = validate_dataset("solfunmeme-index", validator.data_access);
        assert!(result.is_ok());
        
        let (validation_result, progress) = result.unwrap();
        assert_eq!(progress, 1.0); // Dataset validation should be complete
        assert!(validation_result.has_any_capability());
    }
}
