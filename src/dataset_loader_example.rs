use std::collections::HashMap;
use std::fs;
use std::path::Path;

use arrow::array::{BooleanArray, Int64Array, StringArray, UInt32Array};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use serde::{Deserialize, Serialize};

use crate::validator::ValidationError;

/// Example dataset loader that mimics Hugging Face datasets library behavior
pub struct DatasetLoader {
    dataset_dir: String,
    splits: HashMap<String, Vec<String>>, // split_name -> list of parquet files
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetExample {
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

impl DatasetLoader {
    pub fn new(dataset_dir: &str) -> Result<Self, ValidationError> {
        if !Path::new(dataset_dir).exists() {
            return Err(ValidationError::DataAccessError {
                message: format!("Dataset directory does not exist: {}", dataset_dir),
            });
        }

        let mut loader = Self {
            dataset_dir: dataset_dir.to_string(),
            splits: HashMap::new(),
        };

        loader.discover_splits()?;
        Ok(loader)
    }

    /// Discover available splits and their files
    fn discover_splits(&mut self) -> Result<(), ValidationError> {
        let entries = fs::read_dir(&self.dataset_dir)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to read dataset directory: {}", e),
            })?;

        for entry in entries {
            let entry = entry.map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to read directory entry: {}", e),
            })?;

            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "parquet" {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        let split_name = filename.split('-').next().unwrap_or("unknown").to_string();
                        
                        self.splits.entry(split_name)
                            .or_insert_with(Vec::new)
                            .push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // Sort files within each split
        for files in self.splits.values_mut() {
            files.sort();
        }

        Ok(())
    }

    /// Get available split names
    pub fn get_splits(&self) -> Vec<String> {
        let mut splits: Vec<String> = self.splits.keys().cloned().collect();
        splits.sort();
        splits
    }

    /// Load a specific split
    pub fn load_split(&self, split_name: &str) -> Result<Vec<DatasetExample>, ValidationError> {
        let files = self.splits.get(split_name)
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("Split '{}' not found", split_name),
            })?;

        let mut examples = Vec::new();
        
        for file_path in files {
            let file_examples = self.load_parquet_file(file_path)?;
            examples.extend(file_examples);
        }

        Ok(examples)
    }

    /// Load examples from a single Parquet file
    fn load_parquet_file(&self, file_path: &str) -> Result<Vec<DatasetExample>, ValidationError> {
        let file = fs::File::open(file_path)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to open Parquet file {}: {}", file_path, e),
            })?;

        let builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to create Parquet reader for {}: {}", file_path, e),
            })?;

        let mut reader = builder.build()
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to build Parquet reader: {}", e),
            })?;

        let mut examples = Vec::new();

        while let Some(batch_result) = reader.next() {
            let batch = batch_result
                .map_err(|e| ValidationError::DataAccessError {
                    message: format!("Failed to read batch: {}", e),
                })?;

            let batch_examples = self.batch_to_examples(&batch)?;
            examples.extend(batch_examples);
        }

        Ok(examples)
    }

    /// Convert Arrow RecordBatch to DatasetExample structs
    fn batch_to_examples(&self, batch: &RecordBatch) -> Result<Vec<DatasetExample>, ValidationError> {
        use arrow::array::*;

        let num_rows = batch.num_rows();
        let mut examples = Vec::with_capacity(num_rows);

        // Get column arrays
        let id_array = self.get_string_column(batch, "id")?;
        let term_array = self.get_string_column(batch, "term")?;
        let count_array = self.get_uint32_column(batch, "count")?;
        let category_array = self.get_string_column(batch, "category")?;
        let significance_array = self.get_string_column(batch, "significance")?;
        let vibe_array = self.get_string_column(batch, "vibe")?;
        let action_suggestion_array = self.get_string_column(batch, "action_suggestion")?;
        let emoji_array = self.get_optional_string_column(batch, "emoji_representation")?;
        let osi_layer_array = self.get_optional_string_column(batch, "osi_layer")?;
        let prime_factor_array = self.get_optional_int64_column(batch, "prime_factor")?;
        let is_power_of_two_array = self.get_optional_boolean_column(batch, "is_power_of_two")?;
        let numerical_address_array = self.get_optional_string_column(batch, "numerical_address")?;
        let first_seen_array = self.get_optional_int64_column(batch, "first_seen_timestamp")?;
        let last_seen_array = self.get_optional_int64_column(batch, "last_seen_timestamp")?;
        let character_group_array = self.get_string_column(batch, "character_group")?;

        for i in 0..num_rows {
            let example = DatasetExample {
                id: id_array.value(i).to_string(),
                term: term_array.value(i).to_string(),
                count: count_array.value(i),
                category: category_array.value(i).to_string(),
                significance: significance_array.value(i).to_string(),
                vibe: vibe_array.value(i).to_string(),
                action_suggestion: action_suggestion_array.value(i).to_string(),
                emoji_representation: if emoji_array.is_null(i) { None } else { Some(emoji_array.value(i).to_string()) },
                semantic_names: None, // TODO: Handle list arrays
                osi_layer: if osi_layer_array.is_null(i) { None } else { Some(osi_layer_array.value(i).to_string()) },
                prime_factor: if prime_factor_array.is_null(i) { None } else { Some(prime_factor_array.value(i) as u64) },
                is_power_of_two: if is_power_of_two_array.is_null(i) { None } else { Some(is_power_of_two_array.value(i)) },
                numerical_address: if numerical_address_array.is_null(i) { None } else { Some(numerical_address_array.value(i).to_string()) },
                first_seen_timestamp: if first_seen_array.is_null(i) { None } else { Some(first_seen_array.value(i) as u64) },
                last_seen_timestamp: if last_seen_array.is_null(i) { None } else { Some(last_seen_array.value(i) as u64) },
                character_group: character_group_array.value(i).to_string(),
            };
            examples.push(example);
        }

        Ok(examples)
    }

    /// Helper to get string column
    fn get_string_column<'a>(&self, batch: &'a RecordBatch, column_name: &str) -> Result<&'a StringArray, ValidationError> {
        let column = batch.column_by_name(column_name)
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("Column '{}' not found", column_name),
            })?;

        column.as_any().downcast_ref::<StringArray>()
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("Failed to downcast column '{}' to StringArray", column_name),
            })
    }

    /// Helper to get optional string column
    fn get_optional_string_column<'a>(&self, batch: &'a RecordBatch, column_name: &str) -> Result<&'a StringArray, ValidationError> {
        self.get_string_column(batch, column_name)
    }

    /// Helper to get uint32 column
    fn get_uint32_column<'a>(&self, batch: &'a RecordBatch, column_name: &str) -> Result<&'a UInt32Array, ValidationError> {
        let column = batch.column_by_name(column_name)
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("Column '{}' not found", column_name),
            })?;

        column.as_any().downcast_ref::<UInt32Array>()
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("Failed to downcast column '{}' to UInt32Array", column_name),
            })
    }

    /// Helper to get optional int64 column
    fn get_optional_int64_column<'a>(&self, batch: &'a RecordBatch, column_name: &str) -> Result<&'a Int64Array, ValidationError> {
        let column = batch.column_by_name(column_name)
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("Column '{}' not found", column_name),
            })?;

        column.as_any().downcast_ref::<Int64Array>()
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("Failed to downcast column '{}' to Int64Array", column_name),
            })
    }

    /// Helper to get optional boolean column
    fn get_optional_boolean_column<'a>(&self, batch: &'a RecordBatch, column_name: &str) -> Result<&'a BooleanArray, ValidationError> {
        let column = batch.column_by_name(column_name)
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("Column '{}' not found", column_name),
            })?;

        column.as_any().downcast_ref::<BooleanArray>()
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("Failed to downcast column '{}' to BooleanArray", column_name),
            })
    }

    /// Get dataset statistics
    pub fn get_stats(&self) -> Result<HashMap<String, usize>, ValidationError> {
        let mut stats = HashMap::new();
        
        for (split_name, _files) in &self.splits {
            let examples = self.load_split(split_name)?;
            stats.insert(split_name.clone(), examples.len());
        }

        Ok(stats)
    }

    /// Filter examples by criteria
    pub fn filter_examples<F>(&self, split_name: &str, predicate: F) -> Result<Vec<DatasetExample>, ValidationError>
    where
        F: Fn(&DatasetExample) -> bool,
    {
        let examples = self.load_split(split_name)?;
        Ok(examples.into_iter().filter(predicate).collect())
    }

    /// Search examples by term
    pub fn search_by_term(&self, split_name: &str, query: &str) -> Result<Vec<DatasetExample>, ValidationError> {
        self.filter_examples(split_name, |example| {
            example.term.to_lowercase().contains(&query.to_lowercase())
        })
    }

    /// Get examples by character group
    pub fn get_by_character_group(&self, split_name: &str, character_group: &str) -> Result<Vec<DatasetExample>, ValidationError> {
        self.filter_examples(split_name, |example| {
            example.character_group == character_group
        })
    }
}

