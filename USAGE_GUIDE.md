# 📚 Comprehensive Usage Guide

This guide provides detailed instructions for using the **Comprehensive Rust Compilation Analysis Toolkit** to extract semantic analysis, project structure, and LLVM IR generation data from Rust codebases.

## 🎯 **Quick Start**

### **1. Complete Pipeline Analysis** (Recommended)
The easiest way to get started is with complete pipeline analysis:

```bash
# Analyze any Rust project with all three extractors
cargo run --bin hf-validator -- analyze-rust-to-ir /path/to/rust/project output-dataset

# This creates a comprehensive dataset with:
# - Semantic analysis (parsing, name resolution, type inference)
# - Project structure analysis (Cargo metadata, dependencies)
# - LLVM IR generation analysis (across optimization levels)
```

### **2. Test with Simple Example**
```bash
# Create a test file
echo 'fn main() { println!("Hello, world!"); }' > test.rs

# Analyze it
cargo run --bin hf-validator -- analyze-rust-to-ir test.rs test-dataset

# Check the results
ls -la test-dataset/
```

## 📋 **Command Reference**

### **Core Analysis Commands**

#### `analyze-rust-to-ir` - Complete Pipeline Analysis
**Purpose**: Comprehensive analysis combining all three extractors  
**Usage**: `analyze-rust-to-ir <source> [output]`  
**Output**: Semantic + Project + LLVM IR analysis

```bash
# Basic usage
cargo run --bin hf-validator -- analyze-rust-to-ir /path/to/project output

# Real-world examples
cargo run --bin hf-validator -- analyze-rust-to-ir rust-analyzer rust-analyzer-complete
cargo run --bin hf-validator -- analyze-rust-to-ir ~/projects/my-rust-app my-app-analysis
```

#### `generate-hf-dataset` - Semantic Analysis Only
**Purpose**: Deep semantic analysis using rust-analyzer  
**Usage**: `generate-hf-dataset <source> [output]`  
**Output**: Parsing, name resolution, type inference data

```bash
# Semantic analysis only (faster for large projects)
cargo run --bin hf-validator -- generate-hf-dataset /path/to/project semantic-output

# Analyze specific directories
cargo run --bin hf-validator -- generate-hf-dataset rust/compiler rustc-semantic
cargo run --bin hf-validator -- generate-hf-dataset src/ my-project-semantic
```

#### `analyze-cargo-project` - Project Structure Analysis
**Purpose**: Extract Cargo metadata and project structure  
**Usage**: `analyze-cargo-project <source> [output] [include_deps]`  
**Output**: Project metadata, dependencies, build configuration

```bash
# Basic project analysis
cargo run --bin hf-validator -- analyze-cargo-project /path/to/project cargo-output

# Include dependency analysis (slower but more comprehensive)
cargo run --bin hf-validator -- analyze-cargo-project /path/to/project cargo-output true

# Workspace analysis
cargo run --bin hf-validator -- analyze-cargo-project rust/ rust-workspace-analysis
```

#### `analyze-llvm-ir` - LLVM IR Generation Analysis
**Purpose**: Analyze Rust → LLVM IR compilation pipeline  
**Usage**: `analyze-llvm-ir <source> [output] [opt_levels]`  
**Output**: IR generation across optimization levels

```bash
# Default optimization levels (O0,O1,O2,O3)
cargo run --bin hf-validator -- analyze-llvm-ir /path/to/project llvm-output

# Specific optimization levels
cargo run --bin hf-validator -- analyze-llvm-ir /path/to/project llvm-output O0,O2,O3

# Debug builds only
cargo run --bin hf-validator -- analyze-llvm-ir /path/to/project llvm-debug O0
```

### **Validation Commands**

#### `validate-hf-dataset` - Validate Semantic Analysis Dataset
```bash
# Validate semantic analysis output
cargo run --bin hf-validator -- validate-hf-dataset output-dataset/semantic

# Check specific dataset
cargo run --bin hf-validator -- validate-hf-dataset rust-analyzer-dataset
```

#### `validate-cargo-dataset` - Validate Project Analysis Dataset
```bash
# Validate cargo analysis output
cargo run --bin hf-validator -- validate-cargo-dataset output-dataset/cargo

# Check workspace analysis
cargo run --bin hf-validator -- validate-cargo-dataset rust-workspace-analysis
```

#### `validate-llvm-dataset` - Validate LLVM IR Dataset
```bash
# Validate LLVM IR analysis output
cargo run --bin hf-validator -- validate-llvm-dataset output-dataset/llvm-ir

# Check specific optimization analysis
cargo run --bin hf-validator -- validate-llvm-dataset llvm-debug
```

