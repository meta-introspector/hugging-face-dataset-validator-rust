//! # Cargo2HF: Cargo Project to HuggingFace Dataset Extractor
//! 
//! This module provides comprehensive extraction of Cargo project data and dependencies
//! to create rich datasets for machine learning applications. It augments the rust-analyzer
//! semantic analysis with project structure, dependency graphs, and ecosystem metadata.
//! 
//! ## Key Features
//! 
//! - **Project Metadata**: Extract Cargo.toml information, authors, licenses, descriptions
//! - **Dependency Analysis**: Build comprehensive dependency graphs with version constraints
//! - **Source Code Metrics**: Lines of code, complexity metrics, documentation coverage
//! - **Build Configuration**: Features, targets, build scripts, and conditional compilation
//! - **Ecosystem Integration**: Crates.io metadata, download statistics, popularity metrics
//! - **Version History**: Git history analysis, release patterns, maintenance activity
//! 
//! ## Dataset Schema
//! 
//! The extractor generates multiple related datasets:
//! 
//! ### 1. Project Records
//! - Basic project information (name, version, description, authors)
//! - Repository metadata (URL, stars, forks, issues)
//! - License and legal information
//! - Documentation and README analysis
//! 
//! ### 2. Dependency Records  
//! - Direct and transitive dependencies
//! - Version constraints and resolution
//! - Feature flags and optional dependencies
//! - Dependency update patterns and compatibility
//! 
//! ### 3. Source Code Records
//! - File-level metrics (size, complexity, documentation)
//! - Module structure and organization
//! - Public API surface analysis
//! - Code quality indicators
//! 
//! ### 4. Build Records
//! - Build script analysis and custom build logic
//! - Target platform configurations
//! - Feature flag usage and combinations
//! - Compilation profiles and optimizations
//! 
//! ## Integration with Rust-Analyzer
//! 
//! This tool is designed to complement the rust-analyzer semantic analysis by providing:
//! - **Project Context**: Understanding how code fits into larger projects
//! - **Ecosystem Patterns**: Learning from real-world Rust project structures
//! - **Dependency Intelligence**: Understanding how libraries are used together
//! - **Evolution Tracking**: How projects and their dependencies change over time

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use arrow::array::{StringArray, UInt32Array, UInt64Array, Float32Array, BooleanArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::sync::Arc;

/// Represents different types of data extraction phases for Cargo projects
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CargoExtractionPhase {
    /// Extract basic project metadata from Cargo.toml
    ProjectMetadata,
    /// Analyze dependency graph and constraints
    DependencyAnalysis,
    /// Extract source code metrics and structure
    SourceCodeAnalysis,
    /// Analyze build configuration and scripts
    BuildAnalysis,
    /// Extract ecosystem and crates.io metadata
    EcosystemAnalysis,
    /// Analyze git history and development patterns
    VersionHistory,
}

impl CargoExtractionPhase {
    /// Convert phase to string representation for dataset naming
    pub fn as_str(&self) -> &'static str {
        match self {
            CargoExtractionPhase::ProjectMetadata => "project_metadata",
            CargoExtractionPhase::DependencyAnalysis => "dependency_analysis", 
            CargoExtractionPhase::SourceCodeAnalysis => "source_code_analysis",
            CargoExtractionPhase::BuildAnalysis => "build_analysis",
            CargoExtractionPhase::EcosystemAnalysis => "ecosystem_analysis",
            CargoExtractionPhase::VersionHistory => "version_history",
        }
    }
}

