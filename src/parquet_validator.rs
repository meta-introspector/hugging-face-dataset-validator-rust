use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use serde::{Deserialize, Serialize};

use crate::validator::{ValidationError, ValidationResult};

/// Parquet file validator for Hugging Face datasets
pub struct ParquetValidator {
    dataset_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParquetFileInfo {
    pub filename: String,
    pub num_rows: usize,
    pub num_columns: usize,
    pub columns: Vec<String>,
    pub file_size_bytes: u64,
    pub split_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetValidationReport {
    pub dataset_name: String,
    pub total_files: usize,
    pub total_rows: usize,
    pub total_size_bytes: u64,
    pub splits: HashMap<String, SplitValidationInfo>,
    pub schema_consistency: bool,
    pub validation_result: ValidationResult,
    pub sample_records: Vec<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitValidationInfo {
    pub split_name: String,
    pub num_files: usize,
    pub num_rows: usize,
    pub size_bytes: u64,
    pub files: Vec<ParquetFileInfo>,
}

impl ParquetValidator {
    pub fn new(dataset_dir: &str) -> Result<Self, ValidationError> {
        if !Path::new(dataset_dir).exists() {
            return Err(ValidationError::DataAccessError {
                message: format!("Dataset directory does not exist: {}", dataset_dir),
            });
        }

        Ok(Self {
            dataset_dir: dataset_dir.to_string(),
        })
    }

    /// Validate the entire Hugging Face dataset
    pub fn validate_dataset(&self) -> Result<DatasetValidationReport, ValidationError> {
        println!("🔍 Validating Hugging Face dataset at: {}", self.dataset_dir);

        // Find all Parquet files
        let parquet_files = self.find_parquet_files()?;
        
        if parquet_files.is_empty() {
            return Err(ValidationError::DataAccessError {
                message: "No Parquet files found in dataset directory".to_string(),
            });
        }

        println!("📄 Found {} Parquet files", parquet_files.len());

        // Validate each file and collect info
        let mut file_infos = Vec::new();
        let mut total_rows = 0;
        let mut total_size = 0;
        let mut schemas = Vec::new();

        for file_path in &parquet_files {
            let file_info = self.validate_parquet_file(file_path)?;
            total_rows += file_info.num_rows;
            total_size += file_info.file_size_bytes;
            
            // Collect schema for consistency check
            let schema = self.get_file_schema(file_path)?;
            schemas.push(schema);
            
            file_infos.push(file_info);
        }

        // Check schema consistency
        let schema_consistency = self.check_schema_consistency(&schemas);
        if !schema_consistency {
            println!("⚠️  Warning: Schema inconsistency detected across files");
        }

        // Group by splits
        let splits = self.group_by_splits(&file_infos);

        // Generate validation result
        let validation_result = self.assess_capabilities(&file_infos)?;

        // Get sample records
        let sample_records = self.get_sample_records(&parquet_files[0])?;

        let report = DatasetValidationReport {
            dataset_name: self.extract_dataset_name(),
            total_files: parquet_files.len(),
            total_rows,
            total_size_bytes: total_size,
            splits,
            schema_consistency,
            validation_result,
            sample_records,
        };

        Ok(report)
    }

    /// Find all Parquet files in the dataset directory
    fn find_parquet_files(&self) -> Result<Vec<String>, ValidationError> {
        let entries = fs::read_dir(&self.dataset_dir)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to read dataset directory: {}", e),
            })?;

        let mut parquet_files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to read directory entry: {}", e),
            })?;

            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "parquet" {
                    if let Some(path_str) = path.to_str() {
                        parquet_files.push(path_str.to_string());
                    }
                }
            }
        }

        parquet_files.sort();
        Ok(parquet_files)
    }

    /// Validate a single Parquet file
    fn validate_parquet_file(&self, file_path: &str) -> Result<ParquetFileInfo, ValidationError> {
        let file = fs::File::open(file_path)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to open Parquet file {}: {}", file_path, e),
            })?;

        let builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to create Parquet reader for {}: {}", file_path, e),
            })?;

        let metadata = builder.metadata();
        let schema = builder.schema();
        
        // Get file metadata
        let file_metadata = fs::metadata(file_path)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to get file metadata for {}: {}", file_path, e),
            })?;

        let num_rows = metadata.file_metadata().num_rows() as usize;
        let num_columns = schema.fields().len();
        let columns: Vec<String> = schema.fields().iter()
            .map(|field| field.name().clone())
            .collect();

        let filename = Path::new(file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        let split_name = filename.split('-').next().unwrap_or("unknown").to_string();

        Ok(ParquetFileInfo {
            filename,
            num_rows,
            num_columns,
            columns,
            file_size_bytes: file_metadata.len(),
            split_name,
        })
    }

    /// Get schema from a Parquet file
    fn get_file_schema(&self, file_path: &str) -> Result<Arc<arrow::datatypes::Schema>, ValidationError> {
        let file = fs::File::open(file_path)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to open Parquet file {}: {}", file_path, e),
            })?;

        let builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|e| ValidationError::DataAccessError {
                message: format!("Failed to create Parquet reader for {}: {}", file_path, e),
            })?;

        Ok(builder.schema().clone())
    }

    /// Check if all files have consistent schemas
    fn check_schema_consistency(&self, schemas: &[Arc<arrow::datatypes::Schema>]) -> bool {
        if schemas.is_empty() {
            return true;
        }

        let first_schema = &schemas[0];
        schemas.iter().all(|schema| {
            schema.fields().len() == first_schema.fields().len() &&
            schema.fields().iter().zip(first_schema.fields().iter())
                .all(|(field1, field2)| {
                    field1.name() == field2.name() && 
                    field1.data_type() == field2.data_type()
                })
        })
    }

    /// Group files by split
    fn group_by_splits(&self, file_infos: &[ParquetFileInfo]) -> HashMap<String, SplitValidationInfo> {
        let mut splits = HashMap::new();

        for file_info in file_infos {
            let split_info = splits.entry(file_info.split_name.clone())
                .or_insert_with(|| SplitValidationInfo {
                    split_name: file_info.split_name.clone(),
                    num_files: 0,
                    num_rows: 0,
                    size_bytes: 0,
                    files: Vec::new(),
                });

            split_info.num_files += 1;
            split_info.num_rows += file_info.num_rows;
            split_info.size_bytes += file_info.file_size_bytes;
            split_info.files.push(file_info.clone());
        }

        splits
    }

    /// Assess dataset capabilities
    fn assess_capabilities(&self, file_infos: &[ParquetFileInfo]) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::new();

        // Viewer: Can view if files exist and are readable
        result.viewer = !file_infos.is_empty();

        // Preview: Can preview if we have data
        result.preview = file_infos.iter().any(|info| info.num_rows > 0);

        // Search: Can search if we have string columns
        result.search = file_infos.iter().any(|info| {
            info.columns.iter().any(|col| {
                col.contains("term") || col.contains("text") || col.contains("content")
            })
        });

        // Filter: Can filter if we have multiple columns
        result.filter = file_infos.iter().any(|info| info.num_columns > 1);

        // Statistics: Can provide statistics if we have numeric columns
        result.statistics = file_infos.iter().any(|info| {
            info.columns.iter().any(|col| {
                col.contains("count") || col.contains("id") || col.contains("timestamp")
            })
        });

        Ok(result)
    }

    /// Get sample records from a Parquet file
    fn get_sample_records(&self, file_path: &str) -> Result<Vec<HashMap<String, String>>, ValidationError> {
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

        // Read first batch
        if let Some(batch_result) = reader.next() {
            let batch = batch_result
                .map_err(|e| ValidationError::DataAccessError {
                    message: format!("Failed to read batch: {}", e),
                })?;

            return self.extract_sample_records(&batch);
        }

        Ok(Vec::new())
    }

    /// Extract sample records from a RecordBatch
    fn extract_sample_records(&self, batch: &RecordBatch) -> Result<Vec<HashMap<String, String>>, ValidationError> {
        let schema = batch.schema();
        let num_rows = batch.num_rows().min(5); // Get up to 5 sample records
        let mut records = Vec::new();

        for row_idx in 0..num_rows {
            let mut record = HashMap::new();
            
            for (col_idx, field) in schema.fields().iter().enumerate() {
                let column = batch.column(col_idx);
                let value = self.extract_value_at_index(column, row_idx)?;
                record.insert(field.name().clone(), value);
            }
            
            records.push(record);
        }

        Ok(records)
    }

    /// Extract value from Arrow array at specific index
    fn extract_value_at_index(&self, array: &dyn arrow::array::Array, index: usize) -> Result<String, ValidationError> {
        use arrow::array::*;
        use arrow::datatypes::DataType;

        if array.is_null(index) {
            return Ok("null".to_string());
        }

        match array.data_type() {
            DataType::Utf8 => {
                let string_array = array.as_any().downcast_ref::<StringArray>()
                    .ok_or_else(|| ValidationError::DataAccessError {
                        message: "Failed to downcast to StringArray".to_string(),
                    })?;
                Ok(string_array.value(index).to_string())
            }
            DataType::UInt32 => {
                let uint32_array = array.as_any().downcast_ref::<UInt32Array>()
                    .ok_or_else(|| ValidationError::DataAccessError {
                        message: "Failed to downcast to UInt32Array".to_string(),
                    })?;
                Ok(uint32_array.value(index).to_string())
            }
            DataType::Int64 => {
                let int64_array = array.as_any().downcast_ref::<Int64Array>()
                    .ok_or_else(|| ValidationError::DataAccessError {
                        message: "Failed to downcast to Int64Array".to_string(),
                    })?;
                Ok(int64_array.value(index).to_string())
            }
            DataType::Boolean => {
                let bool_array = array.as_any().downcast_ref::<BooleanArray>()
                    .ok_or_else(|| ValidationError::DataAccessError {
                        message: "Failed to downcast to BooleanArray".to_string(),
                    })?;
                Ok(bool_array.value(index).to_string())
            }
            DataType::List(_) => {
                Ok("[list]".to_string()) // Simplified for lists
            }
            _ => {
                Ok(format!("[{}]", array.data_type()))
            }
        }
    }

    /// Extract dataset name from directory
    fn extract_dataset_name(&self) -> String {
        Path::new(&self.dataset_dir)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Print validation report
    pub fn print_report(&self, report: &DatasetValidationReport) {
        println!("\n📊 Dataset Validation Report");
        println!("{}", "=".repeat(50));
        println!("Dataset: {}", report.dataset_name);
        println!("Total files: {}", report.total_files);
        println!("Total rows: {}", report.total_rows);
        println!("Total size: {:.2} MB", report.total_size_bytes as f64 / 1024.0 / 1024.0);
        println!("Schema consistency: {}", if report.schema_consistency { "✅" } else { "❌" });

        println!("\n🎯 Capabilities:");
        println!("  Viewer: {}", if report.validation_result.viewer { "✅" } else { "❌" });
        println!("  Preview: {}", if report.validation_result.preview { "✅" } else { "❌" });
        println!("  Search: {}", if report.validation_result.search { "✅" } else { "❌" });
        println!("  Filter: {}", if report.validation_result.filter { "✅" } else { "❌" });
        println!("  Statistics: {}", if report.validation_result.statistics { "✅" } else { "❌" });
        println!("  Overall Score: {}/5", report.validation_result.capability_count());

        println!("\n📂 Splits:");
        for (split_name, split_info) in &report.splits {
            println!("  {}: {} files, {} rows, {:.2} MB", 
                split_name, 
                split_info.num_files, 
                split_info.num_rows,
                split_info.size_bytes as f64 / 1024.0 / 1024.0
            );
        }

        if !report.sample_records.is_empty() {
            println!("\n🔍 Sample Records:");
            for (i, record) in report.sample_records.iter().take(3).enumerate() {
                println!("  Record {}:", i + 1);
                for (key, value) in record {
                    let display_value = if value.len() > 50 {
                        format!("{}...", &value[..47])
                    } else {
                        value.clone()
                    };
                    println!("    {}: {}", key, display_value);
                }
            }
        }
    }
}

/// CLI function to validate Parquet dataset
pub fn validate_parquet_dataset(dataset_dir: &str) -> Result<(), ValidationError> {
    let validator = ParquetValidator::new(dataset_dir)?;
    let report = validator.validate_dataset()?;
    validator.print_report(&report);
    
    // Export report to JSON
    let report_path = format!("{}/validation_report.json", dataset_dir);
    let report_json = serde_json::to_string_pretty(&report)?;
    fs::write(&report_path, report_json)
        .map_err(|e| ValidationError::DataAccessError {
            message: format!("Failed to write validation report: {}", e),
        })?;
    
    println!("\n📄 Validation report saved to: {}", report_path);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parquet_validator() {
        let dataset_dir = "solfunmeme-hf-dataset";
        
        // Skip test if directory doesn't exist
        if !Path::new(dataset_dir).exists() {
            println!("Skipping test - dataset not found at {}", dataset_dir);
            return;
        }
        
        let validator = ParquetValidator::new(dataset_dir).unwrap();
        let report = validator.validate_dataset().unwrap();
        
        assert!(report.total_files > 0);
        assert!(report.total_rows > 0);
        assert!(report.validation_result.viewer);
    }
}
