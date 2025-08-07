# Rust-Analyzer HF Dataset Integration Plan

## Overview
Integrate rust-analyzer with the HF dataset validator to generate datasets at each phase of Rust code processing, creating rich semantic analysis datasets for ML training.

## Architecture

### Phase 1: Core Integration
```
Rust Source Code → Rust-Analyzer → Phase Extractors → HF Dataset Generator → Parquet Files
```

### Phase 2: Multi-Phase Dataset Generation
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Rust Code     │───▶│  Rust-Analyzer   │───▶│ Phase Datasets  │
│   Repository    │    │   Processing     │    │   Generator     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │                         │
                              ▼                         ▼
                    ┌──────────────────┐    ┌─────────────────┐
                    │ Processing Phases│    │ HF Datasets     │
                    │ • Parsing        │    │ • Syntax Trees  │
                    │ • Name Resolution│    │ • Symbol Tables │
                    │ • Type Inference │    │ • Type Info     │
                    │ • HIR Generation │    │ • Diagnostics   │
                    │ • IDE Features   │    │ • Completions   │
                    └──────────────────┘    └─────────────────┘
```

## Implementation Plan

### 1. Create Rust-Analyzer Dataset Extractor

**File**: `src/rust_analyzer_extractor.rs`
```rust
pub struct RustAnalyzerExtractor {
    analyzer: RustAnalyzer,
    dataset_generator: HfDatasetGenerator,
}

pub enum ProcessingPhase {
    Parsing,
    NameResolution, 
    TypeInference,
    HirGeneration,
    Diagnostics,
    Completions,
}

pub struct PhaseDataset {
    phase: ProcessingPhase,
    source_file: String,
    timestamp: u64,
    data: PhaseData,
}
```

### 2. Phase-Specific Data Structures

#### Parsing Phase Dataset
```rust
pub struct ParsingPhaseData {
    pub file_path: String,
    pub source_code: String,
    pub syntax_tree: String, // Serialized syntax tree
    pub tokens: Vec<Token>,
    pub parse_errors: Vec<ParseError>,
    pub parse_time_ms: u64,
}
```

#### Name Resolution Phase Dataset  
```rust
pub struct NameResolutionPhaseData {
    pub file_path: String,
    pub symbols: Vec<Symbol>,
    pub scopes: Vec<Scope>,
    pub imports: Vec<Import>,
    pub unresolved_names: Vec<UnresolvedName>,
    pub resolution_time_ms: u64,
}
```

#### Type Inference Phase Dataset
```rust
pub struct TypeInferencePhaseData {
    pub file_path: String,
    pub type_assignments: Vec<TypeAssignment>,
    pub type_errors: Vec<TypeError>,
    pub inferred_types: Vec<InferredType>,
    pub inference_time_ms: u64,
}
```

### 3. Integration with Existing HF Dataset Validator

Extend the existing `DatasetExample` structure:

```rust
pub struct RustAnalyzerDatasetExample {
    pub id: String,
    pub phase: String,
    pub file_path: String,
    pub source_location: String, // line:col
    pub element_type: String, // function, struct, enum, etc.
    pub element_name: String,
    pub semantic_data: String, // JSON serialized phase-specific data
    pub processing_time_ms: u64,
    pub timestamp: u64,
    pub rust_version: String,
    pub analyzer_version: String,
}
```

### 4. Dataset Generation Workflow

#### Step 1: Rust-Analyzer Processing Hook
```rust
impl RustAnalyzerExtractor {
    pub fn process_codebase(&mut self, codebase_path: &Path) -> Result<Vec<PhaseDataset>> {
        let mut datasets = Vec::new();
        
        for rust_file in find_rust_files(codebase_path) {
            // Phase 1: Parsing
            let parse_data = self.extract_parsing_data(&rust_file)?;
            datasets.push(PhaseDataset::new(ProcessingPhase::Parsing, parse_data));
            
            // Phase 2: Name Resolution
            let resolution_data = self.extract_name_resolution_data(&rust_file)?;
            datasets.push(PhaseDataset::new(ProcessingPhase::NameResolution, resolution_data));
            
            // Phase 3: Type Inference
            let type_data = self.extract_type_inference_data(&rust_file)?;
            datasets.push(PhaseDataset::new(ProcessingPhase::TypeInference, type_data));
            
            // Continue for other phases...
        }
        
        Ok(datasets)
    }
}
```

#### Step 2: HF Dataset Generation
```rust
impl HfDatasetConverter {
    pub fn convert_rust_analyzer_datasets(
        &self, 
        phase_datasets: Vec<PhaseDataset>,
        output_dir: &Path
    ) -> Result<()> {
        // Group by phase
        let grouped = self.group_by_phase(phase_datasets);
        
        for (phase, datasets) in grouped {
            let hf_dataset_path = output_dir.join(format!("rust-analyzer-{:?}", phase));
            self.create_phase_dataset(datasets, &hf_dataset_path)?;
        }
        
        Ok(())
    }
}
```

## Dataset Schema Design

### Multi-Phase Dataset Structure
```
rust-analyzer-datasets/
├── parsing-phase/
│   ├── train/
│   │   └── data-00000-of-00001.parquet
│   ├── validation/
│   │   └── data-00000-of-00001.parquet
│   ├── test/
│   │   └── data-00000-of-00001.parquet
│   ├── README.md
│   ├── dataset_info.json
│   └── state.json
├── name-resolution-phase/
│   └── ... (same structure)
├── type-inference-phase/
│   └── ... (same structure)
└── combined-phases/
    └── ... (all phases in one dataset)