/// Main record structure for Cargo project analysis data
/// 
/// This structure captures comprehensive information about Cargo projects
/// and their dependencies, designed for machine learning applications
/// focused on understanding Rust project patterns and ecosystem dynamics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoProjectRecord {
    // === Identification ===
    /// Unique identifier for this record
    pub id: String,
    /// Path to the project root (containing Cargo.toml)
    pub project_path: String,
    /// Name of the project from Cargo.toml
    pub project_name: String,
    /// Project version
    pub project_version: String,
    /// Extraction phase that generated this record
    pub phase: String,
    /// Processing order for reproducible dataset generation
    pub processing_order: u32,
    
    // === Project Metadata ===
    /// Project description from Cargo.toml
    pub description: Option<String>,
    /// Project authors
    pub authors: Option<String>, // JSON array as string
    /// License identifier (e.g., "MIT", "Apache-2.0")
    pub license: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Documentation URL
    pub documentation: Option<String>,
    /// Keywords from Cargo.toml
    pub keywords: Option<String>, // JSON array as string
    /// Categories from Cargo.toml
    pub categories: Option<String>, // JSON array as string
    
    // === Source Code Metrics ===
    /// Total lines of Rust code in the project
    pub lines_of_code: u32,
    /// Number of Rust source files
    pub source_file_count: u32,
    /// Number of test files
    pub test_file_count: u32,
    /// Number of example files
    pub example_file_count: u32,
    /// Number of benchmark files
    pub benchmark_file_count: u32,
    /// Estimated code complexity score
    pub complexity_score: f32,
    /// Documentation coverage percentage
    pub documentation_coverage: f32,
    
    // === Dependency Information ===
    /// Number of direct dependencies
    pub direct_dependencies: u32,
    /// Number of transitive dependencies (total in dependency tree)
    pub total_dependencies: u32,
    /// Number of dev dependencies
    pub dev_dependencies: u32,
    /// Number of build dependencies
    pub build_dependencies: u32,
    /// Dependency data as JSON
    pub dependency_data: Option<String>,
    
    // === Build Configuration ===
    /// Available feature flags
    pub features: Option<String>, // JSON object as string
    /// Target platforms/architectures
    pub targets: Option<String>, // JSON array as string
    /// Has custom build script (build.rs)
    pub has_build_script: bool,
    /// Build script complexity (lines of code in build.rs)
    pub build_script_complexity: u32,
    
    // === Ecosystem Metadata ===
    /// Crates.io download count (if available)
    pub download_count: Option<u64>,
    /// GitHub stars (if repository is on GitHub)
    pub github_stars: Option<u32>,
    /// GitHub forks
    pub github_forks: Option<u32>,
    /// GitHub issues count
    pub github_issues: Option<u32>,
    /// Last update timestamp from repository
    pub last_updated: Option<u64>,
    
    // === Version History ===
    /// Number of git commits in the project
    pub commit_count: Option<u32>,
    /// Number of contributors
    pub contributor_count: Option<u32>,
    /// Age of the project in days
    pub project_age_days: Option<u32>,
    /// Release frequency (releases per year)
    pub release_frequency: Option<f32>,
    
    // === Processing Metadata ===
    /// Time taken to process this record (milliseconds)
    pub processing_time_ms: u64,
    /// Unix timestamp when this record was created
    pub timestamp: u64,
    /// Version of cargo2hf tool used
    pub extractor_version: String,
    /// Version of Cargo used for analysis
    pub cargo_version: String,
    /// Version of Rust toolchain
    pub rust_version: String,
}

/// Detailed dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Name of the dependency
    pub name: String,
    /// Version requirement (e.g., "^1.0", "=0.2.5")
    pub version_req: String,
    /// Resolved version (if available)
    pub resolved_version: Option<String>,
    /// Whether this is an optional dependency
    pub optional: bool,
    /// Default features enabled
    pub default_features: bool,
    /// Specific features enabled
    pub features: Vec<String>,
    /// Dependency source (crates.io, git, path, etc.)
    pub source: String,
    /// Whether this is a dev dependency
    pub is_dev: bool,
    /// Whether this is a build dependency
    pub is_build: bool,
}

/// Source code file analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFileInfo {
    /// Relative path from project root
    pub path: String,
    /// File type (lib, bin, test, example, bench)
    pub file_type: String,
    /// Lines of code in this file
    pub lines_of_code: u32,
    /// Number of functions defined
    pub function_count: u32,
    /// Number of structs defined
    pub struct_count: u32,
    /// Number of enums defined
    pub enum_count: u32,
    /// Number of traits defined
    pub trait_count: u32,
    /// Number of impl blocks
    pub impl_count: u32,
    /// Number of public items
    pub public_items: u32,
    /// Estimated complexity score for this file
    pub complexity_score: f32,
    /// Documentation coverage for this file
    pub doc_coverage: f32,
}

