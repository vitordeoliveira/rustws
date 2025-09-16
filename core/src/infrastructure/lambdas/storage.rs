//! Lambda storage implementation
//!
//! This module provides the core storage functionality for lambda functions.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::instrument;
use uuid::Uuid;

use super::metrics::LambdasMetrics;
use crate::error_handling::types::{AppError, AppResult};

/// Lambda repository implementation
#[derive(Debug, Clone)]
pub struct LambdaStorage {
    /// Base path for lambda storage
    base_path: PathBuf,
    /// Metrics tracking for lambda executions
    metrics: LambdasMetrics,
}

impl LambdaStorage {
    /// Create new lambda storage manager
    pub fn new() -> Self {
        let mut storage = Self {
            base_path: PathBuf::from("src/infrastructure/lambdas"),
            metrics: LambdasMetrics::default(),
        };
        storage.metrics = storage.load_metrics();
        storage
    }

    /// Get path to source files directory
    pub(crate) fn source_dir(&self) -> PathBuf {
        self.base_path.join("source")
    }

    /// Get path to WASM files directory
    pub(crate) fn wasm_dir(&self) -> PathBuf {
        self.base_path.join("wasm")
    }

    /// Get path to metrics JSON file
    fn metrics_file(&self) -> PathBuf {
        self.base_path.join("metrics.json")
    }

    /// Get lambda execution metrics
    pub fn get_metrics(&self) -> &LambdasMetrics {
        &self.metrics
    }

    /// Get mutable reference to metrics (for updates)
    pub(crate) fn get_metrics_mut(&mut self) -> &mut LambdasMetrics {
        &mut self.metrics
    }

