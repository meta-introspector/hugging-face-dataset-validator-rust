use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use arrow::array::{StringArray, UInt32Array, UInt64Array, BooleanArray};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::sync::Arc;

// Import rust-analyzer components (these would need to be added to Cargo.toml)
// use ra_ide::{Analysis, AnalysisHost, FileId, FilePosition};
// use ra_syntax::{SyntaxNode, ast, AstNode};
// use ra_hir::{Semantics, HirDatabase};

/// Represents different phases of rust-analyzer processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProcessingPhase {
    Parsing,
    NameResolution,
    TypeInference,
    HirGeneration,
    Diagnostics,
    Completions,
    Hover,
    GotoDefinition,
    FindReferences,
}

impl ProcessingPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessingPhase::Parsing => "parsing",
            ProcessingPhase::NameResolution => "name_resolution",
            ProcessingPhase::TypeInference => "type_inference", 
            ProcessingPhase::HirGeneration => "hir_generation",
            ProcessingPhase::Diagnostics => "diagnostics",
            ProcessingPhase::Completions => "completions",
            ProcessingPhase::Hover => "hover",
            ProcessingPhase::GotoDefinition => "goto_definition",
            ProcessingPhase::FindReferences => "find_references",
        }
    }
}

/// Data structure for rust-analyzer dataset records
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Phase-specific data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsingPhaseData {
    pub file_path: String,
    pub source_code: String,
    pub syntax_tree_json: String, // Serialized syntax tree
    pub tokens: Vec<TokenInfo>,
    pub parse_errors: Vec<ParseErrorInfo>,
    pub parse_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub kind: String,
    pub text: String,
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseErrorInfo {
    pub message: String,
    pub start: u32,
    pub end: u32,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameResolutionPhaseData {
    pub file_path: String,
    pub symbols: Vec<SymbolInfo>,
    pub scopes: Vec<ScopeInfo>,
    pub imports: Vec<ImportInfo>,
    pub unresolved_names: Vec<UnresolvedNameInfo>,
    pub resolution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String, // function, struct, variable, etc.
    pub definition_location: LocationInfo,
    pub visibility: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeInfo {
    pub scope_id: String,
    pub parent_scope: Option<String>,
    pub start: u32,
    pub end: u32,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub path: String,
    pub alias: Option<String>,
    pub location: LocationInfo,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedNameInfo {
    pub name: String,
    pub location: LocationInfo,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInferencePhaseData {
    pub file_path: String,
    pub type_assignments: Vec<TypeAssignmentInfo>,
    pub type_errors: Vec<TypeErrorInfo>,
    pub inferred_types: Vec<InferredTypeInfo>,
    pub inference_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAssignmentInfo {
    pub location: LocationInfo,
    pub expression: String,
    pub inferred_type: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeErrorInfo {
    pub message: String,
    pub location: LocationInfo,
    pub expected_type: Option<String>,
    pub actual_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredTypeInfo {
    pub symbol_name: String,
    pub location: LocationInfo,
    pub inferred_type: String,
    pub inference_method: String, // "explicit", "inferred", "default"
}

/// Main extractor for rust-analyzer data
pub struct RustAnalyzerExtractor {
    // analysis_host: AnalysisHost,
    rust_version: String,
    analyzer_version: String,
    processing_order: u32,
}

impl RustAnalyzerExtractor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            // analysis_host: AnalysisHost::new(),
            rust_version: Self::get_rust_version()?,
            analyzer_version: Self::get_analyzer_version()?,
            processing_order: 0,
        })
    }

    fn get_rust_version() -> Result<String> {
        // In a real implementation, this would get the actual Rust version
        Ok("1.86.0".to_string())
    }

    fn get_analyzer_version() -> Result<String> {
        // In a real implementation, this would get the actual rust-analyzer version
        Ok("0.3.2000".to_string())
    }

    /// Process a Rust codebase and generate Parquet files for HuggingFace dataset
    pub fn process_codebase_to_parquet(&mut self, codebase_path: &Path, phases: &[ProcessingPhase], output_dir: &Path) -> Result<()> {
        let rust_files = self.find_rust_files(codebase_path)?;
        println!("Found {} Rust files to process", rust_files.len());

        // Create output directory
        std::fs::create_dir_all(output_dir)?;

        // Process each phase separately
        for phase in phases {
            println!("Processing phase: {:?}", phase);
            let mut phase_records = Vec::new();

            for (file_index, rust_file) in rust_files.iter().enumerate() {
                if file_index % 100 == 0 {
                    println!("Processing file {}/{}: {}", file_index + 1, rust_files.len(), rust_file.display());
                }
                
                let file_records = self.extract_phase_data(rust_file, phase)?;
                phase_records.extend(file_records);
            }

            println!("Generated {} records for phase {:?}", phase_records.len(), phase);

            // Write to Parquet files (split if necessary)
            self.write_phase_to_parquet(&phase_records, phase, output_dir)?;
        }

        Ok(())
    }

    /// Write phase records to Parquet files, splitting if they exceed size limits
    fn write_phase_to_parquet(&self, records: &[RustAnalyzerRecord], phase: &ProcessingPhase, output_dir: &Path) -> Result<()> {
        const MAX_FILE_SIZE_MB: usize = 9; // Stay under 10MB for Git LFS
        const RECORDS_PER_BATCH: usize = 1000; // Process in batches to estimate size

        let phase_dir = output_dir.join(format!("{}-phase", phase.as_str()));
        std::fs::create_dir_all(&phase_dir)?;

        if records.is_empty() {
            println!("No records for phase {:?}, skipping", phase);
            return Ok();
        }

        // Estimate size per record by writing a small sample
        let sample_size = std::cmp::min(100, records.len());
        let sample_records = &records[0..sample_size];
        
        let temp_file = phase_dir.join("temp_sample.parquet");
        self.write_records_to_parquet(sample_records, &temp_file)?;
        
        let sample_size_bytes = std::fs::metadata(&temp_file)?.len();
        std::fs::remove_file(&temp_file)?;
        
        let bytes_per_record = sample_size_bytes as f64 / sample_size as f64;
        let max_records_per_file = ((MAX_FILE_SIZE_MB * 1024 * 1024) as f64 * 0.9 / bytes_per_record) as usize;
        
        println!("Estimated {} bytes per record, max {} records per file", bytes_per_record as usize, max_records_per_file);

        if records.len() <= max_records_per_file {
            // Single file
            let output_file = phase_dir.join("data.parquet");
            self.write_records_to_parquet(records, &output_file)?;
            
            let file_size_mb = std::fs::metadata(&output_file)?.len() as f64 / (1024.0 * 1024.0);
            println!("Created single file: {} ({:.2} MB)", output_file.display(), file_size_mb);
        } else {
            // Multiple files
            let num_files = (records.len() + max_records_per_file - 1) / max_records_per_file;
            
            for (file_idx, chunk) in records.chunks(max_records_per_file).enumerate() {
                let output_file = phase_dir.join(format!("data-{:05}-of-{:05}.parquet", file_idx, num_files));
                self.write_records_to_parquet(chunk, &output_file)?;
                
                let file_size_mb = std::fs::metadata(&output_file)?.len() as f64 / (1024.0 * 1024.0);
                println!("Created chunk {}/{}: {} ({:.2} MB, {} records)", 
                    file_idx + 1, num_files, output_file.display(), file_size_mb, chunk.len());
            }
        }

        Ok(())
    }

    /// Write records to a single Parquet file
    fn write_records_to_parquet(&self, records: &[RustAnalyzerRecord], output_file: &Path) -> Result<()> {
        use arrow::datatypes::{DataType, Field, Schema};

        // Define schema
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("line", DataType::UInt32, false),
            Field::new("column", DataType::UInt32, false),
            Field::new("phase", DataType::Utf8, false),
            Field::new("processing_order", DataType::UInt32, false),
            Field::new("element_type", DataType::Utf8, false),
            Field::new("element_name", DataType::Utf8, true),
            Field::new("element_signature", DataType::Utf8, true),
            Field::new("syntax_data", DataType::Utf8, true),
            Field::new("symbol_data", DataType::Utf8, true),
            Field::new("type_data", DataType::Utf8, true),
            Field::new("diagnostic_data", DataType::Utf8, true),
            Field::new("processing_time_ms", DataType::UInt64, false),
            Field::new("timestamp", DataType::UInt64, false),
            Field::new("rust_version", DataType::Utf8, false),
            Field::new("analyzer_version", DataType::Utf8, false),
            Field::new("source_snippet", DataType::Utf8, false),
            Field::new("context_before", DataType::Utf8, true),
            Field::new("context_after", DataType::Utf8, true),
        ]));

        // Convert records to Arrow arrays
        let ids: Vec<String> = records.iter().map(|r| r.id.clone()).collect();
        let file_paths: Vec<String> = records.iter().map(|r| r.file_path.clone()).collect();
        let lines: Vec<u32> = records.iter().map(|r| r.line).collect();
        let columns: Vec<u32> = records.iter().map(|r| r.column).collect();
        let phases: Vec<String> = records.iter().map(|r| r.phase.clone()).collect();
        let processing_orders: Vec<u32> = records.iter().map(|r| r.processing_order).collect();
        let element_types: Vec<String> = records.iter().map(|r| r.element_type.clone()).collect();
        let element_names: Vec<Option<String>> = records.iter().map(|r| r.element_name.clone()).collect();
        let element_signatures: Vec<Option<String>> = records.iter().map(|r| r.element_signature.clone()).collect();
        let syntax_data: Vec<Option<String>> = records.iter().map(|r| r.syntax_data.clone()).collect();
        let symbol_data: Vec<Option<String>> = records.iter().map(|r| r.symbol_data.clone()).collect();
        let type_data: Vec<Option<String>> = records.iter().map(|r| r.type_data.clone()).collect();
        let diagnostic_data: Vec<Option<String>> = records.iter().map(|r| r.diagnostic_data.clone()).collect();
        let processing_times: Vec<u64> = records.iter().map(|r| r.processing_time_ms).collect();
        let timestamps: Vec<u64> = records.iter().map(|r| r.timestamp).collect();
        let rust_versions: Vec<String> = records.iter().map(|r| r.rust_version.clone()).collect();
        let analyzer_versions: Vec<String> = records.iter().map(|r| r.analyzer_version.clone()).collect();
        let source_snippets: Vec<String> = records.iter().map(|r| r.source_snippet.clone()).collect();
        let context_befores: Vec<Option<String>> = records.iter().map(|r| r.context_before.clone()).collect();
        let context_afters: Vec<Option<String>> = records.iter().map(|r| r.context_after.clone()).collect();

        // Create Arrow arrays
        let id_array = Arc::new(StringArray::from(ids));
        let file_path_array = Arc::new(StringArray::from(file_paths));
        let line_array = Arc::new(UInt32Array::from(lines));
        let column_array = Arc::new(UInt32Array::from(columns));
        let phase_array = Arc::new(StringArray::from(phases));
        let processing_order_array = Arc::new(UInt32Array::from(processing_orders));
        let element_type_array = Arc::new(StringArray::from(element_types));
        let element_name_array = Arc::new(StringArray::from(element_names));
        let element_signature_array = Arc::new(StringArray::from(element_signatures));
        let syntax_data_array = Arc::new(StringArray::from(syntax_data));
        let symbol_data_array = Arc::new(StringArray::from(symbol_data));
        let type_data_array = Arc::new(StringArray::from(type_data));
        let diagnostic_data_array = Arc::new(StringArray::from(diagnostic_data));
        let processing_time_array = Arc::new(UInt64Array::from(processing_times));
        let timestamp_array = Arc::new(UInt64Array::from(timestamps));
        let rust_version_array = Arc::new(StringArray::from(rust_versions));
        let analyzer_version_array = Arc::new(StringArray::from(analyzer_versions));
        let source_snippet_array = Arc::new(StringArray::from(source_snippets));
        let context_before_array = Arc::new(StringArray::from(context_befores));
        let context_after_array = Arc::new(StringArray::from(context_afters));

        // Create record batch
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                id_array,
                file_path_array,
                line_array,
                column_array,
                phase_array,
                processing_order_array,
                element_type_array,
                element_name_array,
                element_signature_array,
                syntax_data_array,
                symbol_data_array,
                type_data_array,
                diagnostic_data_array,
                processing_time_array,
                timestamp_array,
                rust_version_array,
                analyzer_version_array,
                source_snippet_array,
                context_before_array,
                context_after_array,
            ],
        )?;

        // Write to Parquet file
        let file = std::fs::File::create(output_file)?;
        let props = WriterProperties::builder()
            .set_compression(parquet::basic::Compression::SNAPPY)
            .build();
        
        let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;
        writer.write(&batch)?;
        writer.close()?;

        Ok(())
    }

    /// Process a Rust codebase and extract data from all phases
    pub fn process_codebase(&mut self, codebase_path: &Path, phases: &[ProcessingPhase]) -> Result<Vec<RustAnalyzerRecord>> {
        let mut records = Vec::new();
        let rust_files = self.find_rust_files(codebase_path)?;

        println!("Found {} Rust files to process", rust_files.len());

        for (file_index, rust_file) in rust_files.iter().enumerate() {
            println!("Processing file {}/{}: {}", file_index + 1, rust_files.len(), rust_file.display());
            
            for phase in phases {
                let phase_records = self.extract_phase_data(rust_file, phase)?;
                records.extend(phase_records);
            }
        }

        println!("Generated {} total records", records.len());
        Ok(records)
    }

    /// Find all Rust files in a directory
    fn find_rust_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut rust_files = Vec::new();
        self.find_rust_files_recursive(dir, &mut rust_files)?;
        Ok(rust_files)
    }

    fn find_rust_files_recursive(&self, dir: &Path, rust_files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip target and hidden directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with('.') || dir_name == "target" {
                        continue;
                    }
                }
                self.find_rust_files_recursive(&path, rust_files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                rust_files.push(path);
            }
        }

        Ok(())
    }

    /// Extract data for a specific phase from a Rust file
    fn extract_phase_data(&mut self, file_path: &Path, phase: &ProcessingPhase) -> Result<Vec<RustAnalyzerRecord>> {
        let _start_time = Instant::now();
        
        match phase {
            ProcessingPhase::Parsing => self.extract_parsing_data(file_path),
            ProcessingPhase::NameResolution => self.extract_name_resolution_data(file_path),
            ProcessingPhase::TypeInference => self.extract_type_inference_data(file_path),
            ProcessingPhase::HirGeneration => self.extract_hir_data(file_path),
            ProcessingPhase::Diagnostics => self.extract_diagnostics_data(file_path),
            ProcessingPhase::Completions => self.extract_completions_data(file_path),
            ProcessingPhase::Hover => self.extract_hover_data(file_path),
            ProcessingPhase::GotoDefinition => self.extract_goto_definition_data(file_path),
            ProcessingPhase::FindReferences => self.extract_find_references_data(file_path),
        }
    }

    /// Extract parsing phase data (mock implementation)
    fn extract_parsing_data(&mut self, file_path: &Path) -> Result<Vec<RustAnalyzerRecord>> {
        let source_code = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        // Mock parsing data - in real implementation, this would use rust-analyzer's parser
        let mut records = Vec::new();
        let lines: Vec<&str> = source_code.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let record = RustAnalyzerRecord {
                id: format!("{}:{}:parsing", file_path.display(), line_num + 1),
                file_path: file_path.to_string_lossy().to_string(),
                line: (line_num + 1) as u32,
                column: 1,
                phase: ProcessingPhase::Parsing.as_str().to_string(),
                processing_order: self.next_processing_order(),
                element_type: self.detect_element_type(line),
                element_name: self.extract_element_name(line),
                element_signature: None,
                syntax_data: Some(self.create_mock_syntax_data(line)),
                symbol_data: None,
                type_data: None,
                diagnostic_data: None,
                processing_time_ms: 1, // Mock timing
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                rust_version: self.rust_version.clone(),
                analyzer_version: self.analyzer_version.clone(),
                source_snippet: line.to_string(),
                context_before: if line_num > 0 { Some(lines[line_num - 1].to_string()) } else { None },
                context_after: if line_num + 1 < lines.len() { Some(lines[line_num + 1].to_string()) } else { None },
            };

            records.push(record);
        }

        Ok(records)
    }

    /// Extract name resolution data (mock implementation)
    fn extract_name_resolution_data(&mut self, file_path: &Path) -> Result<Vec<RustAnalyzerRecord>> {
        let source_code = std::fs::read_to_string(file_path)?;
        let mut records = Vec::new();

        // Mock name resolution - find function definitions, struct definitions, etc.
        for (line_num, line) in source_code.lines().enumerate() {
            if line.contains("fn ") || line.contains("struct ") || line.contains("enum ") {
                let record = RustAnalyzerRecord {
                    id: format!("{}:{}:name_resolution", file_path.display(), line_num + 1),
                    file_path: file_path.to_string_lossy().to_string(),
                    line: (line_num + 1) as u32,
                    column: 1,
                    phase: ProcessingPhase::NameResolution.as_str().to_string(),
                    processing_order: self.next_processing_order(),
                    element_type: self.detect_element_type(line),
                    element_name: self.extract_element_name(line),
                    element_signature: Some(line.trim().to_string()),
                    syntax_data: None,
                    symbol_data: Some(self.create_mock_symbol_data(line)),
                    type_data: None,
                    diagnostic_data: None,
                    processing_time_ms: 2,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                    rust_version: self.rust_version.clone(),
                    analyzer_version: self.analyzer_version.clone(),
                    source_snippet: line.to_string(),
                    context_before: None,
                    context_after: None,
                };

                records.push(record);
            }
        }

        Ok(records)
    }

    /// Extract type inference data (mock implementation)
    fn extract_type_inference_data(&mut self, file_path: &Path) -> Result<Vec<RustAnalyzerRecord>> {
        let source_code = std::fs::read_to_string(file_path)?;
        let mut records = Vec::new();

        // Mock type inference - find variable declarations, function returns, etc.
        for (line_num, line) in source_code.lines().enumerate() {
            if line.contains("let ") || line.contains("-> ") {
                let record = RustAnalyzerRecord {
                    id: format!("{}:{}:type_inference", file_path.display(), line_num + 1),
                    file_path: file_path.to_string_lossy().to_string(),
                    line: (line_num + 1) as u32,
                    column: 1,
                    phase: ProcessingPhase::TypeInference.as_str().to_string(),
                    processing_order: self.next_processing_order(),
                    element_type: "variable_or_return".to_string(),
                    element_name: self.extract_variable_name(line),
                    element_signature: None,
                    syntax_data: None,
                    symbol_data: None,
                    type_data: Some(self.create_mock_type_data(line)),
                    diagnostic_data: None,
                    processing_time_ms: 3,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                    rust_version: self.rust_version.clone(),
                    analyzer_version: self.analyzer_version.clone(),
                    source_snippet: line.to_string(),
                    context_before: None,
                    context_after: None,
                };

                records.push(record);
            }
        }

        Ok(records)
    }

    // Placeholder implementations for other phases
    fn extract_hir_data(&mut self, _file_path: &Path) -> Result<Vec<RustAnalyzerRecord>> {
        Ok(Vec::new()) // TODO: Implement HIR extraction
    }

    fn extract_diagnostics_data(&mut self, _file_path: &Path) -> Result<Vec<RustAnalyzerRecord>> {
        Ok(Vec::new()) // TODO: Implement diagnostics extraction
    }

    fn extract_completions_data(&mut self, _file_path: &Path) -> Result<Vec<RustAnalyzerRecord>> {
        Ok(Vec::new()) // TODO: Implement completions extraction
    }

    fn extract_hover_data(&mut self, _file_path: &Path) -> Result<Vec<RustAnalyzerRecord>> {
        Ok(Vec::new()) // TODO: Implement hover extraction
    }

    fn extract_goto_definition_data(&mut self, _file_path: &Path) -> Result<Vec<RustAnalyzerRecord>> {
        Ok(Vec::new()) // TODO: Implement goto definition extraction
    }

    fn extract_find_references_data(&mut self, _file_path: &Path) -> Result<Vec<RustAnalyzerRecord>> {
        Ok(Vec::new()) // TODO: Implement find references extraction
    }

    // Helper methods
    fn next_processing_order(&mut self) -> u32 {
        self.processing_order += 1;
        self.processing_order
    }

    fn detect_element_type(&self, line: &str) -> String {
        if line.contains("fn ") {
            "function".to_string()
        } else if line.contains("struct ") {
            "struct".to_string()
        } else if line.contains("enum ") {
            "enum".to_string()
        } else if line.contains("impl ") {
            "impl".to_string()
        } else if line.contains("let ") {
            "variable".to_string()
        } else if line.contains("use ") {
            "import".to_string()
        } else {
            "other".to_string()
        }
    }

    fn extract_element_name(&self, line: &str) -> Option<String> {
        // Simple regex-like extraction (in real implementation, use proper parsing)
        if let Some(fn_pos) = line.find("fn ") {
            let after_fn = &line[fn_pos + 3..];
            if let Some(paren_pos) = after_fn.find('(') {
                return Some(after_fn[..paren_pos].trim().to_string());
            }
        }
        
        if let Some(struct_pos) = line.find("struct ") {
            let after_struct = &line[struct_pos + 7..];
            if let Some(space_pos) = after_struct.find(' ') {
                return Some(after_struct[..space_pos].trim().to_string());
            }
        }

        None
    }

    fn extract_variable_name(&self, line: &str) -> Option<String> {
        if let Some(let_pos) = line.find("let ") {
            let after_let = &line[let_pos + 4..];
            if let Some(eq_pos) = after_let.find('=') {
                return Some(after_let[..eq_pos].trim().to_string());
            }
            if let Some(colon_pos) = after_let.find(':') {
                return Some(after_let[..colon_pos].trim().to_string());
            }
        }
        None
    }

    fn create_mock_syntax_data(&self, line: &str) -> String {
        serde_json::json!({
            "tokens": [
                {
                    "kind": "keyword",
                    "text": line.split_whitespace().next().unwrap_or(""),
                    "start": 0,
                    "end": line.len()
                }
            ],
            "ast_node_type": self.detect_element_type(line)
        }).to_string()
    }

    fn create_mock_symbol_data(&self, line: &str) -> String {
        serde_json::json!({
            "symbol_kind": self.detect_element_type(line),
            "visibility": "public",
            "definition_location": {
                "line": 1,
                "column": 1
            }
        }).to_string()
    }

    fn create_mock_type_data(&self, line: &str) -> String {
        let inferred_type = if line.contains("String") {
            "String"
        } else if line.contains("i32") {
            "i32"
        } else if line.contains("bool") {
            "bool"
        } else {
            "unknown"
        };

        serde_json::json!({
            "inferred_type": inferred_type,
            "confidence": 0.95,
            "inference_method": "explicit"
        }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_rust_analyzer_extractor_creation() {
        let extractor = RustAnalyzerExtractor::new();
        assert!(extractor.is_ok());
    }

    #[test]
    fn test_find_rust_files() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        fs::write(&rust_file, "fn main() {}").unwrap();

        let extractor = RustAnalyzerExtractor::new().unwrap();
        let rust_files = extractor.find_rust_files(temp_dir.path()).unwrap();
        
        assert_eq!(rust_files.len(), 1);
        assert_eq!(rust_files[0], rust_file);
    }

    #[test]
    fn test_extract_parsing_data() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        fs::write(&rust_file, "fn main() {\n    println!(\"Hello, world!\");\n}").unwrap();

        let mut extractor = RustAnalyzerExtractor::new().unwrap();
        let records = extractor.extract_parsing_data(&rust_file).unwrap();
        
        assert!(!records.is_empty());
        assert_eq!(records[0].phase, "parsing");
        assert_eq!(records[0].element_type, "function");
    }

    #[test]
    fn test_element_type_detection() {
        let extractor = RustAnalyzerExtractor::new().unwrap();
        
        assert_eq!(extractor.detect_element_type("fn main() {"), "function");
        assert_eq!(extractor.detect_element_type("struct Point {"), "struct");
        assert_eq!(extractor.detect_element_type("enum Color {"), "enum");
        assert_eq!(extractor.detect_element_type("let x = 5;"), "variable");
    }
}