/// Main extractor for Cargo project data
/// 
/// This extractor analyzes Cargo projects comprehensively, extracting metadata,
/// dependencies, source code metrics, and ecosystem information to create
/// rich datasets for machine learning applications.
pub struct Cargo2HfExtractor {
    /// Version of the extractor tool
    extractor_version: String,
    /// Version of Cargo being used
    cargo_version: String,
    /// Version of Rust toolchain
    rust_version: String,
    /// Processing order counter
    processing_order: u32,
}

impl Cargo2HfExtractor {
    /// Create a new Cargo2HF extractor instance
    /// 
    /// Initializes the extractor with current tool versions and processing state.
    /// This will query the system for Cargo and Rust versions to include in
    /// the generated dataset metadata.
    pub fn new() -> Result<Self> {
        Ok(Self {
            extractor_version: env!("CARGO_PKG_VERSION").to_string(),
            cargo_version: Self::get_cargo_version()?,
            rust_version: Self::get_rust_version()?,
            processing_order: 0,
        })
    }
    
    /// Get the current Cargo version
    fn get_cargo_version() -> Result<String> {
        // TODO: Execute `cargo --version` and parse output
        Ok("1.86.0".to_string())
    }
    
    /// Get the current Rust version
    fn get_rust_version() -> Result<String> {
        // TODO: Execute `rustc --version` and parse output
        Ok("1.86.0".to_string())
    }
    
    /// Process a Cargo project and generate HuggingFace dataset
    /// 
    /// This is the main entry point for extracting comprehensive data from
    /// a Cargo project. It analyzes the project through multiple phases and
    /// generates Parquet files suitable for machine learning applications.
    /// 
    /// # Arguments
    /// 
    /// * `project_path` - Path to the Cargo project root (containing Cargo.toml)
    /// * `phases` - List of extraction phases to run
    /// * `output_dir` - Directory where Parquet files will be written
    /// * `include_dependencies` - Whether to recursively analyze dependencies
    /// 
    /// # Phases
    /// 
    /// - **ProjectMetadata**: Basic project information from Cargo.toml
    /// - **DependencyAnalysis**: Dependency graph and version constraints
    /// - **SourceCodeAnalysis**: Code metrics and structure analysis
    /// - **BuildAnalysis**: Build scripts and configuration
    /// - **EcosystemAnalysis**: Crates.io and GitHub metadata
    /// - **VersionHistory**: Git history and development patterns
    pub fn extract_project_to_parquet(
        &mut self,
        project_path: &Path,
        phases: &[CargoExtractionPhase],
        output_dir: &Path,
        include_dependencies: bool,
    ) -> Result<()> {
        println!("Analyzing Cargo project: {}", project_path.display());
        
        // Verify this is a Cargo project
        let cargo_toml = project_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(anyhow::anyhow!("No Cargo.toml found in {}", project_path.display()));
        }
        
        // Create output directory
        std::fs::create_dir_all(output_dir)?;
        
        // Process each phase
        for phase in phases {
            println!("Processing phase: {:?}", phase);
            let phase_records = self.extract_phase_data(project_path, phase, include_dependencies)?;
            println!("Generated {} records for phase {:?}", phase_records.len(), phase);
            
            // Write to Parquet files
            self.write_phase_to_parquet(&phase_records, phase, output_dir)?;
        }
        
