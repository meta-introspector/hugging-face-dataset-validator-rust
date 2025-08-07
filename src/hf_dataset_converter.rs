use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::sync::Arc;

use arrow::array::{
    ArrayRef, BooleanArray, Int64Array, ListArray, StringArray, UInt32Array,
};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;

use crate::solfunmeme_validator::SolfunmemeDataAccess;
use crate::validator::{DataAccess, ValidationError};

/// Hugging Face dataset converter for solfunmeme-index
pub struct HuggingFaceDatasetConverter {
    data_access: SolfunmemeDataAccess,
    output_dir: String,
}

/// Standard Hugging Face dataset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetConfig {
    pub dataset_name: String,
    pub description: String,
    pub version: String,
    pub license: String,
    pub homepage: String,
    pub repository: String,
    pub tags: Vec<String>,
    pub task_categories: Vec<String>,
    pub language: Vec<String>,
    pub size_categories: String,
}

/// Dataset info structure for Hugging Face
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    pub description: String,
    pub citation: String,
    pub homepage: String,
    pub license: String,
    pub features: HashMap<String, FeatureInfo>,
    pub splits: HashMap<String, SplitInfo>,
    pub download_size: u64,
    pub dataset_size: u64,
    pub config_name: String,
    pub dataset_name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    pub dtype: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_label: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitInfo {
    pub name: String,
    pub num_bytes: u64,
    pub num_examples: u64,
    pub dataset_name: String,
}

impl HuggingFaceDatasetConverter {
    pub fn new(base_path: &str, output_dir: &str) -> Result<Self, ValidationError> {
        let data_access = SolfunmemeDataAccess::new(base_path);
        data_access.health_check()?;
        
        // Create output directory structure
        fs::create_dir_all(output_dir).map_err(|e| ValidationError::DataAccessError {
            message: format!("Failed to create output directory {}: {}", output_dir, e),
        })?;

        Ok(Self {
            data_access,
            output_dir: output_dir.to_string(),
        })
    }

    /// Create the complete Hugging Face dataset structure
    pub async fn create_huggingface_dataset(&self) -> Result<(), ValidationError> {
        println!("🚀 Creating Hugging Face dataset structure...");

        // 1. Create dataset configuration
        self.create_dataset_config()?;
        
        // 2. Create README.md
        self.create_readme()?;
        
        // 3. Convert data to Parquet format
        self.convert_to_parquet().await?;
        
        // 4. Create dataset_info.json
        self.create_dataset_info().await?;
        
        // 5. Create state.json
        self.create_state_json()?;

        println!("✅ Hugging Face dataset created successfully!");
        Ok(())
    }

    /// Create dataset configuration (dataset_infos.json)
    fn create_dataset_config(&self) -> Result<(), ValidationError> {
        let config = DatasetConfig {
            dataset_name: "solfunmeme-index".to_string(),
            description: "A comprehensive semantic analysis dataset containing terms extracted from the ragit codebase, organized by first character and enriched with metadata for AI-powered code understanding.".to_string(),
            version: "1.0.0".to_string(),
            license: "agpl-3.0".to_string(),
            homepage: "https://github.com/your-org/solfunmeme-index".to_string(),
            repository: "https://github.com/your-org/solfunmeme-index".to_string(),
            tags: vec![
                "code-understanding".to_string(),
                "semantic-analysis".to_string(),
                "rust".to_string(),
                "ai".to_string(),
                "codebase".to_string(),
                "index".to_string(),
                "nlp".to_string(),
                "programming".to_string(),
            ],
            task_categories: vec![
                "text-classification".to_string(),
                "feature-extraction".to_string(),
                "text-retrieval".to_string(),
            ],
            language: vec!["en".to_string(), "code".to_string()],
            size_categories: "10K<n<100K".to_string(),
        };

        let config_path = format!("{}/dataset_infos.json", self.output_dir);
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, config_json).map_err(|e| ValidationError::DataAccessError {
            message: format!("Failed to write dataset config: {}", e),
        })?;