## 🏗️ **Real-World Examples**

### **Example 1: Analyze rust-analyzer** (533K records)
```bash
# Clone rust-analyzer
git clone https://github.com/rust-lang/rust-analyzer.git
cd rust-analyzer

# Complete analysis (takes ~20 minutes)
cargo run --bin hf-validator -- analyze-rust-to-ir . ../rust-analyzer-complete

# Or semantic analysis only (faster)
cargo run --bin hf-validator -- generate-hf-dataset . ../rust-analyzer-semantic

# Validate the results
cargo run --bin hf-validator -- validate-hf-dataset ../rust-analyzer-complete/semantic
```

### **Example 2: Analyze Rust Compiler** (835K records)
```bash
# Clone Rust compiler
git clone https://github.com/rust-lang/rust.git
cd rust

# Analyze just the compiler directory (recommended)
cargo run --bin hf-validator -- generate-hf-dataset compiler/ ../rustc-compiler-analysis

# Or analyze the entire workspace (very large)
cargo run --bin hf-validator -- analyze-cargo-project . ../rust-workspace-analysis

# Validate
cargo run --bin hf-validator -- validate-hf-dataset ../rustc-compiler-analysis
```

### **Example 3: Analyze LLVM Bindings** (9K records)
```bash
# Clone llvm-sys.rs
git clone https://gitlab.com/taricorp/llvm-sys.rs.git
cd llvm-sys.rs

# Complete pipeline analysis
cargo run --bin hf-validator -- analyze-rust-to-ir . ../llvm-sys-complete

# Check all phases
ls -la ../llvm-sys-complete/
```

### **Example 4: Analyze Your Own Project**
```bash
# Navigate to your project
cd /path/to/your/rust/project

# Quick semantic analysis
cargo run --bin hf-validator -- generate-hf-dataset . my-project-analysis

# Complete pipeline analysis
cargo run --bin hf-validator -- analyze-rust-to-ir . my-project-complete

# Validate results
cargo run --bin hf-validator -- validate-hf-dataset my-project-analysis
```

## 📊 **Understanding Output Structure**

### **Complete Pipeline Output**
```
output-dataset/
├── semantic/                    # Rust semantic analysis
│   ├── parsing-phase/
│   │   ├── data-00000-of-00009.parquet
│   │   ├── data-00001-of-00009.parquet
│   │   └── ...
│   ├── name_resolution-phase/
│   │   └── data.parquet
│   ├── type_inference-phase/
│   │   └── data.parquet
│   └── README.md
├── cargo/                       # Project structure analysis
│   ├── project_metadata-phase/
│   │   └── data.parquet
│   └── README.md
├── llvm-ir/                     # LLVM IR generation analysis
│   ├── ir_generation-O0-phase/
│   │   └── data.parquet
│   ├── ir_generation-O1-phase/
│   │   └── data.parquet
│   ├── ir_generation-O2-phase/
│   │   └── data.parquet
│   ├── ir_generation-O3-phase/
│   │   └── data.parquet
│   └── README.md
└── README.md                    # Master documentation
```

### **File Sizes and Record Counts**
- **Small projects** (< 100 files): ~1MB, ~1K records
- **Medium projects** (100-1K files): ~10MB, ~10K records  
- **Large projects** (1K-10K files): ~100MB, ~100K records
- **Massive projects** (10K+ files): ~1GB+, ~1M+ records

## 🔧 **Advanced Usage**

### **Performance Optimization**

#### For Large Projects (1000+ files)
```bash
# Start with semantic analysis only
cargo run --bin hf-validator -- generate-hf-dataset large-project semantic-only

# Add project analysis separately
cargo run --bin hf-validator -- analyze-cargo-project large-project cargo-analysis

# Skip LLVM IR analysis for very large projects (it's still experimental)
```

#### Memory Management
```bash
# Set Rust memory limits for very large projects
export RUST_MIN_STACK=8388608  # 8MB stack
export RUST_BACKTRACE=1        # Enable backtraces for debugging

# Run analysis
cargo run --release --bin hf-validator -- generate-hf-dataset huge-project output
```

### **Custom Analysis Workflows**

#### Analyze Specific Directories
```bash
# Analyze only source code
cargo run --bin hf-validator -- generate-hf-dataset src/ src-analysis

# Analyze tests separately
cargo run --bin hf-validator -- generate-hf-dataset tests/ test-analysis

# Analyze examples
cargo run --bin hf-validator -- generate-hf-dataset examples/ example-analysis
```

#### Batch Analysis
```bash
#!/bin/bash
# Analyze multiple projects
for project in project1 project2 project3; do
    echo "Analyzing $project..."
    cargo run --bin hf-validator -- analyze-rust-to-ir "$project" "${project}-analysis"
done
```