        Ok(())
    }
    
    /// Extract data for a specific extraction phase
    fn extract_phase_data(
        &mut self,
        project_path: &Path,
        phase: &CargoExtractionPhase,
        include_dependencies: bool,
    ) -> Result<Vec<CargoProjectRecord>> {
        match phase {
            CargoExtractionPhase::ProjectMetadata => {
                self.extract_project_metadata(project_path)
            }
            CargoExtractionPhase::DependencyAnalysis => {
                self.extract_dependency_analysis(project_path, include_dependencies)
            }
            CargoExtractionPhase::SourceCodeAnalysis => {
                self.extract_source_code_analysis(project_path)
            }
            CargoExtractionPhase::BuildAnalysis => {
                self.extract_build_analysis(project_path)
            }
            CargoExtractionPhase::EcosystemAnalysis => {
                self.extract_ecosystem_analysis(project_path)
            }
            CargoExtractionPhase::VersionHistory => {
                self.extract_version_history(project_path)
            }
        }
    }
    
    /// Extract basic project metadata from Cargo.toml
    /// 
    /// This phase analyzes the Cargo.toml file to extract fundamental project
    /// information including name, version, description, authors, license,
    /// and other metadata fields that describe the project.
    /// 
    /// Handles both regular packages and workspace configurations.
    fn extract_project_metadata(&mut self, project_path: &Path) -> Result<Vec<CargoProjectRecord>> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read Cargo.toml: {}", cargo_toml_path.display()))?;
        
        // Parse Cargo.toml
        let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)
            .with_context(|| "Failed to parse Cargo.toml")?;
        
        // Check if this is a workspace or a package
        if let Some(workspace) = cargo_toml.get("workspace") {
            // Handle workspace Cargo.toml
            self.extract_workspace_metadata(project_path, workspace)
        } else if let Some(package) = cargo_toml.get("package") {
            // Handle regular package Cargo.toml
            self.extract_package_metadata(project_path, package)
        } else {
            Err(anyhow::anyhow!("No [package] or [workspace] section in Cargo.toml"))
        }
    }
    
    /// Extract metadata from a workspace Cargo.toml
    fn extract_workspace_metadata(&mut self, project_path: &Path, workspace: &toml::Value) -> Result<Vec<CargoProjectRecord>> {
        // For workspace, we'll create a record representing the workspace itself
        let project_name = project_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown-workspace")
            .to_string();
        
        // Extract workspace members
        let members = workspace.get("members")
            .and_then(|v| v.as_array())
            .map(|arr| serde_json::to_string(arr).unwrap_or_default());
        
        let record = CargoProjectRecord {
            id: format!("{}:workspace:project_metadata", project_name),
            project_path: project_path.to_string_lossy().to_string(),
            project_name: project_name.clone(),
            project_version: "workspace".to_string(),
            phase: CargoExtractionPhase::ProjectMetadata.as_str().to_string(),
            processing_order: self.next_processing_order(),
            
            // Workspace-specific metadata
            description: Some(format!("Cargo workspace with {} members", 
                workspace.get("members")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0))),
            authors: None, // Workspaces typically don't have authors
            license: None, // Workspaces typically don't have licenses
            repository: None,
            homepage: None,
            documentation: None,
            keywords: members.clone(), // Store members in keywords field for now
            categories: None,
            
            // Initialize other fields with defaults
            lines_of_code: 0,
            source_file_count: 0,
            test_file_count: 0,
            example_file_count: 0,
            benchmark_file_count: 0,
            complexity_score: 0.0,
            documentation_coverage: 0.0,
            direct_dependencies: 0,
            total_dependencies: 0,
            dev_dependencies: 0,
            build_dependencies: 0,
            dependency_data: members, // Store workspace members as dependency data
            features: None,
            targets: None,
            has_build_script: project_path.join("build.rs").exists(),
            build_script_complexity: 0,
            download_count: None,
            github_stars: None,
            github_forks: None,
            github_issues: None,
            last_updated: None,
            commit_count: None,
            contributor_count: None,
            project_age_days: None,
            release_frequency: None,
            processing_time_ms: 1, // Mock timing
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            extractor_version: self.extractor_version.clone(),
            cargo_version: self.cargo_version.clone(),
            rust_version: self.rust_version.clone(),
        };
        
        Ok(vec![record])
    }
    
    /// Extract metadata from a regular package Cargo.toml
    fn extract_package_metadata(&mut self, project_path: &Path, package: &toml::Value) -> Result<Vec<CargoProjectRecord>> {
        // Extract basic metadata
        let project_name = package.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No package name in Cargo.toml"))?
            .to_string();
        
        let project_version = package.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();
        
        let record = CargoProjectRecord {
            id: format!("{}:{}:project_metadata", project_name, project_version),
            project_path: project_path.to_string_lossy().to_string(),
            project_name,
            project_version,
            phase: CargoExtractionPhase::ProjectMetadata.as_str().to_string(),
            processing_order: self.next_processing_order(),
            
            // Extract optional metadata fields
            description: package.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            authors: package.get("authors")
                .and_then(|v| v.as_array())
                .map(|arr| serde_json::to_string(arr).unwrap_or_default()),
            license: package.get("license").and_then(|v| v.as_str()).map(|s| s.to_string()),
            repository: package.get("repository").and_then(|v| v.as_str()).map(|s| s.to_string()),
            homepage: package.get("homepage").and_then(|v| v.as_str()).map(|s| s.to_string()),
            documentation: package.get("documentation").and_then(|v| v.as_str()).map(|s| s.to_string()),
            keywords: package.get("keywords")
                .and_then(|v| v.as_array())
                .map(|arr| serde_json::to_string(arr).unwrap_or_default()),
            categories: package.get("categories")
                .and_then(|v| v.as_array())
                .map(|arr| serde_json::to_string(arr).unwrap_or_default()),
            
            // Initialize other fields with defaults (will be filled in other phases)
            lines_of_code: 0,
            source_file_count: 0,
            test_file_count: 0,
            example_file_count: 0,
            benchmark_file_count: 0,
            complexity_score: 0.0,
            documentation_coverage: 0.0,
            direct_dependencies: 0,
            total_dependencies: 0,
            dev_dependencies: 0,
            build_dependencies: 0,
            dependency_data: None,
            features: None,
            targets: None,
            has_build_script: project_path.join("build.rs").exists(),
            build_script_complexity: 0,
            download_count: None,
            github_stars: None,
            github_forks: None,
            github_issues: None,
            last_updated: None,
            commit_count: None,
            contributor_count: None,
            project_age_days: None,
            release_frequency: None,
            processing_time_ms: 1, // Mock timing
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            extractor_version: self.extractor_version.clone(),
            cargo_version: self.cargo_version.clone(),
            rust_version: self.rust_version.clone(),
        };
        
        Ok(vec![record])
    }
    
    /// Placeholder implementations for other phases
    /// TODO: Implement comprehensive dependency analysis
    fn extract_dependency_analysis(&mut self, _project_path: &Path, _include_deps: bool) -> Result<Vec<CargoProjectRecord>> {
        Ok(Vec::new())
    }
    
    /// TODO: Implement source code analysis with metrics
    fn extract_source_code_analysis(&mut self, _project_path: &Path) -> Result<Vec<CargoProjectRecord>> {
        Ok(Vec::new())
    }
    
    /// TODO: Implement build configuration analysis
    fn extract_build_analysis(&mut self, _project_path: &Path) -> Result<Vec<CargoProjectRecord>> {
        Ok(Vec::new())
    }
    
    /// TODO: Implement ecosystem metadata extraction
    fn extract_ecosystem_analysis(&mut self, _project_path: &Path) -> Result<Vec<CargoProjectRecord>> {
        Ok(Vec::new())
    }
    
    /// TODO: Implement version history analysis
    fn extract_version_history(&mut self, _project_path: &Path) -> Result<Vec<CargoProjectRecord>> {
        Ok(Vec::new())
    }
    
    /// Generate next processing order number
    fn next_processing_order(&mut self) -> u32 {
        self.processing_order += 1;
        self.processing_order
    }
    
    /// Write phase records to Parquet files with automatic splitting
    fn write_phase_to_parquet(
        &self,
        records: &[CargoProjectRecord],
        phase: &CargoExtractionPhase,
        output_dir: &Path,
    ) -> Result<()> {
        const MAX_FILE_SIZE_MB: usize = 9;
        
        let phase_dir = output_dir.join(format!("{}-phase", phase.as_str()));
        std::fs::create_dir_all(&phase_dir)?;
        
        if records.is_empty() {
            println!("No records for phase {:?}, skipping", phase);
            return Ok(());
        }
        
        // For now, write single file (TODO: implement splitting like rust-analyzer extractor)
        let output_file = phase_dir.join("data.parquet");
        self.write_records_to_parquet(records, &output_file)?;
        
        let file_size_mb = std::fs::metadata(&output_file)?.len() as f64 / (1024.0 * 1024.0);
        println!("Created file: {} ({:.2} MB, {} records)", 
            output_file.display(), file_size_mb, records.len());
        
        Ok(())
    }
    
    /// Write records to a single Parquet file
    fn write_records_to_parquet(&self, records: &[CargoProjectRecord], output_file: &Path) -> Result<()> {
        // Define Arrow schema for Cargo project records
        let schema = Arc::new(Schema::new(vec![
            // Identification fields
            Field::new("id", DataType::Utf8, false),
            Field::new("project_path", DataType::Utf8, false),
            Field::new("project_name", DataType::Utf8, false),
            Field::new("project_version", DataType::Utf8, false),
            Field::new("phase", DataType::Utf8, false),
            Field::new("processing_order", DataType::UInt32, false),
            
            // Project metadata
            Field::new("description", DataType::Utf8, true),
            Field::new("authors", DataType::Utf8, true),
            Field::new("license", DataType::Utf8, true),
            Field::new("repository", DataType::Utf8, true),
            Field::new("homepage", DataType::Utf8, true),
            Field::new("documentation", DataType::Utf8, true),
            Field::new("keywords", DataType::Utf8, true),
            Field::new("categories", DataType::Utf8, true),
            
            // Source code metrics
            Field::new("lines_of_code", DataType::UInt32, false),
            Field::new("source_file_count", DataType::UInt32, false),
            Field::new("test_file_count", DataType::UInt32, false),
            Field::new("example_file_count", DataType::UInt32, false),
            Field::new("benchmark_file_count", DataType::UInt32, false),
            Field::new("complexity_score", DataType::Float32, false),
            Field::new("documentation_coverage", DataType::Float32, false),
            
            // Dependency information
            Field::new("direct_dependencies", DataType::UInt32, false),
            Field::new("total_dependencies", DataType::UInt32, false),
            Field::new("dev_dependencies", DataType::UInt32, false),
            Field::new("build_dependencies", DataType::UInt32, false),
            Field::new("dependency_data", DataType::Utf8, true),
            
            // Build configuration
            Field::new("features", DataType::Utf8, true),
            Field::new("targets", DataType::Utf8, true),
            Field::new("has_build_script", DataType::Boolean, false),
            Field::new("build_script_complexity", DataType::UInt32, false),
            
            // Ecosystem metadata
            Field::new("download_count", DataType::UInt64, true),
            Field::new("github_stars", DataType::UInt32, true),
            Field::new("github_forks", DataType::UInt32, true),
            Field::new("github_issues", DataType::UInt32, true),
            Field::new("last_updated", DataType::UInt64, true),
            
            // Version history
            Field::new("commit_count", DataType::UInt32, true),
            Field::new("contributor_count", DataType::UInt32, true),
            Field::new("project_age_days", DataType::UInt32, true),
            Field::new("release_frequency", DataType::Float32, true),
            
            // Processing metadata
            Field::new("processing_time_ms", DataType::UInt64, false),
            Field::new("timestamp", DataType::UInt64, false),
            Field::new("extractor_version", DataType::Utf8, false),
            Field::new("cargo_version", DataType::Utf8, false),
            Field::new("rust_version", DataType::Utf8, false),
        ]));
        
        // Convert records to Arrow arrays (similar to rust-analyzer extractor)
        let ids: Vec<String> = records.iter().map(|r| r.id.clone()).collect();
        let project_paths: Vec<String> = records.iter().map(|r| r.project_path.clone()).collect();
        let project_names: Vec<String> = records.iter().map(|r| r.project_name.clone()).collect();
        let project_versions: Vec<String> = records.iter().map(|r| r.project_version.clone()).collect();
        let phases: Vec<String> = records.iter().map(|r| r.phase.clone()).collect();
        let processing_orders: Vec<u32> = records.iter().map(|r| r.processing_order).collect();
        
        let descriptions: Vec<Option<String>> = records.iter().map(|r| r.description.clone()).collect();
        let authors: Vec<Option<String>> = records.iter().map(|r| r.authors.clone()).collect();
        let licenses: Vec<Option<String>> = records.iter().map(|r| r.license.clone()).collect();
        let repositories: Vec<Option<String>> = records.iter().map(|r| r.repository.clone()).collect();
        let homepages: Vec<Option<String>> = records.iter().map(|r| r.homepage.clone()).collect();
        let documentations: Vec<Option<String>> = records.iter().map(|r| r.documentation.clone()).collect();
        let keywords: Vec<Option<String>> = records.iter().map(|r| r.keywords.clone()).collect();
        let categories: Vec<Option<String>> = records.iter().map(|r| r.categories.clone()).collect();
        
        let lines_of_code: Vec<u32> = records.iter().map(|r| r.lines_of_code).collect();
        let source_file_counts: Vec<u32> = records.iter().map(|r| r.source_file_count).collect();
        let test_file_counts: Vec<u32> = records.iter().map(|r| r.test_file_count).collect();
        let example_file_counts: Vec<u32> = records.iter().map(|r| r.example_file_count).collect();
        let benchmark_file_counts: Vec<u32> = records.iter().map(|r| r.benchmark_file_count).collect();
        let complexity_scores: Vec<f32> = records.iter().map(|r| r.complexity_score).collect();
        let documentation_coverages: Vec<f32> = records.iter().map(|r| r.documentation_coverage).collect();
        
        let direct_dependencies: Vec<u32> = records.iter().map(|r| r.direct_dependencies).collect();
        let total_dependencies: Vec<u32> = records.iter().map(|r| r.total_dependencies).collect();
        let dev_dependencies: Vec<u32> = records.iter().map(|r| r.dev_dependencies).collect();
        let build_dependencies: Vec<u32> = records.iter().map(|r| r.build_dependencies).collect();
        let dependency_data: Vec<Option<String>> = records.iter().map(|r| r.dependency_data.clone()).collect();
        
        let features: Vec<Option<String>> = records.iter().map(|r| r.features.clone()).collect();
        let targets: Vec<Option<String>> = records.iter().map(|r| r.targets.clone()).collect();
        let has_build_scripts: Vec<bool> = records.iter().map(|r| r.has_build_script).collect();
        let build_script_complexities: Vec<u32> = records.iter().map(|r| r.build_script_complexity).collect();
        
        let download_counts: Vec<Option<u64>> = records.iter().map(|r| r.download_count).collect();
        let github_stars: Vec<Option<u32>> = records.iter().map(|r| r.github_stars).collect();
        let github_forks: Vec<Option<u32>> = records.iter().map(|r| r.github_forks).collect();
        let github_issues: Vec<Option<u32>> = records.iter().map(|r| r.github_issues).collect();
        let last_updateds: Vec<Option<u64>> = records.iter().map(|r| r.last_updated).collect();
        
        let commit_counts: Vec<Option<u32>> = records.iter().map(|r| r.commit_count).collect();
        let contributor_counts: Vec<Option<u32>> = records.iter().map(|r| r.contributor_count).collect();
        let project_age_days: Vec<Option<u32>> = records.iter().map(|r| r.project_age_days).collect();
        let release_frequencies: Vec<Option<f32>> = records.iter().map(|r| r.release_frequency).collect();
        
        let processing_times: Vec<u64> = records.iter().map(|r| r.processing_time_ms).collect();
        let timestamps: Vec<u64> = records.iter().map(|r| r.timestamp).collect();
        let extractor_versions: Vec<String> = records.iter().map(|r| r.extractor_version.clone()).collect();
        let cargo_versions: Vec<String> = records.iter().map(|r| r.cargo_version.clone()).collect();
        let rust_versions: Vec<String> = records.iter().map(|r| r.rust_version.clone()).collect();
        
        // Create Arrow arrays
        let id_array = Arc::new(StringArray::from(ids));
        let project_path_array = Arc::new(StringArray::from(project_paths));
        let project_name_array = Arc::new(StringArray::from(project_names));
        let project_version_array = Arc::new(StringArray::from(project_versions));
        let phase_array = Arc::new(StringArray::from(phases));
        let processing_order_array = Arc::new(UInt32Array::from(processing_orders));
        
        let description_array = Arc::new(StringArray::from(descriptions));
        let authors_array = Arc::new(StringArray::from(authors));
        let license_array = Arc::new(StringArray::from(licenses));
        let repository_array = Arc::new(StringArray::from(repositories));
        let homepage_array = Arc::new(StringArray::from(homepages));
        let documentation_array = Arc::new(StringArray::from(documentations));
        let keywords_array = Arc::new(StringArray::from(keywords));
        let categories_array = Arc::new(StringArray::from(categories));
        
        let lines_of_code_array = Arc::new(UInt32Array::from(lines_of_code));
        let source_file_count_array = Arc::new(UInt32Array::from(source_file_counts));
        let test_file_count_array = Arc::new(UInt32Array::from(test_file_counts));
        let example_file_count_array = Arc::new(UInt32Array::from(example_file_counts));
        let benchmark_file_count_array = Arc::new(UInt32Array::from(benchmark_file_counts));
        let complexity_score_array = Arc::new(Float32Array::from(complexity_scores));
        let documentation_coverage_array = Arc::new(Float32Array::from(documentation_coverages));
        
        let direct_dependencies_array = Arc::new(UInt32Array::from(direct_dependencies));
        let total_dependencies_array = Arc::new(UInt32Array::from(total_dependencies));
        let dev_dependencies_array = Arc::new(UInt32Array::from(dev_dependencies));
        let build_dependencies_array = Arc::new(UInt32Array::from(build_dependencies));
        let dependency_data_array = Arc::new(StringArray::from(dependency_data));
        
        let features_array = Arc::new(StringArray::from(features));
        let targets_array = Arc::new(StringArray::from(targets));
        let has_build_script_array = Arc::new(BooleanArray::from(has_build_scripts));
        let build_script_complexity_array = Arc::new(UInt32Array::from(build_script_complexities));
        
        let download_count_array = Arc::new(UInt64Array::from(download_counts));
        let github_stars_array = Arc::new(UInt32Array::from(github_stars));
        let github_forks_array = Arc::new(UInt32Array::from(github_forks));
        let github_issues_array = Arc::new(UInt32Array::from(github_issues));
        let last_updated_array = Arc::new(UInt64Array::from(last_updateds));
        
        let commit_count_array = Arc::new(UInt32Array::from(commit_counts));
        let contributor_count_array = Arc::new(UInt32Array::from(contributor_counts));
        let project_age_days_array = Arc::new(UInt32Array::from(project_age_days));
        let release_frequency_array = Arc::new(Float32Array::from(release_frequencies));
        
        let processing_time_array = Arc::new(UInt64Array::from(processing_times));
        let timestamp_array = Arc::new(UInt64Array::from(timestamps));
        let extractor_version_array = Arc::new(StringArray::from(extractor_versions));
        let cargo_version_array = Arc::new(StringArray::from(cargo_versions));
        let rust_version_array = Arc::new(StringArray::from(rust_versions));
        
        // Create record batch with all arrays
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                id_array,
                project_path_array,
                project_name_array,
                project_version_array,
                phase_array,
                processing_order_array,
                description_array,
                authors_array,
                license_array,
                repository_array,
                homepage_array,
                documentation_array,
                keywords_array,
                categories_array,
                lines_of_code_array,
                source_file_count_array,
                test_file_count_array,
                example_file_count_array,
                benchmark_file_count_array,
                complexity_score_array,
                documentation_coverage_array,
                direct_dependencies_array,
                total_dependencies_array,
                dev_dependencies_array,
                build_dependencies_array,
                dependency_data_array,
                features_array,
                targets_array,
                has_build_script_array,
                build_script_complexity_array,
                download_count_array,
                github_stars_array,
                github_forks_array,
                github_issues_array,
                last_updated_array,
                commit_count_array,
                contributor_count_array,
                project_age_days_array,
                release_frequency_array,
                processing_time_array,
                timestamp_array,
                extractor_version_array,
                cargo_version_array,
                rust_version_array,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_cargo2hf_extractor_creation() {
        let extractor = Cargo2HfExtractor::new();
        assert!(extractor.is_ok());
    }

    #[test]
    fn test_project_metadata_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        
        fs::write(&cargo_toml, r#"
[package]
name = "test-project"
version = "0.1.0"
description = "A test project"
authors = ["Test Author <test@example.com>"]
license = "MIT"
"#).unwrap();

        let mut extractor = Cargo2HfExtractor::new().unwrap();
        let records = extractor.extract_project_metadata(temp_dir.path()).unwrap();
        
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].project_name, "test-project");
        assert_eq!(records[0].project_version, "0.1.0");
        assert_eq!(records[0].description, Some("A test project".to_string()));
        assert_eq!(records[0].license, Some("MIT".to_string()));
    }
}
