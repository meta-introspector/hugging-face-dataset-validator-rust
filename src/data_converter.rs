use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;

use crate::solfunmeme_validator::SolfunmemeDataAccess;
use crate::validator::{DataAccess, ValidationError};

/// Data converter for exporting solfunmeme dataset to different formats
pub struct DataConverter {
    data_access: SolfunmemeDataAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermRecord {
    pub id: String,
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
    pub first_seen_timestamp: Option<u64>,
    pub last_seen_timestamp: Option<u64>,
    pub character_group: String,
}

impl DataConverter {
    pub fn new(base_path: &str) -> Result<Self, ValidationError> {
        let data_access = SolfunmemeDataAccess::new(base_path);
        data_access.health_check()?;
        Ok(Self { data_access })
    }

    /// Export all terms from a character group to JSON Lines format
    pub fn export_character_to_jsonl(&self, character: &str, output_path: &str) -> Result<usize, ValidationError> {
        let term_ids = self.data_access.get_split_names("solfunmeme-index", character)?;
        
        let mut file = File::create(output_path)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to create output file {}: {}", output_path, e),
            })?;

        let mut exported_count = 0;
        for term_id in term_ids {
            match self.data_access.load_term(&term_id) {
                Ok(term) => {
                    let record = TermRecord {
                        id: term_id.clone(),
                        term: term.term,
                        count: term.count,
                        category: term.category,
                        significance: term.significance,
                        vibe: term.vibe,
                        action_suggestion: term.action_suggestion,
                        emoji_representation: term.emoji_representation,
                        semantic_names: term.semantic_names,
                        osi_layer: term.osi_layer,
                        prime_factor: term.prime_factor,
                        is_power_of_two: term.is_power_of_two,
                        numerical_address: term.numerical_address,
                        first_seen_timestamp: term.first_seen_timestamp,
                        last_seen_timestamp: term.last_seen_timestamp,
                        character_group: character.to_string(),
                    };

                    let json_line = serde_json::to_string(&record)
                        .map_err(|e| ValidationError::DataAccessError {
                            message: format!("Failed to serialize term {}: {}", term_id, e),
                        })?;
                    
                    writeln!(file, "{}", json_line)
                        .map_err(|e| ValidationError::DataAccessError {
                            message: format!("Failed to write to output file: {}", e),
                        })?;
                    
                    exported_count += 1;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load term {}: {}", term_id, e);
                }
            }
        }

