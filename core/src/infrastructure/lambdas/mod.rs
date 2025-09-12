//! Lambda infrastructure implementation
//!
//! This module provides the concrete implementation of lambda repository

use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::business_logic::lambdas::{LambdaRepository, LambdaStatus, LambdaSummary};
use crate::error_handling::types::{AppError, AppResult};

/// Lambda repository implementation
pub struct LambdaStorage {
    /// Base path for lambda storage
    base_path: PathBuf,
}

impl LambdaStorage {
    /// Create new lambda storage manager
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("src/infrastructure/lambdas"),
        }
    }

    /// Create lambda storage with custom base path
    pub fn with_base_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            base_path: path.as_ref().to_path_buf(),
        }
    }

    /// Get path to source files directory
    fn source_dir(&self) -> PathBuf {
        self.base_path.join("source")
    }
}

impl Default for LambdaStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of LambdaRepository trait for LambdaStorage
impl LambdaRepository for LambdaStorage {
    /// Get all lambda functions
    async fn get_all(&self) -> AppResult<Vec<LambdaSummary>> {
        let source_dir = self.source_dir();

        if !source_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&source_dir)
            .map_err(|e| AppError::internal(&format!("Failed to read source directory: {}", e)))?;

        let mut summaries = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                AppError::internal(&format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_file() {
                // Extract filename without extension as lambda name
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Extract file extension as runtime
                let runtime = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Create a mock LambdaSummary for the source file
                let summary = LambdaSummary {
                    id: Uuid::new_v4(),
                    name,
                    description: Some(format!("Lambda function from source file")),
                    runtime,
                    memory_mb: 128,                 // Default memory
                    timeout_seconds: 30,            // Default timeout
                    status: LambdaStatus::Inactive, // Source files are not compiled yet
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                summaries.push(summary);
            }
        }

        Ok(summaries)
    }

    /// Compile source code to WASM
    async fn compile(&self, source_code: &str, runtime: &str) -> AppResult<Vec<u8>> {
        // TODO: Implementation for compiling source code to WASM
        // For now, return error indicating not implemented
        Err(AppError::internal(&format!(
            "Compilation for runtime '{}' not yet implemented",
            runtime
        )))
    }
}
