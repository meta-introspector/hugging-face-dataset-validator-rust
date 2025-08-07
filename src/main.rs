mod validator;
mod solfunmeme_validator;
mod data_converter;
mod hf_dataset_converter;
mod parquet_validator;
mod dataset_loader_example;

use validator::{
    DatasetValidator, MockDataAccess, EntityIdentifier, ValidationLevel,
    validate_split, validate_config, validate_dataset, ValidationError
};
use std::env;

fn main() -> Result<(), ValidationError> {
    // Use tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main())
}

async fn async_main() -> Result<(), ValidationError> {
    println!("🚀 Hugging Face Dataset Validator - Rust Implementation");
    println!("======================================================\n");

    let args: Vec<String> = env::args().collect();
    
    match args.get(1).map(|s| s.as_str()) {
        Some("test-mock") => {
            println!("Running mock dataset tests...\n");
            test_mock_dataset()?;
        }
        Some("test-solfunmeme") => {
            println!("Running solfunmeme dataset tests...\n");
            test_solfunmeme_dataset()?;
        }
        Some("benchmark") => {
            println!("Running performance benchmarks...\n");
            run_benchmarks()?;
        }
        Some("export-all") => {
            println!("Exporting solfunmeme dataset to JSONL...\n");
            let base_path = "/home/mdupont/2025/08/07/solfunmeme-index";
            let output_path = args.get(2).map(|s| s.as_str()).unwrap_or("solfunmeme_export.jsonl");
            data_converter::run_data_conversion(base_path, "export-all", output_path)?;
        }
        Some("export-stats") => {
            println!("Exporting solfunmeme dataset statistics...\n");
            let base_path = "/home/mdupont/2025/08/07/solfunmeme-index";
            let output_path = args.get(2).map(|s| s.as_str()).unwrap_or("solfunmeme_stats.json");
            data_converter::run_data_conversion(base_path, "export-stats", output_path)?;
        }
        Some("create-sample") => {
            println!("Creating sample dataset...\n");
            let base_path = "/home/mdupont/2025/08/07/solfunmeme-index";
            let output_path = args.get(2).map(|s| s.as_str()).unwrap_or("solfunmeme_sample");
            data_converter::run_data_conversion(base_path, "create-sample", output_path)?;
        }
        Some("create-hf-dataset") => {
            println!("Creating Hugging Face dataset...\n");
            let base_path = "/home/mdupont/2025/08/07/solfunmeme-index";
            let output_path = args.get(2).map(|s| s.as_str()).unwrap_or("solfunmeme-hf-dataset");
            hf_dataset_converter::create_huggingface_dataset(base_path, output_path).await?;
        }
        Some("validate-parquet") => {
            println!("Validating Parquet dataset...\n");
            let dataset_path = args.get(2).map(|s| s.as_str()).unwrap_or("solfunmeme-hf-dataset");
            parquet_validator::validate_parquet_dataset(dataset_path)?;
        }
        Some("demo-dataset") => {
            println!("Demonstrating dataset loading...\n");
            let dataset_path = args.get(2).map(|s| s.as_str()).unwrap_or("solfunmeme-hf-dataset");
            dataset_loader_example::demonstrate_dataset_loading(dataset_path)?;
        }
        _ => {
            println!("Available commands:");
            println!("  test-mock        - Test with mock data");
            println!("  test-solfunmeme  - Test with solfunmeme-index dataset");
            println!("  benchmark        - Run performance benchmarks");
            println!("  export-all [file] - Export all solfunmeme terms to JSONL");
            println!("  export-stats [file] - Export dataset statistics to JSON");
            println!("  create-sample [dir] - Create sample dataset for testing");
            println!("  create-hf-dataset [dir] - Create Hugging Face dataset with Parquet files");
            println!("  validate-parquet [dir] - Validate Hugging Face Parquet dataset");
            println!("  demo-dataset [dir] - Demonstrate dataset loading and usage");
            println!("\nRunning mock tests by default...\n");
            
            test_mock_dataset()?;
        }
    }

    Ok(())
}

fn test_mock_dataset() -> Result<(), ValidationError> {
    println!("=== Mock Dataset Validation Tests ===\n");
    
    let service = MockDataAccess::default();
    let validator = DatasetValidator::new(service.clone());
    
    // Test individual validation levels
    println!("1. Testing individual validation levels:");
    
    // Split validation
    let (result, progress) = validate_split("mock/dataset", "default", "train", service.clone())?;
    println!("   Split (mock/dataset/default/train):");
    println!("     Capabilities: {:?}", result);
    println!("     Progress: {:.1}%", progress * 100.0);
    println!("     Score: {}/5", result.capability_count());
    
    // Config validation
    let (result, progress) = validate_config("mock/dataset", "default", service.clone())?;
    println!("   Config (mock/dataset/default):");
    println!("     Capabilities: {:?}", result);
    println!("     Progress: {:.1}%", progress * 100.0);
    println!("     Score: {}/5", result.capability_count());
    
    // Dataset validation
    let (result, progress) = validate_dataset("mock/dataset", service.clone())?;
    println!("   Dataset (mock/dataset):");
    println!("     Capabilities: {:?}", result);
    println!("     Progress: {:.1}%", progress * 100.0);
    println!("     Score: {}/5", result.capability_count());
    
    println!();
    
    // Test batch validation
    println!("2. Testing batch validation:");
    let entities = vec![
        (EntityIdentifier::new_split("mock/dataset".to_string(), "default".to_string(), "train".to_string()), ValidationLevel::Split),
        (EntityIdentifier::new_split("mock/dataset".to_string(), "default".to_string(), "test".to_string()), ValidationLevel::Split),
        (EntityIdentifier::new_config("mock/dataset".to_string(), "default".to_string()), ValidationLevel::Config),
        (EntityIdentifier::new_dataset("mock/dataset".to_string()), ValidationLevel::Dataset),
    ];
    
    let mut successful = 0;
    let mut total_capabilities = 0;
    
    for (entity, level) in &entities {
        match validator.validate(entity, *level) {
            Ok((result, progress)) => {
                successful += 1;
                total_capabilities += result.capability_count();
                println!("   ✅ {} ({}) - {}/5 capabilities, {:.1}% progress", 
                         entity, format!("{:?}", level).to_lowercase(), result.capability_count(), progress * 100.0);
            }
            Err(e) => {
                println!("   ❌ {} ({}) - Error: {}", entity, format!("{:?}", level).to_lowercase(), e);
            }
        }
    }
    
    println!("   Summary: {}/{} successful, {} total capabilities", successful, entities.len(), total_capabilities);
    
    Ok(())
}