        Ok(())
    }

    /// Create comprehensive README.md
    fn create_readme(&self) -> Result<(), ValidationError> {
        let readme_content = r#"---
license: agpl-3.0
task_categories:
- text-classification
- feature-extraction
- text-retrieval
language:
- en
- code
tags:
- code-understanding
- semantic-analysis
- rust
- ai
- codebase
- index
- nlp
- programming
size_categories: 10K<n<100K
---

# Solfunmeme Index Dataset

## Dataset Description

The Solfunmeme Index is a comprehensive semantic analysis dataset containing terms extracted from the ragit codebase. It's designed to help AI systems understand and navigate complex codebases through semantic term analysis and relationship mapping.

## Dataset Structure

The dataset contains **26,236 terms** organized across **103 character groups**, including:
- English terms from code documentation and comments
- Programming language keywords and identifiers
- Unicode characters from multiple languages (Korean, Bengali, Arabic, Mathematical symbols)
- Semantic metadata for each term

### Features

- **id**: Unique identifier for each term
- **term**: The actual term/word
- **count**: Frequency of occurrence in the codebase
- **category**: Semantic category (if available)
- **significance**: Importance rating (if available)
- **vibe**: Emotional/contextual tone (if available)
- **action_suggestion**: Recommended actions (if available)
- **emoji_representation**: Associated emoji (if available)
- **semantic_names**: Alternative semantic names (if available)
- **osi_layer**: OSI layer classification (if available)
- **prime_factor**: Mathematical prime factor (if available)
- **is_power_of_two**: Boolean indicating if count is power of 2
- **numerical_address**: Memory/address information (if available)
- **first_seen_timestamp**: When term was first encountered
- **last_seen_timestamp**: When term was last encountered
- **character_group**: First character grouping (a-z, 0-9, unicode)

### Splits

The dataset is organized by character groups:
- **train**: Terms starting with letters a-z (majority of data)
- **validation**: Terms starting with numbers 0-9
- **test**: Terms starting with unicode characters

## Usage

```python
from datasets import load_dataset

# Load the full dataset
dataset = load_dataset("your-org/solfunmeme-index")

# Load specific split
train_data = load_dataset("your-org/solfunmeme-index", split="train")

# Example usage
for example in train_data:
    print(f"Term: {example['term']}")
    print(f"Count: {example['count']}")
    print(f"Character Group: {example['character_group']}")
```

## Use Cases

1. **Code Understanding**: Train models to understand semantic relationships in code
2. **Documentation Generation**: Generate meaningful documentation from code terms
3. **Code Search**: Improve code search and retrieval systems
4. **Semantic Analysis**: Analyze patterns in programming terminology
5. **AI-Assisted Development**: Power AI tools for code completion and suggestion

## Dataset Statistics

- **Total Terms**: 26,236
- **Character Groups**: 103
- **Languages**: Multiple (English, Korean, Bengali, Arabic, Mathematical)
- **Top Character Groups**:
  - 's': 2,648 terms
  - 'c': 2,378 terms
  - 'p': 1,745 terms
  - 'a': 1,474 terms
  - 'd': 1,426 terms

## Data Collection

The data was extracted from the ragit codebase using advanced semantic analysis techniques. Each term was processed to extract:
- Frequency information
- Contextual metadata
- Semantic relationships
- Character-based organization

## Licensing

This dataset is released under the AGPL-3.0 license. Please ensure compliance with the license terms when using this dataset.

## Citation

If you use this dataset in your research, please cite:

```bibtex
@dataset{solfunmeme_index_2025,
  title={Solfunmeme Index: A Semantic Analysis Dataset for Code Understanding},
  author={Your Organization},
  year={2025},
  url={https://huggingface.co/datasets/your-org/solfunmeme-index}
}
```

## Contact

For questions or issues regarding this dataset, please open an issue in the repository or contact the maintainers.
"#;

        let readme_path = format!("{}/README.md", self.output_dir);
        fs::write(readme_path, readme_content).map_err(|e| ValidationError::DataAccessError {
            message: format!("Failed to write README: {}", e),
        })?;

        Ok(())
    }

    /// Convert data to Parquet format with proper Hugging Face structure
    pub async fn convert_to_parquet(&self) -> Result<(), ValidationError> {
        println!("📦 Converting data to Parquet format...");

        // Get all character groups
        let characters = self.data_access.get_config_names("solfunmeme-index")?;
        
        // Create splits based on character types
        let mut train_chars = Vec::new();
        let mut validation_chars = Vec::new();
        let mut test_chars = Vec::new();

        for char in characters {
            let first_char = char.chars().next().unwrap_or('a');
            if first_char.is_ascii_alphabetic() {
                train_chars.push(char);
            } else if first_char.is_ascii_digit() {
                validation_chars.push(char);
            } else {
                test_chars.push(char);
            }
        }

        // Convert each split
        self.convert_split_to_parquet("train", &train_chars).await?;
        self.convert_split_to_parquet("validation", &validation_chars).await?;
        self.convert_split_to_parquet("test", &test_chars).await?;

        Ok(())
    }

    /// Convert a specific split to Parquet
    async fn convert_split_to_parquet(&self, split_name: &str, characters: &[String]) -> Result<(), ValidationError> {
        println!("  Converting {} split ({} character groups)...", split_name, characters.len());

        // Define Arrow schema
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("term", DataType::Utf8, false),
            Field::new("count", DataType::UInt32, false),
            Field::new("category", DataType::Utf8, false),
            Field::new("significance", DataType::Utf8, false),
            Field::new("vibe", DataType::Utf8, false),
            Field::new("action_suggestion", DataType::Utf8, false),
            Field::new("emoji_representation", DataType::Utf8, true),
            Field::new("semantic_names", DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))), true),
            Field::new("osi_layer", DataType::Utf8, true),
            Field::new("prime_factor", DataType::Int64, true),
            Field::new("is_power_of_two", DataType::Boolean, true),
            Field::new("numerical_address", DataType::Utf8, true),
            Field::new("first_seen_timestamp", DataType::Int64, true),
            Field::new("last_seen_timestamp", DataType::Int64, true),
            Field::new("character_group", DataType::Utf8, false),
        ]));

        // Collect all data for this split
        let mut all_data = Vec::new();
        
        for character in characters {
            match self.data_access.get_split_names("solfunmeme-index", character) {
                Ok(term_ids) => {
                    for term_id in term_ids {
                        match self.data_access.load_term(&term_id) {
                            Ok(term) => {
                                all_data.push((term_id, term, character.clone()));
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to load term {}: {}", term_id, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get terms for character {}: {}", character, e);
                }
            }
        }

        if all_data.is_empty() {
            println!("  No data found for {} split", split_name);
            return Ok(());
        }

        // Convert to Arrow arrays
        let batch_size = 10000; // Process in batches
        let mut batch_num = 0;

        for chunk in all_data.chunks(batch_size) {
            let record_batch = self.create_record_batch(schema.clone(), chunk)?;
            
            // Write to Parquet file
            let output_path = if chunk.len() == all_data.len() {
                format!("{}/{}-00000-of-00001.parquet", self.output_dir, split_name)
            } else {
                format!("{}/{}-{:05}-of-{:05}.parquet", self.output_dir, split_name, batch_num, (all_data.len() + batch_size - 1) / batch_size)
            };

            let file = File::create(&output_path).map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to create parquet file {}: {}", output_path, e),
            })?;

            let props = WriterProperties::builder().build();
            let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props)).map_err(|e| {
                ValidationError::DataAccessError {
                    message: format!("Failed to create Arrow writer: {}", e),
                }
            })?;

            writer.write(&record_batch).map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to write record batch: {}", e),
            })?;

            writer.close().map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to close writer: {}", e),
            })?;

            batch_num += 1;
        }

        println!("    ✅ {} split: {} records in {} files", split_name, all_data.len(), batch_num);
        Ok(())
    }

    /// Create Arrow RecordBatch from data
    fn create_record_batch(
        &self,
        schema: Arc<Schema>,
        data: &[(String, crate::solfunmeme_validator::IndexTerm, String)],
    ) -> Result<RecordBatch, ValidationError> {
        let _len = data.len();

        // Create arrays
        let ids: StringArray = data.iter().map(|(id, _, _)| Some(id.as_str())).collect();
        let terms: StringArray = data.iter().map(|(_, term, _)| Some(term.term.as_str())).collect();
        let counts: UInt32Array = data.iter().map(|(_, term, _)| Some(term.count)).collect();
        let categories: StringArray = data.iter().map(|(_, term, _)| Some(term.category.as_str())).collect();
        let significances: StringArray = data.iter().map(|(_, term, _)| Some(term.significance.as_str())).collect();
        let vibes: StringArray = data.iter().map(|(_, term, _)| Some(term.vibe.as_str())).collect();
        let action_suggestions: StringArray = data.iter().map(|(_, term, _)| Some(term.action_suggestion.as_str())).collect();
        
        let emoji_representations: StringArray = data.iter().map(|(_, term, _)| {
            term.emoji_representation.as_ref().map(|s| s.as_str())
        }).collect();

        // Handle semantic_names as List<String>
        let semantic_names_values: Vec<Option<String>> = data.iter().flat_map(|(_, term, _)| {
            match &term.semantic_names {
                Some(names) => names.iter().map(|name| Some(name.clone())).collect(),
                None => vec![],
            }
        }).collect();
        
        let semantic_names_offsets: Vec<i32> = {
            let mut offsets = vec![0i32];
            let mut current_offset = 0i32;
            for (_, term, _) in data {
                current_offset += term.semantic_names.as_ref().map(|names| names.len() as i32).unwrap_or(0);
                offsets.push(current_offset);
            }
            offsets
        };
        
        let semantic_names_string_array = StringArray::from(semantic_names_values);
        let semantic_names = ListArray::new(
            Arc::new(Field::new("item", DataType::Utf8, true)),
            OffsetBuffer::new(semantic_names_offsets.into()),
            Arc::new(semantic_names_string_array),
            None,
        );

        let osi_layers: StringArray = data.iter().map(|(_, term, _)| {
            term.osi_layer.as_ref().map(|s| s.as_str())
        }).collect();

        let prime_factors: Int64Array = data.iter().map(|(_, term, _)| {
            term.prime_factor.map(|pf| pf as i64)
        }).collect();

        let is_power_of_twos: BooleanArray = data.iter().map(|(_, term, _)| {
            term.is_power_of_two
        }).collect();

        let numerical_addresses: StringArray = data.iter().map(|(_, term, _)| {
            term.numerical_address.as_ref().map(|s| s.as_str())
        }).collect();

        let first_seen_timestamps: Int64Array = data.iter().map(|(_, term, _)| {
            term.first_seen_timestamp.map(|ts| ts as i64)
        }).collect();

        let last_seen_timestamps: Int64Array = data.iter().map(|(_, term, _)| {
            term.last_seen_timestamp.map(|ts| ts as i64)
        }).collect();

        let character_groups: StringArray = data.iter().map(|(_, _, char_group)| Some(char_group.as_str())).collect();

        // Create arrays vector
        let arrays: Vec<ArrayRef> = vec![
            Arc::new(ids),
            Arc::new(terms),
            Arc::new(counts),
            Arc::new(categories),
            Arc::new(significances),
            Arc::new(vibes),
            Arc::new(action_suggestions),
            Arc::new(emoji_representations),
            Arc::new(semantic_names),
            Arc::new(osi_layers),
            Arc::new(prime_factors),
            Arc::new(is_power_of_twos),
            Arc::new(numerical_addresses),
            Arc::new(first_seen_timestamps),
            Arc::new(last_seen_timestamps),
            Arc::new(character_groups),
        ];

        RecordBatch::try_new(schema, arrays).map_err(|e| ValidationError::DataAccessError {
            message: format!("Failed to create record batch: {}", e),
        })
    }

    /// Create dataset_info.json
    async fn create_dataset_info(&self) -> Result<(), ValidationError> {
        println!("📋 Creating dataset info...");

        // Calculate dataset statistics
        let characters = self.data_access.get_config_names("solfunmeme-index")?;
        let mut total_examples = 0;
        let mut splits = HashMap::new();

        // Calculate split sizes
        let mut train_size = 0;
        let mut validation_size = 0;
        let mut test_size = 0;

        for character in &characters {
            if let Ok(term_ids) = self.data_access.get_split_names("solfunmeme-index", character) {
                let count = term_ids.len() as u64;
                total_examples += count;

                let first_char = character.chars().next().unwrap_or('a');
                if first_char.is_ascii_alphabetic() {
                    train_size += count;
                } else if first_char.is_ascii_digit() {
                    validation_size += count;
                } else {
                    test_size += count;
                }
            }
        }

        splits.insert("train".to_string(), SplitInfo {
            name: "train".to_string(),
            num_bytes: train_size * 200, // Estimate ~200 bytes per record
            num_examples: train_size,
            dataset_name: "solfunmeme-index".to_string(),
        });

        splits.insert("validation".to_string(), SplitInfo {
            name: "validation".to_string(),
            num_bytes: validation_size * 200,
            num_examples: validation_size,
            dataset_name: "solfunmeme-index".to_string(),
        });

        splits.insert("test".to_string(), SplitInfo {
            name: "test".to_string(),
            num_bytes: test_size * 200,
            num_examples: test_size,
            dataset_name: "solfunmeme-index".to_string(),
        });

        // Define features
        let mut features = HashMap::new();
        features.insert("id".to_string(), FeatureInfo {
            dtype: "string".to_string(),
            description: "Unique identifier for the term".to_string(),
            class_label: None,
        });
        features.insert("term".to_string(), FeatureInfo {
            dtype: "string".to_string(),
            description: "The actual term/word from the codebase".to_string(),
            class_label: None,
        });
        features.insert("count".to_string(), FeatureInfo {
            dtype: "uint32".to_string(),
            description: "Frequency of occurrence in the codebase".to_string(),
            class_label: None,
        });
        features.insert("category".to_string(), FeatureInfo {
            dtype: "string".to_string(),
            description: "Semantic category classification".to_string(),
            class_label: None,
        });
        features.insert("character_group".to_string(), FeatureInfo {
            dtype: "string".to_string(),
            description: "First character grouping (a-z, 0-9, unicode)".to_string(),
            class_label: None,
        });

        let dataset_info = DatasetInfo {
            description: "A comprehensive semantic analysis dataset containing terms extracted from the ragit codebase".to_string(),
            citation: "@dataset{solfunmeme_index_2025,\n  title={Solfunmeme Index: A Semantic Analysis Dataset for Code Understanding},\n  author={Your Organization},\n  year={2025},\n  url={https://huggingface.co/datasets/your-org/solfunmeme-index}\n}".to_string(),
            homepage: "https://github.com/your-org/solfunmeme-index".to_string(),
            license: "agpl-3.0".to_string(),
            features,
            splits,
            download_size: total_examples * 200,
            dataset_size: total_examples * 200,
            config_name: "default".to_string(),
            dataset_name: "solfunmeme-index".to_string(),
            version: "1.0.0".to_string(),
        };

        let info_path = format!("{}/dataset_info.json", self.output_dir);
        let info_json = serde_json::to_string_pretty(&dataset_info)?;
        fs::write(info_path, info_json).map_err(|e| ValidationError::DataAccessError {
            message: format!("Failed to write dataset info: {}", e),
        })?;

        println!("    Total examples: {}", total_examples);
        println!("    Train: {} examples", train_size);
        println!("    Validation: {} examples", validation_size);
        println!("    Test: {} examples", test_size);

        Ok(())
    }

    /// Create state.json for Hugging Face
    fn create_state_json(&self) -> Result<(), ValidationError> {
        let state = serde_json::json!({
            "_data_files": [
                {"split": "train"},
                {"split": "validation"},
                {"split": "test"}
            ],
            "_fingerprint": uuid::Uuid::new_v4().to_string(),
            "_format_columns": null,
            "_format_kwargs": {},
            "_format_type": null,
            "_output_all_columns": true,
            "_split": null
        });

        let state_path = format!("{}/state.json", self.output_dir);
        let state_json = serde_json::to_string_pretty(&state)?;
        fs::write(state_path, state_json).map_err(|e| ValidationError::DataAccessError {
            message: format!("Failed to write state.json: {}", e),
        })?;

        Ok(())
    }
}

/// CLI function to create Hugging Face dataset
pub async fn create_huggingface_dataset(
    base_path: &str,
    output_dir: &str,
) -> Result<(), ValidationError> {
    let converter = HuggingFaceDatasetConverter::new(base_path, output_dir)?;
    converter.create_huggingface_dataset().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn test_hf_dataset_converter() {
        let base_path = "/home/mdupont/2025/08/07/solfunmeme-index";
        
        // Skip test if directory doesn't exist
        if !Path::new(base_path).exists() {
            println!("Skipping test - dataset not found at {}", base_path);
            return;
        }
        
        let output_dir = "/tmp/test_hf_dataset";
        let converter = HuggingFaceDatasetConverter::new(base_path, output_dir).unwrap();
        
        // Test creating a small sample
        let result = converter.create_dataset_config();
        assert!(result.is_ok());
        
        // Clean up
        let _ = fs::remove_dir_all(output_dir);
    }
}
