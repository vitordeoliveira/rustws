//! Lambda infrastructure management
//!
//! This module handles the filesystem layer for lambda functions:
//! - Loading lambda source code
//! - Reading/writing WASM files
//! - Managing lambda function artifacts

use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error_handling::types::{AppError, AppResult};

/// Lambda storage manager for handling filesystem operations
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

    /// Get path to WASM files directory
    pub fn wasm_dir(&self) -> PathBuf {
        self.base_path.join("wasm")
    }

    /// Get path to source files directory
    pub fn source_dir(&self) -> PathBuf {
        self.base_path.join("source")
    }

    /// Save WASM file for a lambda function
    pub async fn save_wasm(&self, lambda_id: Uuid, wasm_bytes: &[u8]) -> AppResult<PathBuf> {
        let wasm_path = self.wasm_dir().join(format!("{}.wasm", lambda_id));

        // Ensure directory exists
        if let Some(parent) = wasm_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::internal(&format!("Failed to create WASM directory: {}", e))
            })?;
        }

        // Write WASM file
        fs::write(&wasm_path, wasm_bytes)
            .map_err(|e| AppError::internal(&format!("Failed to write WASM file: {}", e)))?;

        Ok(wasm_path)
    }

    /// Load WASM file for a lambda function
    pub async fn load_wasm(&self, lambda_id: Uuid) -> AppResult<Vec<u8>> {
        let wasm_path = self.wasm_dir().join(format!("{}.wasm", lambda_id));

        fs::read(&wasm_path).map_err(|e| {
            AppError::not_found(&format!(
                "WASM file not found for lambda {}: {}",
                lambda_id, e
            ))
        })
    }

    /// Delete WASM file for a lambda function
    pub async fn delete_wasm(&self, lambda_id: Uuid) -> AppResult<()> {
        let wasm_path = self.wasm_dir().join(format!("{}.wasm", lambda_id));

        if wasm_path.exists() {
            fs::remove_file(&wasm_path)
                .map_err(|e| AppError::internal(&format!("Failed to delete WASM file: {}", e)))?;
        }

        Ok(())
    }

    /// Save source code for a lambda function
    pub async fn save_source(
        &self,
        lambda_name: &str,
        source_code: &str,
        extension: &str,
    ) -> AppResult<PathBuf> {
        let source_path = self
            .source_dir()
            .join(format!("{}.{}", lambda_name, extension));

        // Ensure directory exists
        if let Some(parent) = source_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::internal(&format!("Failed to create source directory: {}", e))
            })?;
        }

        // Write source file
        fs::write(&source_path, source_code)
            .map_err(|e| AppError::internal(&format!("Failed to write source file: {}", e)))?;

        Ok(source_path)
    }

    /// Load source code for a lambda function
    pub async fn load_source(&self, lambda_name: &str, extension: &str) -> AppResult<String> {
        let source_path = self
            .source_dir()
            .join(format!("{}.{}", lambda_name, extension));

        fs::read_to_string(&source_path).map_err(|e| {
            AppError::not_found(&format!(
                "Source file not found for lambda {}: {}",
                lambda_name, e
            ))
        })
    }

    /// List all available WASM files
    pub async fn list_wasm_files(&self) -> AppResult<Vec<PathBuf>> {
        let wasm_dir = self.wasm_dir();

        if !wasm_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&wasm_dir)
            .map_err(|e| AppError::internal(&format!("Failed to read WASM directory: {}", e)))?;

        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                AppError::internal(&format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                files.push(path);
            }
        }

        Ok(files)
    }

    /// List all available source files
    pub async fn list_source_files(&self) -> AppResult<Vec<PathBuf>> {
        let source_dir = self.source_dir();

        if !source_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&source_dir)
            .map_err(|e| AppError::internal(&format!("Failed to read source directory: {}", e)))?;

        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                AppError::internal(&format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_file() {
                files.push(path);
            }
        }

        Ok(files)
    }
}

impl Default for LambdaStorage {
    fn default() -> Self {
        Self::new()
    }
}