fn test_solfunmeme_dataset() -> Result<(), ValidationError> {
    println!("=== Solfunmeme Dataset Tests ===\n");
    
    let base_path = "/home/mdupont/2025/08/07/solfunmeme-index";
    
    // Check if the dataset exists
    if !std::path::Path::new(base_path).exists() {
        println!("❌ Solfunmeme dataset not found at {}", base_path);
        println!("   Please ensure the dataset is available at this path.");
        return Ok(());
    }
    
    println!("📁 Dataset found at {}", base_path);
    println!("🔄 Using real SolfunmemeDataAccess implementation");
    
    // Use the real solfunmeme validator
    match solfunmeme_validator::test_solfunmeme_dataset() {
        Ok(()) => {
            println!("\n✅ Solfunmeme dataset validation completed successfully!");
        }
        Err(e) => {
            println!("\n❌ Solfunmeme dataset validation failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

fn run_benchmarks() -> Result<(), ValidationError> {
    println!("=== Performance Benchmarks ===\n");
    
    let service = MockDataAccess::default();
    let validator = DatasetValidator::new(service.clone());
    
    // Benchmark single validations
    println!("1. Single validation benchmarks:");
    
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = validate_split("benchmark/dataset", "default", "train", service.clone())?;
    }
    let duration = start.elapsed();
    println!("   1000 split validations: {:?} ({:.2}μs per validation)", 
             duration, duration.as_micros() as f64 / 1000.0);
    
    // Benchmark batch validations
    println!("2. Batch validation benchmarks:");
    
    let entities: Vec<_> = (0..100).map(|i| {
        (EntityIdentifier::new_split(
            "benchmark/dataset".to_string(),
            "default".to_string(),
            format!("split_{}", i)
        ), ValidationLevel::Split)
    }).collect();
    
    let start = std::time::Instant::now();
    let mut successful = 0;
    for (entity, level) in &entities {
        if validator.validate(entity, *level).is_ok() {
            successful += 1;
        }
    }
    let duration = start.elapsed();
    
    println!("   100 entity batch: {:?} ({:.2}μs per entity, {}/{} successful)", 
             duration, duration.as_micros() as f64 / 100.0, successful, entities.len());
    
    // Memory usage test
    println!("3. Memory usage test:");
    let start = std::time::Instant::now();
    let large_entities: Vec<_> = (0..10000).map(|i| {
        EntityIdentifier::new_split(
            format!("dataset_{}", i % 100),
            format!("config_{}", i % 10),
            format!("split_{}", i % 3)
        )
    }).collect();
    let creation_time = start.elapsed();
    
    println!("   Created 10,000 entities in {:?}", creation_time);
    println!("   Memory usage: ~{} KB (estimated)", large_entities.len() * std::mem::size_of::<EntityIdentifier>() / 1024);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        let service = MockDataAccess::default();
        
        // Test split validation - use a dataset that exists in mock data
        let result = validate_split("mock/dataset", "default", "train", service);
        assert!(result.is_ok());
        
        let (validation_result, progress) = result.unwrap();
        assert_eq!(progress, 1.0);
        assert!(validation_result.has_any_capability());
    }

    #[test]
    fn test_entity_identifier() {
        let entity = EntityIdentifier::new_split("test".to_string(), "config".to_string(), "split".to_string());
        assert_eq!(entity.dataset, "test");
        assert_eq!(entity.config, Some("config".to_string()));
        assert_eq!(entity.split, Some("split".to_string()));
        assert_eq!(entity.infer_level(), ValidationLevel::Split);
        assert_eq!(entity.to_string(), "test/config/split");
    }

    #[test]
    fn test_validation_result() {
        let mut result1 = validator::ValidationResult {
            viewer: true,
            preview: false,
            search: true,
            filter: false,
            statistics: true,
        };
        
        let result2 = validator::ValidationResult {
            viewer: false,
            preview: true,
            search: false,
            filter: true,
            statistics: false,
        };
        
        result1.merge(&result2);
        
        assert!(result1.viewer);
        assert!(result1.preview);
        assert!(result1.search);
        assert!(result1.filter);
        assert!(result1.statistics);
        assert_eq!(result1.capability_count(), 5);
    }
}