    /// Load metrics from JSON file
    #[instrument(skip_all, fields(operation = "load_metrics"))]
    fn load_metrics(&self) -> LambdasMetrics {
        let metrics_path = self.metrics_file();

        if !metrics_path.exists() {
            tracing::debug!("Metrics file not found, using default metrics");
            return LambdasMetrics::default();
        }

        match fs::read_to_string(&metrics_path) {
            Ok(content) => match serde_json::from_str::<LambdasMetrics>(&content) {
                Ok(metrics) => {
                    tracing::debug!(
                        total_executions = metrics.total_executions,
                        "Loaded metrics from file"
                    );
                    metrics
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "Failed to parse metrics file, using default metrics"
                    );
                    LambdasMetrics::default()
                }
            },
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to read metrics file, using default metrics"
                );
                LambdasMetrics::default()
            }
        }
    }

    /// Save metrics to JSON file
    #[instrument(skip_all, fields(operation = "save_metrics"))]
    pub(crate) fn save_metrics(&self) -> AppResult<()> {
        let metrics_path = self.metrics_file();

        let content = serde_json::to_string_pretty(&self.metrics)
            .map_err(|e| AppError::internal(&format!("Failed to serialize metrics: {}", e)))?;

        fs::write(&metrics_path, content)
            .map_err(|e| AppError::internal(&format!("Failed to save metrics file: {}", e)))?;

        tracing::debug!(
            total_executions = self.metrics.total_executions,
            "Metrics saved to file"
        );

        Ok(())
    }

    /// Compile Rust source code to WASM
    pub(crate) async fn compile_rust(&self, source_code: &str) -> AppResult<Vec<u8>> {
        // Create temporary directory for compilation
        let temp_dir = std::env::temp_dir().join(format!("lambda_compile_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir)
            .map_err(|e| AppError::internal(&format!("Failed to create temp dir: {}", e)))?;

        // Create src directory
        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir)
            .map_err(|e| AppError::internal(&format!("Failed to create src dir: {}", e)))?;

        // Create Cargo.toml with serde dependencies
        let cargo_toml = r#"[package]
name = "lambda_function"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0.219", features = ["derive"] }
serde_json = "1.0.145"


[profile.release]
opt-level = "s"
lto = true
panic = "abort"
strip = "symbols"
"#;

        let cargo_toml_path = temp_dir.join("Cargo.toml");
        fs::write(&cargo_toml_path, cargo_toml)
            .map_err(|e| AppError::internal(&format!("Failed to write Cargo.toml: {}", e)))?;

        // Write source code to lib.rs
        let lib_path = src_dir.join("lib.rs");
        fs::write(&lib_path, source_code)
            .map_err(|e| AppError::internal(&format!("Failed to write source: {}", e)))?;

        // Compile with cargo to WASM
        let output = Command::new("cargo")
            .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
            .current_dir(&temp_dir)
            .output()
            .map_err(|e| AppError::internal(&format!("Failed to run cargo: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            let _ = fs::remove_dir_all(&temp_dir); // Cleanup on error
            return Err(AppError::validation(&format!(
                "Rust compilation failed: {}",
                error
            )));
        }

        // Read compiled WASM bytes
        let wasm_path = temp_dir
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join("lambda_function.wasm");

        let wasm_bytes = fs::read(&wasm_path)
            .map_err(|e| AppError::internal(&format!("Failed to read WASM output: {}", e)))?;

        // Cleanup temporary directory
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(wasm_bytes)
    }

    /// Check if cargo with WASM target is available
    pub(crate) fn check_rust_wasm_toolchain(&self) -> AppResult<()> {
        // Check if cargo is available
        let cargo_check = Command::new("cargo")
            .args(["--version"])
            .output()
            .map_err(|e| AppError::internal(&format!("Cargo not found: {}", e)))?;

        if !cargo_check.status.success() {
            return Err(AppError::internal("Cargo is required but not available"));
        }

        // Check if WASM target is available
        let output = Command::new("rustc")
            .args(["--print", "target-list"])
            .output()
            .map_err(|e| AppError::internal(&format!("Failed to check rustc: {}", e)))?;

        let targets = String::from_utf8_lossy(&output.stdout);
        if !targets.contains("wasm32-unknown-unknown") {
            return Err(AppError::internal(
                "wasm32-unknown-unknown target not found. Install with: rustup target add wasm32-unknown-unknown",
            ));
        }

        Ok(())
    }

    /// Save compiled WASM bytes
    pub(crate) async fn save_compiled_wasm(
        &self,
        lambda_name: &str,
        wasm_bytes: &[u8],
    ) -> AppResult<PathBuf> {
        let wasm_dir = self.wasm_dir();

        // Ensure WASM directory exists
        fs::create_dir_all(&wasm_dir)
            .map_err(|e| AppError::internal(&format!("Failed to create WASM directory: {}", e)))?;

        // Save WASM file with lambda name
        let wasm_path = wasm_dir.join(format!("{}.wasm", lambda_name));
        fs::write(&wasm_path, wasm_bytes)
            .map_err(|e| AppError::internal(&format!("Failed to save WASM file: {}", e)))?;

        Ok(wasm_path)
    }

    /// Compile source code to WASM
    pub(crate) async fn compile(&self, source_code: &str, runtime: &str) -> AppResult<Vec<u8>> {
        match runtime {
            "rs" | "rust" => {
                // Check if Rust WASM toolchain is available
                self.check_rust_wasm_toolchain()?;

                // Compile Rust source to WASM
                self.compile_rust(source_code).await
            }
            _ => Err(AppError::validation(&format!(
                "Unsupported runtime: {}. Currently only Rust ('rs') is supported.",
                runtime
            ))),
        }
    }

    /// Compile source file to WASM
    pub(crate) async fn compile_source_file(&self, lambda_name: &str) -> AppResult<Vec<u8>> {
        let source_dir = self.source_dir();

        // Try different Rust file extensions
        let possible_extensions = ["rs", "rust"];
        let mut source_path = None;

        for ext in &possible_extensions {
            let path = source_dir.join(format!("{}.{}", lambda_name, ext));
            if path.exists() {
                source_path = Some(path);
                break;
            }
        }

        let source_path = source_path.ok_or_else(|| {
            AppError::not_found(&format!(
                "Source file not found for lambda '{}'. Tried extensions: {:?}",
                lambda_name, possible_extensions
            ))
        })?;

        // Read source code
        let source_code = fs::read_to_string(&source_path)
            .map_err(|e| AppError::internal(&format!("Failed to read source file: {}", e)))?;

        // Determine runtime from file extension
        let runtime = source_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("rs");

        // Compile to WASM
        self.compile(&source_code, runtime).await
    }
}

impl Default for LambdaStorage {
    fn default() -> Self {
        Self::new()
    }
}