```

### Parquet Schema
```rust
pub struct RustAnalyzerRecord {
    // Identification
    pub id: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    
    // Phase information
    pub phase: String,
    pub processing_order: u32,
    
    // Element information
    pub element_type: String, // function, struct, variable, etc.
    pub element_name: Option<String>,
    pub element_signature: Option<String>,
    
    // Semantic data (JSON serialized)
    pub syntax_data: Option<String>,
    pub symbol_data: Option<String>,
    pub type_data: Option<String>,
    pub diagnostic_data: Option<String>,
    
    // Metadata
    pub processing_time_ms: u64,
    pub timestamp: u64,
    pub rust_version: String,
    pub analyzer_version: String,
    
    // Source context
    pub source_snippet: String,
    pub context_before: Option<String>,
    pub context_after: Option<String>,
}
```

## Implementation Steps

### Phase 1: Basic Integration (Week 1-2)
1. Create `rust_analyzer_extractor.rs` module
2. Implement basic parsing phase extraction
3. Extend HF dataset converter for rust-analyzer data
4. Create simple test with small Rust project

### Phase 2: Multi-Phase Support (Week 3-4)
1. Add name resolution phase extraction
2. Add type inference phase extraction
3. Implement phase-specific dataset generation
4. Add comprehensive testing

### Phase 3: Advanced Features (Week 5-6)
1. Add HIR generation phase
2. Add diagnostics and IDE features phases
3. Implement incremental processing
4. Add performance optimizations

### Phase 4: Production Ready (Week 7-8)
1. Add comprehensive error handling
2. Implement batch processing for large codebases
3. Add dataset validation and quality checks
4. Create documentation and examples

## Usage Examples

### Generate Datasets from Rust Project
```bash
# Process the rust-analyzer codebase itself
cargo run -- analyze-rust-project /home/mdupont/2025/06/27/rust-analyzer rust-analyzer-datasets

# Process the HF dataset validator
cargo run -- analyze-rust-project /home/mdupont/2025/08/07/hf-dataset-validator-rust hf-validator-datasets

# Process specific phases only
cargo run -- analyze-rust-project --phases parsing,type-inference /path/to/project output-dir
```

### Validate Generated Datasets
```bash
# Validate all phase datasets
cargo run -- validate-rust-analyzer-datasets rust-analyzer-datasets

# Load and explore datasets
cargo run -- explore-rust-analyzer-datasets rust-analyzer-datasets/parsing-phase
```

## Potential Applications

### 1. ML Training Data
- **Code completion models**: Train on parsing and name resolution data
- **Type inference models**: Learn from type inference patterns
- **Bug detection models**: Train on diagnostic data
- **Code understanding models**: Learn from semantic analysis

### 2. Research Applications
- **Compiler optimization**: Analyze compilation patterns
- **Language design**: Study how developers use language features
- **IDE improvement**: Understand common user interactions
- **Code quality metrics**: Develop better static analysis tools

### 3. Educational Tools
- **Rust learning**: Show how code is processed step-by-step
- **Compiler education**: Visualize compilation phases
- **Code analysis tutorials**: Interactive examples of semantic analysis

## Technical Considerations

### Performance
- **Streaming processing**: Handle large codebases efficiently
- **Parallel processing**: Process multiple files concurrently
- **Incremental updates**: Only reprocess changed files
- **Memory management**: Avoid loading entire codebases into memory

### Data Quality
- **Schema validation**: Ensure consistent data structure
- **Deduplication**: Remove duplicate analysis results
- **Error handling**: Gracefully handle analysis failures
- **Completeness checks**: Verify all phases completed successfully

### Scalability
- **Batch processing**: Process codebases in configurable batches
- **Distributed processing**: Support for cluster-based processing
- **Storage optimization**: Efficient Parquet compression
- **Query optimization**: Fast dataset access patterns

## Next Steps

1. **Create initial integration module** in the HF dataset validator project
2. **Implement basic parsing phase extraction** using rust-analyzer APIs
3. **Extend the existing HF dataset converter** to handle rust-analyzer data
4. **Test with small Rust project** to validate the approach
5. **Iterate and expand** to additional phases based on results

This integration will create a unique dataset that captures the semantic understanding process of Rust code, providing valuable training data for AI models focused on code understanding and generation.
