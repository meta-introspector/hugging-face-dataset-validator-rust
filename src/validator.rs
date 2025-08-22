use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

// ============================================================================
// Core Data Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationCapability {
    Viewer,
    Preview,
    Search,
    Filter,
    Statistics,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub viewer: bool,
    pub preview: bool,
    pub search: bool,
    pub filter: bool,
    pub statistics: bool,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: &ValidationResult) {
        self.viewer |= other.viewer;
        self.preview |= other.preview;
        self.search |= other.search;
        self.filter |= other.filter;
        self.statistics |= other.statistics;
    }

    // pub fn has_any_capability(&self) -> bool {
    //     self.viewer || self.preview || self.search || self.filter || self.statistics
    // }

    pub fn capability_count(&self) -> usize {
        [self.viewer, self.preview, self.search, self.filter, self.statistics]
            .iter()
            .filter(|&&x| x)
            .count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationLevel {
    Split,
    Config,
    Dataset,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityIdentifier {
    pub dataset: String,
    pub config: Option<String>,
    pub split: Option<String>,
}

impl EntityIdentifier {
    pub fn new_dataset(dataset: String) -> Self {
        Self {
            dataset,
            config: None,
            split: None,
        }
    }

    pub fn new_config(dataset: String, config: String) -> Self {
        Self {
            dataset,
            config: Some(config),
            split: None,
        }
    }

    pub fn new_split(dataset: String, config: String, split: String) -> Self {
        Self {
            dataset,
            config: Some(config),
            split: Some(split),
        }
    }

    pub fn infer_level(&self) -> ValidationLevel {
        match (&self.config, &self.split) {
            (Some(_), Some(_)) => ValidationLevel::Split,
            (Some(_), None) => ValidationLevel::Config,
            (None, None) => ValidationLevel::Dataset,
            (None, Some(_)) => ValidationLevel::Dataset,
        }
    }

    pub fn cache_key(&self, kind: &str) -> String {
        format!(
            "{}:{}:{}:{}",
            kind,
            self.dataset,
            self.config.as_deref().unwrap_or(""),
            self.split.as_deref().unwrap_or("")
        )
    }
}

impl fmt::Display for EntityIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.config, &self.split) {
            (Some(config), Some(split)) => write!(f, "{}/{}/{}", self.dataset, config, split),
            (Some(config), None) => write!(f, "{}/{}", self.dataset, config),
            _ => write!(f, "{}", self.dataset),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedResponse {
    pub http_status: u16,
    pub content: ValidationResult,
    pub progress: f64,
}

impl CachedResponse {
    pub fn new(status: u16, content: ValidationResult, progress: f64) -> Self {
        Self {
            http_status: status,
            content,
            progress,
        }
    }

    pub fn is_success(&self) -> bool {
        self.http_status == 200
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParquetMetadata {
    pub features: HashMap<String, String>,
    pub num_rows: Option<u64>,
}

impl ParquetMetadata {
    pub fn new(features: HashMap<String, String>) -> Self {
        Self {
            features,
            num_rows: None,
        }
    }

    pub fn with_rows(mut self, num_rows: u64) -> Self {
        self.num_rows = Some(num_rows);
        self
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Invalid entity identifier: {message}")]
    InvalidEntityIdentifier { message: String },

    #[error("Data access error: {message}")]
    DataAccessError { message: String },

    #[error("Metadata not found for {entity}")]
    MetadataNotFound { entity: String },

    #[error("Cache error: {message}")]
    CacheError { message: String },

    //#[error("Network error: {message}")]
    //NetworkError { message: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),
}

impl From<serde_json::Error> for ValidationError {
    fn from(err: serde_json::Error) -> Self {
        ValidationError::DataAccessError {
            message: format!("JSON serialization error: {}", err),
        }
    }
}

impl ValidationError {
    // pub fn is_retryable(&self) -> bool {
    //     matches!(
    //         self,
    //         ValidationError::NetworkError { .. } | ValidationError::DataAccessError { .. }
    //     )
    // }
}

// ============================================================================
// Service Traits
// ============================================================================

pub trait DataAccess: Send + Sync {
    fn check_successful_response(&self, kind: &str, entity: &EntityIdentifier) -> Result<bool, ValidationError>;
    fn get_parquet_metadata(&self, dataset: &str, config: &str) -> Result<ParquetMetadata, ValidationError>;
    fn get_split_names(&self, dataset: &str, config: &str) -> Result<Vec<String>, ValidationError>;
    fn get_config_names(&self, dataset: &str) -> Result<Vec<String>, ValidationError>;
    fn get_cached_validation(&self, kind: &str, entity: &EntityIdentifier) -> Result<CachedResponse, ValidationError>;
    fn has_indexable_columns(&self, features: &HashMap<String, String>) -> bool;
}

// ============================================================================
// Mock Implementation
// ============================================================================

#[derive(Clone)]
pub struct MockDataAccess {
    successful_responses: HashMap<String, bool>,
    parquet_metadata: HashMap<String, ParquetMetadata>,
    split_names: HashMap<String, Vec<String>>,
    config_names: HashMap<String, Vec<String>>,
    cached_validations: HashMap<String, CachedResponse>,
}

impl MockDataAccess {
    pub fn new() -> Self {
        Self {
            successful_responses: HashMap::new(),
            parquet_metadata: HashMap::new(),
            split_names: HashMap::new(),
            config_names: HashMap::new(),
            cached_validations: HashMap::new(),
        }
    }

    pub fn setup_default_data(&mut self) {
        let datasets = vec!["user/repo", "org/dataset", "mock/dataset"];
        let configs = vec!["default", "extra"];
        let splits = vec!["train", "test", "validation"];

        for dataset in &datasets {
            self.config_names.insert(dataset.to_string(), configs.iter().map(|s| s.to_string()).collect());
            
            for config in &configs {
                self.split_names.insert(format!("{}:{}", dataset, config), splits.iter().map(|s| s.to_string()).collect());
                
                let mut features = HashMap::new();
                features.insert("text".to_string(), "string".to_string());
                features.insert("label".to_string(), "int64".to_string());
                
                self.parquet_metadata.insert(format!("{}:{}", dataset, config), ParquetMetadata::new(features));
                
                for split in &splits {
                    let entity = EntityIdentifier::new_split(dataset.to_string(), config.to_string(), split.to_string());
                    
                    self.successful_responses.insert(entity.cache_key("config-has-viewer"), true);
                    self.successful_responses.insert(entity.cache_key("split-has-preview"), true);
                    self.successful_responses.insert(entity.cache_key("split-has-statistics"), split != &"validation".to_string());
                    
                    let result = ValidationResult {
                        viewer: true,
                        preview: true,
                        search: true,
                        filter: true,
                        statistics: split != &"validation".to_string(),
                    };
                    
                    self.cached_validations.insert(entity.cache_key("split-is-valid"), CachedResponse::new(200, result, 1.0));
                }
                
                let config_entity = EntityIdentifier::new_config(dataset.to_string(), config.to_string());
                let config_result = ValidationResult {
                    viewer: true,
                    preview: true,
                    search: true,
                    filter: true,
                    statistics: true,
                };
                self.cached_validations.insert(config_entity.cache_key("config-is-valid"), CachedResponse::new(200, config_result, 1.0));
            }
        }
    }
}

impl Default for MockDataAccess {
    fn default() -> Self {
        let mut mock = Self::new();
        mock.setup_default_data();
        mock
    }
}

impl DataAccess for MockDataAccess {
    fn check_successful_response(&self, kind: &str, entity: &EntityIdentifier) -> Result<bool, ValidationError> {
        let key = entity.cache_key(kind);
        self.successful_responses.get(&key)
            .copied()
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("No response found for {}", key),
            })
    }

    fn get_parquet_metadata(&self, dataset: &str, config: &str) -> Result<ParquetMetadata, ValidationError> {
        let key = format!("{}:{}", dataset, config);
        self.parquet_metadata.get(&key)
            .cloned()
            .ok_or_else(|| ValidationError::MetadataNotFound {
                entity: key,
            })
    }

    fn get_split_names(&self, dataset: &str, config: &str) -> Result<Vec<String>, ValidationError> {
        let key = format!("{}:{}", dataset, config);
        self.split_names.get(&key)
            .cloned()
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("No split names found for {}", key),
            })
    }

    fn get_config_names(&self, dataset: &str) -> Result<Vec<String>, ValidationError> {
        self.config_names.get(dataset)
            .cloned()
            .ok_or_else(|| ValidationError::DataAccessError {
                message: format!("No config names found for {}", dataset),
            })
    }

    fn get_cached_validation(&self, kind: &str, entity: &EntityIdentifier) -> Result<CachedResponse, ValidationError> {
        let key = entity.cache_key(kind);
        self.cached_validations.get(&key)
            .cloned()
            .ok_or_else(|| ValidationError::CacheError {
                message: format!("No cached validation found for {}", key),
            })
    }

    fn has_indexable_columns(&self, features: &HashMap<String, String>) -> bool {
        features.values().any(|v| v.contains("string") || v.contains("text"))
    }
}

// ============================================================================
// Validator Implementation
// ============================================================================

pub struct DatasetValidator<D: DataAccess> {
    pub data_access: D,
}

impl<D: DataAccess> DatasetValidator<D> {
    pub fn new(data_access: D) -> Self {
        Self { data_access }
    }

    pub fn validate(&self, entity: &EntityIdentifier, level: ValidationLevel) -> Result<(ValidationResult, f64), ValidationError> {
        match level {
            ValidationLevel::Split => self.validate_split(entity),
            ValidationLevel::Config => self.validate_config(entity),
            ValidationLevel::Dataset => self.validate_dataset(entity),
        }
    }

    fn validate_split(&self, entity: &EntityIdentifier) -> Result<(ValidationResult, f64), ValidationError> {
        let dataset = &entity.dataset;
        let config = entity.config.as_ref().ok_or_else(|| ValidationError::InvalidEntityIdentifier {
            message: "Config required for split validation".to_string(),
        })?;
        let _split = entity.split.as_ref().ok_or_else(|| ValidationError::InvalidEntityIdentifier {
            message: "Split required for split validation".to_string(),
        })?;

        let mut result = ValidationResult::new();

        let config_entity = EntityIdentifier::new_config(dataset.clone(), config.clone());
        result.viewer = self.data_access.check_successful_response("config-has-viewer", &config_entity).unwrap_or(false);
        result.preview = self.data_access.check_successful_response("split-has-preview", entity).unwrap_or(false);

        match self.data_access.get_parquet_metadata(dataset, config) {
            Ok(metadata) => {
                result.filter = true;
                result.search = self.data_access.has_indexable_columns(&metadata.features);
            }
            Err(_) => {
                result.filter = false;
                result.search = false;
            }
        }

        result.statistics = self.data_access.check_successful_response("split-has-statistics", entity).unwrap_or(false);

        Ok((result, 1.0))
    }

    fn validate_config(&self, entity: &EntityIdentifier) -> Result<(ValidationResult, f64), ValidationError> {
        let dataset = &entity.dataset;
        let config = entity.config.as_ref().ok_or_else(|| ValidationError::InvalidEntityIdentifier {
            message: "Config required for config validation".to_string(),
        })?;

        let mut result = ValidationResult::new();
        let mut total = 0;
        let mut pending = 0;

        let splits = self.data_access.get_split_names(dataset, config)?;

        for split in &splits {
            total += 1;
            let split_entity = EntityIdentifier::new_split(dataset.clone(), config.clone(), split.clone());

            match self.data_access.get_cached_validation("split-is-valid", &split_entity) {
                Ok(split_result) if split_result.is_success() => {
                    result.merge(&split_result.content);
                }
                Err(ValidationError::CacheError { .. }) => {
                    pending += 1;
                }
                _ => {}
            }
        }

        let progress = if total > 0 {
            (total - pending) as f64 / total as f64
        } else {
            1.0
        };

        Ok((result, progress))
    }

    fn validate_dataset(&self, entity: &EntityIdentifier) -> Result<(ValidationResult, f64), ValidationError> {
        let dataset = &entity.dataset;

        let mut result = ValidationResult::new();
        let mut total = 0;
        let mut pending = 0;

        let configs = self.data_access.get_config_names(dataset)?;

        for config in &configs {
            total += 1;
            let config_entity = EntityIdentifier::new_config(dataset.clone(), config.clone());

            match self.data_access.get_cached_validation("config-is-valid", &config_entity) {
                Ok(config_result) if config_result.is_success() => {
                    result.merge(&config_result.content);
                }
                Err(ValidationError::CacheError { .. }) => {
                    pending += 1;
                }
                _ => {}
            }
        }

        let progress = if total > 0 {
            (total - pending) as f64 / total as f64
        } else {
            1.0
        };

        Ok((result, progress))
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

pub fn validate_split<D: DataAccess>(
    dataset: &str,
    config: &str,
    split: &str,
    data_access: D,
) -> Result<(ValidationResult, f64), ValidationError> {
    let validator = DatasetValidator::new(data_access);
    let entity = EntityIdentifier::new_split(dataset.to_string(), config.to_string(), split.to_string());
    validator.validate(&entity, ValidationLevel::Split)
}

pub fn validate_config<D: DataAccess>(
    dataset: &str,
    config: &str,
    data_access: D,
) -> Result<(ValidationResult, f64), ValidationError> {
    let validator = DatasetValidator::new(data_access);
    let entity = EntityIdentifier::new_config(dataset.to_string(), config.to_string());
    validator.validate(&entity, ValidationLevel::Config)
}

pub fn validate_dataset<D: DataAccess>(
    dataset: &str,
    data_access: D,
) -> Result<(ValidationResult, f64), ValidationError> {
    let validator = DatasetValidator::new(data_access);
    let entity = EntityIdentifier::new_dataset(dataset.to_string());
    validator.validate(&entity, ValidationLevel::Dataset)
}