### **Integration with ML Workflows**

#### Loading Data in Python
```python
import pandas as pd
import pyarrow.parquet as pq

# Load semantic analysis data
parsing_df = pd.read_parquet('output-dataset/semantic/parsing-phase/data-00000-of-00009.parquet')
print(f"Loaded {len(parsing_df)} parsing records")

# Load project metadata
cargo_df = pd.read_parquet('output-dataset/cargo/project_metadata-phase/data.parquet')
print(f"Project: {cargo_df['project_name'].iloc[0]}")

# Load LLVM IR data
ir_df = pd.read_parquet('output-dataset/llvm-ir/ir_generation-O2-phase/data.parquet')
print(f"LLVM IR records: {len(ir_df)}")
```

#### Loading Data in Rust
```rust
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::fs::File;

fn load_dataset(path: &str) -> Result<Vec<RecordBatch>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let reader = builder.build()?;
    
    let mut batches = Vec::new();
    for batch_result in reader {
        batches.push(batch_result?);
    }
    
    Ok(batches)
}

// Usage
let batches = load_dataset("output-dataset/semantic/parsing-phase/data.parquet")?;
println!("Loaded {} batches", batches.len());
```

## 🐛 **Troubleshooting**

### **Common Issues**

#### "No such file or directory"
```bash
# Make sure the source path exists and contains Rust files
ls -la /path/to/rust/project
find /path/to/rust/project -name "*.rs" | head -5
```

#### "Failed to parse Cargo.toml"
```bash
# Check if Cargo.toml exists and is valid
cat /path/to/rust/project/Cargo.toml
cargo check --manifest-path /path/to/rust/project/Cargo.toml
```

#### "Out of memory" for large projects
```bash
# Use release build for better performance
cargo build --release
./target/release/hf-validator analyze-rust-to-ir large-project output

# Or analyze in smaller chunks
cargo run --bin hf-validator -- generate-hf-dataset src/ src-analysis
cargo run --bin hf-validator -- generate-hf-dataset tests/ test-analysis
```

#### "Permission denied" writing output
```bash
# Check write permissions
ls -la output-directory/
mkdir -p output-directory
chmod 755 output-directory
```

### **Validation Failures**

#### Missing Parquet files
```bash
# Check if analysis completed successfully
ls -la output-dataset/semantic/
ls -la output-dataset/cargo/
ls -la output-dataset/llvm-ir/

# Re-run analysis if files are missing
cargo run --bin hf-validator -- analyze-rust-to-ir source-project output-dataset
```

#### Schema validation errors
```bash
# Check Parquet file integrity
python3 -c "import pandas as pd; print(pd.read_parquet('data.parquet').info())"

# Re-generate if corrupted
rm -rf output-dataset
cargo run --bin hf-validator -- analyze-rust-to-ir source-project output-dataset
```

## 🎯 **Best Practices**

### **Project Selection**
- **Start small**: Test with simple projects first
- **Use release builds**: For large projects, use `cargo build --release`
- **Check disk space**: Large projects can generate GB of data
- **Validate incrementally**: Run validation after each analysis phase

### **Data Management**
- **Organize outputs**: Use descriptive directory names
- **Version control**: Track analysis parameters and versions
- **Backup important datasets**: Large analyses take time to regenerate
- **Document analysis**: Keep notes on analysis parameters and goals

### **Performance Tips**
- **Use SSD storage**: Significantly faster for large datasets
- **Increase memory**: Large projects benefit from more RAM
- **Parallel analysis**: Analyze different projects simultaneously
- **Monitor resources**: Watch CPU, memory, and disk usage

## 📚 **Next Steps**

### **Research Applications**
1. **Load datasets** into your ML framework
2. **Explore data patterns** in semantic analysis
3. **Train models** on compilation patterns
4. **Correlate** source patterns with performance

### **Tool Development**
1. **Extend extractors** with new analysis phases
2. **Add custom schemas** for specific research needs
3. **Integrate** with existing development tools
4. **Contribute** improvements back to the project

### **Educational Use**
1. **Study compilation** processes in real codebases
2. **Understand** how rust-analyzer works internally
3. **Learn** about LLVM IR generation
4. **Teach** compiler concepts with real data

---

**🚀 Ready to revolutionize Rust analysis and ML-powered development tools!**

For more information:
- **Documentation**: https://github.com/solfunmeme/hf-dataset-validator-rust
- **Dataset**: https://huggingface.co/datasets/introspector/rust
- **Issues**: https://github.com/solfunmeme/hf-dataset-validator-rust/issues