        Ok(exported_count)
    }

    /// Export all terms to a single JSON Lines file
    pub fn export_all_to_jsonl(&self, output_path: &str) -> Result<usize, ValidationError> {
        let characters = self.data_access.get_config_names("solfunmeme-index")?;
        
        let mut file = File::create(output_path)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to create output file {}: {}", output_path, e),
            })?;

        let mut total_exported = 0;
        
        for character in characters {
            println!("Exporting character group: {}", character);
            
            let term_ids = match self.data_access.get_split_names("solfunmeme-index", &character) {
                Ok(ids) => ids,
                Err(e) => {
                    eprintln!("Warning: Failed to get terms for character {}: {}", character, e);
                    continue;
                }
            };

            for term_id in term_ids {
                match self.data_access.load_term(&term_id) {
                    Ok(term) => {
                        let record = TermRecord {
                            id: term_id.clone(),
                            term: term.term,
                            count: term.count,
                            category: term.category,
                            significance: term.significance,
                            vibe: term.vibe,
                            action_suggestion: term.action_suggestion,
                            emoji_representation: term.emoji_representation,
                            semantic_names: term.semantic_names,
                            osi_layer: term.osi_layer,
                            prime_factor: term.prime_factor,
                            is_power_of_two: term.is_power_of_two,
                            numerical_address: term.numerical_address,
                            first_seen_timestamp: term.first_seen_timestamp,
                            last_seen_timestamp: term.last_seen_timestamp,
                            character_group: character.clone(),
                        };

                        let json_line = serde_json::to_string(&record)
                            .map_err(|e| ValidationError::DataAccessError {
                                message: format!("Failed to serialize term {}: {}", term_id, e),
                            })?;
                        
                        writeln!(file, "{}", json_line)
                            .map_err(|e| ValidationError::DataAccessError {
                                message: format!("Failed to write to output file: {}", e),
                            })?;
                        
                        total_exported += 1;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load term {} from {}: {}", term_id, character, e);
                    }
                }
            }
        }

        Ok(total_exported)
    }

    /// Export dataset statistics to JSON
    pub fn export_statistics(&self, output_path: &str) -> Result<(), ValidationError> {
        let characters = self.data_access.get_config_names("solfunmeme-index")?;
        
        let mut total_terms = 0;
        let mut character_stats = HashMap::new();

        for character in &characters {
            match self.data_access.get_split_names("solfunmeme-index", character) {
                Ok(term_ids) => {
                    let count = term_ids.len();
                    character_stats.insert(character.clone(), count);
                    total_terms += count;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get terms for character {}: {}", character, e);
                    character_stats.insert(character.clone(), 0);
                }
            }
        }

        // Create a more flexible stats structure
        let stats = serde_json::json!({
            "total_characters": characters.len(),
            "total_terms": total_terms,
            "character_stats": character_stats
        });

        let json_output = serde_json::to_string_pretty(&stats)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to serialize statistics: {}", e),
            })?;

        fs::write(output_path, json_output)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to write statistics file {}: {}", output_path, e),
            })?;

        Ok(())
    }

    /// Create a sample dataset with a subset of terms for testing
    pub fn create_sample_dataset(&self, output_dir: &str, sample_size: usize) -> Result<usize, ValidationError> {
        let characters = self.data_access.get_config_names("solfunmeme-index")?;
        
        // Create output directory
        fs::create_dir_all(output_dir)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to create output directory {}: {}", output_dir, e),
            })?;

        let mut total_exported = 0;
        let sample_per_char = (sample_size / characters.len()).max(1);

        for character in characters {
            let term_ids = match self.data_access.get_split_names("solfunmeme-index", &character) {
                Ok(ids) => ids,
                Err(_) => continue,
            };

            let char_output_path = format!("{}/{}_sample.jsonl", output_dir, character);
            let mut file = File::create(&char_output_path)
                .map_err(|e| ValidationError::DataAccessError {
                    message: format!("Failed to create file {}: {}", char_output_path, e),
                })?;

            let sample_terms: Vec<_> = term_ids.into_iter().take(sample_per_char).collect();
            
            for term_id in sample_terms {
                match self.data_access.load_term(&term_id) {
                    Ok(term) => {
                        let record = TermRecord {
                            id: term_id.clone(),
                            term: term.term,
                            count: term.count,
                            category: term.category,
                            significance: term.significance,
                            vibe: term.vibe,
                            action_suggestion: term.action_suggestion,
                            emoji_representation: term.emoji_representation,
                            semantic_names: term.semantic_names,
                            osi_layer: term.osi_layer,
                            prime_factor: term.prime_factor,
                            is_power_of_two: term.is_power_of_two,
                            numerical_address: term.numerical_address,
                            first_seen_timestamp: term.first_seen_timestamp,
                            last_seen_timestamp: term.last_seen_timestamp,
                            character_group: character.clone(),
                        };

                        let json_line = serde_json::to_string(&record)
                            .map_err(|e| ValidationError::DataAccessError {
                                message: format!("Failed to serialize term {}: {}", term_id, e),
                            })?;
                        
                        writeln!(file, "{}", json_line)
                            .map_err(|e| ValidationError::DataAccessError {
                                message: format!("Failed to write to file: {}", e),
                            })?;
                        
                        total_exported += 1;
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(total_exported)
    }
}

/// CLI function to run data conversion
pub fn run_data_conversion(base_path: &str, command: &str, output_path: &str) -> Result<(), ValidationError> {
    let converter = DataConverter::new(base_path)?;
    
    match command {
        "export-all" => {
            println!("Exporting all terms to JSONL format...");
            let count = converter.export_all_to_jsonl(output_path)?;
            println!("✅ Exported {} terms to {}", count, output_path);
        }
        "export-stats" => {
            println!("Exporting dataset statistics...");
            converter.export_statistics(output_path)?;
            println!("✅ Exported statistics to {}", output_path);
        }
        "create-sample" => {
            println!("Creating sample dataset...");
            let count = converter.create_sample_dataset(output_path, 1000)?;
            println!("✅ Created sample dataset with {} terms in {}", count, output_path);
        }
        _ => {
            return Err(ValidationError::InvalidEntityIdentifier {
                message: format!("Unknown conversion command: {}", command),
            });
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_data_converter() {
        let base_path = "/home/mdupont/2025/08/07/solfunmeme-index";
        
        // Skip test if directory doesn't exist
        if !Path::new(base_path).exists() {
            println!("Skipping test - dataset not found at {}", base_path);
            return;
        }
        
        let converter = DataConverter::new(base_path).unwrap();
        
        // Test exporting a single character
        let temp_file = "/tmp/test_export.jsonl";
        let result = converter.export_character_to_jsonl("a", temp_file);
        assert!(result.is_ok());
        
        let count = result.unwrap();
        assert!(count > 0);
        
        // Clean up
        let _ = fs::remove_file(temp_file);
    }
}