/// CLI function to demonstrate dataset loading
pub fn demonstrate_dataset_loading(dataset_dir: &str) -> Result<(), ValidationError> {
    println!("📚 Demonstrating Dataset Loading");
    println!("{}", "=".repeat(50));

    let loader = DatasetLoader::new(dataset_dir)?;
    
    // Show available splits
    let splits = loader.get_splits();
    println!("Available splits: {:?}", splits);

    // Get statistics
    let stats = loader.get_stats()?;
    println!("\nDataset statistics:");
    for (split, count) in &stats {
        println!("  {}: {} examples", split, count);
    }

    // Load a small sample from train split
    if splits.contains(&"train".to_string()) {
        println!("\n🔍 Sample from train split:");
        let train_examples = loader.load_split("train")?;
        
        for (i, example) in train_examples.iter().take(3).enumerate() {
            println!("  Example {}:", i + 1);
            println!("    Term: '{}'", example.term);
            println!("    Count: {}", example.count);
            println!("    Character Group: '{}'", example.character_group);
            println!("    Category: '{}'", example.category);
        }

        // Demonstrate filtering
        println!("\n🔎 Searching for terms containing 'rust':");
        let rust_terms = loader.search_by_term("train", "rust")?;
        println!("  Found {} terms containing 'rust'", rust_terms.len());
        for term in rust_terms.iter().take(5) {
            println!("    '{}' (count: {})", term.term, term.count);
        }

        // Demonstrate character group filtering
        println!("\n📝 Terms in character group 'r':");
        let r_terms = loader.get_by_character_group("train", "r")?;
        println!("  Found {} terms in group 'r'", r_terms.len());
        for term in r_terms.iter().take(5) {
            println!("    '{}' (count: {})", term.term, term.count);
        }
    }

    println!("\n✅ Dataset loading demonstration completed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_loader() {
        let dataset_dir = "solfunmeme-hf-dataset";
        
        // Skip test if directory doesn't exist
        if !Path::new(dataset_dir).exists() {
            println!("Skipping test - dataset not found at {}", dataset_dir);
            return;
        }
        
        let loader = DatasetLoader::new(dataset_dir).unwrap();
        let splits = loader.get_splits();
        
        assert!(!splits.is_empty());
        assert!(splits.contains(&"train".to_string()));
        
        let stats = loader.get_stats().unwrap();
        assert!(stats.get("train").unwrap() > &0);
    }
}
