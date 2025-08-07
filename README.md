# Hugging Face Dataset Validator - Rust Implementation

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

A unified Rust library for validating and converting Hugging Face datasets with Parquet support, part of the **solfunmeme** project for semantic code analysis and AI-powered development tools.

## Overview

This project provides comprehensive tools for:
- **Dataset Validation**: Multi-level validation of Hugging Face dataset capabilities
- **Parquet Conversion**: Convert datasets to efficient Parquet format with proper HF structure
- **Schema Validation**: Ensure data integrity and consistency across dataset files
- **Performance Analysis**: Benchmark and optimize dataset operations

## Features

### 🔍 **Dataset Validation**
- Multi-level validation hierarchy (Split → Config → Dataset)
- 5-capability assessment: Viewer, Preview, Search, Filter, Statistics
- Progress tracking and comprehensive error handling
- Mock data support for testing and development

### 📦 **Parquet Support**
- Convert datasets to standard Hugging Face Parquet format
- Schema consistency validation across multiple files
- Efficient columnar storage with Arrow compatibility
- Type-safe data structures with proper serialization

### 🚀 **Performance**
- Pure Rust implementation for maximum performance
- Streaming processing for large datasets
- Concurrent batch processing
- Memory-efficient operations

### 🎯 **Solfunmeme Integration**
- Specialized support for solfunmeme-index semantic analysis dataset
- 26,236+ semantic terms from codebase analysis
- Character-based organization (a-z, 0-9, unicode)
- Rich metadata including frequency, categories, and semantic relationships

## Quick Start

### Installation

```bash
git clone https://github.com/solfunmeme/hf-dataset-validator-rust.git
cd hf-dataset-validator-rust
cargo build --release
```

### Basic Usage

```bash
# Test with mock data
cargo run -- test-mock

# Validate solfunmeme dataset
cargo run -- test-solfunmeme

# Create Hugging Face dataset
cargo run -- create-hf-dataset output-dir

# Validate Parquet dataset
cargo run -- validate-parquet dataset-dir

# Demonstrate dataset loading
cargo run -- demo-dataset dataset-dir
```

## Commands

| Command | Description |
|---------|-------------|
| `test-mock` | Test with mock data |
| `test-solfunmeme` | Test with solfunmeme-index dataset |
| `benchmark` | Run performance benchmarks |
| `export-all [file]` | Export all solfunmeme terms to JSONL |
| `export-stats [file]` | Export dataset statistics to JSON |
| `create-sample [dir]` | Create sample dataset for testing |
| `create-hf-dataset [dir]` | Create Hugging Face dataset with Parquet files |
| `validate-parquet [dir]` | Validate Hugging Face Parquet dataset |
| `demo-dataset [dir]` | Demonstrate dataset loading and usage |

## Architecture

### Core Components

- **`validator.rs`** - Core validation framework with trait-based design
- **`solfunmeme_validator.rs`** - Solfunmeme dataset access implementation
- **`hf_dataset_converter.rs`** - Hugging Face dataset creation with Parquet export
- **`parquet_validator.rs`** - Parquet file validation and analysis
- **`dataset_loader_example.rs`** - Dataset loading and usage demonstration
- **`data_converter.rs`** - Data conversion utilities

### Data Flow

```
Original Dataset → Validation → Parquet Conversion → HF Dataset → Validation Report
```

## Dataset Structure

The solfunmeme-index dataset contains semantic analysis data with the following structure:

```rust
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
```

## Performance Metrics

- **Dataset Size**: 26,236 semantic terms
- **Storage Efficiency**: 0.65 MB in Parquet format
- **Processing Speed**: ~26K records processed in seconds
- **Memory Usage**: Streaming processing with minimal memory footprint
- **Validation Score**: 5/5 capabilities (Viewer, Preview, Search, Filter, Statistics)

## Development

### Prerequisites

- Rust 1.70+ 
- Cargo

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with specific dataset path
DATASET_PATH=/path/to/solfunmeme-index cargo run -- test-solfunmeme
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_mock_dataset

# Run with output
cargo test -- --nocapture
```

## Contributing

This project is part of the solfunmeme ecosystem for AI-powered development tools. Contributions are welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the AGPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## Solfunmeme Project

This is part of the larger **solfunmeme** project, which focuses on:
- Semantic analysis of codebases
- AI-powered development tools
- Code understanding and navigation
- Automated documentation generation

For more information about the solfunmeme project, visit our main repository.

## Contact

- **Author**: j mike dupont
- **Email**: h4@solfunmeme.com
- **Project**: [solfunmeme](https://github.com/solfunmeme)

## Acknowledgments

- Built with Rust and the Arrow/Parquet ecosystem
- Designed for compatibility with Hugging Face datasets
- Optimized for semantic code analysis workflows
