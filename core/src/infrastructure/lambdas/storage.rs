//! Lambda storage implementation
//!
//! This module provides the core storage functionality for lambda functions.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::instrument;
use uuid::Uuid;

use super::metrics::{LambdaMetricsLedger, LambdasMetrics};
use crate::error_handling::types::{AppError, AppResult};

/// Lambda repository implementation
#[derive(Debug, Clone)]
pub struct LambdaStorage {
    /// Base path for lambda storage
    base_path: PathBuf,
    /// Metrics ledger for lambda executions
    metrics_ledger: LambdaMetricsLedger,
}

impl LambdaStorage {
    /// Create new lambda storage manager
    pub fn new() -> Self {
        let mut storage = Self {
            base_path: PathBuf::from("src/infrastructure/lambdas"),
            metrics_ledger: LambdaMetricsLedger::default(),
        };
        storage.metrics_ledger = storage.load_metrics_ledger();
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

    /// Get lambda execution metrics (calculated from ledger)
    pub fn get_metrics(&self) -> LambdasMetrics {
        self.metrics_ledger.calculate_totals()
    }

    /// Get mutable reference to metrics ledger
    pub(crate) fn get_metrics_ledger_mut(&mut self) -> &mut LambdaMetricsLedger {
        &mut self.metrics_ledger
    }

    /// Get reference to metrics ledger
    pub fn get_metrics_ledger(&self) -> &LambdaMetricsLedger {
        &self.metrics_ledger
    }

    /// Load metrics ledger from JSON file
    #[instrument(skip_all, fields(operation = "load_metrics_ledger"))]
    fn load_metrics_ledger(&self) -> LambdaMetricsLedger {
        let metrics_path = self.metrics_file();

        if !metrics_path.exists() {
            tracing::debug!("Metrics file not found, using default ledger");
            return LambdaMetricsLedger::default();
        }

        match fs::read_to_string(&metrics_path) {
            Ok(content) => {
                // Try to load as new ledger format first
                match serde_json::from_str::<LambdaMetricsLedger>(&content) {
                    Ok(ledger) => {
                        tracing::debug!(
                            entries_count = ledger.entries.len(),
                            "Loaded metrics ledger from file"
                        );
                        ledger
                    }
                    Err(_) => {
                        // Try to load as old metrics format for backward compatibility
                        match serde_json::from_str::<LambdasMetrics>(&content) {
                            Ok(_old_metrics) => {
                                tracing::warn!(
                                    "Found old metrics format, starting with empty ledger"
                                );
                                LambdaMetricsLedger::default()
                            }
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    "Failed to parse metrics file, using default ledger"
                                );
                                LambdaMetricsLedger::default()
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to read metrics file, using default ledger"
                );
                LambdaMetricsLedger::default()
            }
        }
    }

    /// Save metrics ledger to JSON file
    #[instrument(skip_all, fields(operation = "save_metrics"))]
    pub(crate) fn save_metrics(&self) -> AppResult<()> {
        let metrics_path = self.metrics_file();

        let content = serde_json::to_string_pretty(&self.metrics_ledger).map_err(|e| {
            AppError::internal(&format!("Failed to serialize metrics ledger: {}", e))
        })?;

        fs::write(&metrics_path, content)
            .map_err(|e| AppError::internal(&format!("Failed to save metrics file: {}", e)))?;

        let totals = self.metrics_ledger.calculate_totals();
        tracing::debug!(
            entries_count = self.metrics_ledger.entries.len(),
            total_executions = totals.total_executions,
            "Metrics ledger saved to file"
        );

        Ok(())
    }

    /// Compile Rust source code to WASM with metadata
    pub(crate) async fn compile_rust(
        &self,
        source_code: &str,
    ) -> AppResult<(
        Vec<u8>,
        Option<crate::business_logic::lambdas::dto::LambdaMetadata>,
    )> {
        // Create temporary directory for compilation
        let temp_dir = std::env::temp_dir().join(format!("lambda_compile_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir)
            .map_err(|e| AppError::internal(&format!("Failed to create temp dir: {}", e)))?;

        // Create src directory
        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir)
            .map_err(|e| AppError::internal(&format!("Failed to create src dir: {}", e)))?;

        // Create Cargo.toml with lambda macro and serde dependencies
        let cargo_toml = format!(
            r#"[package]
name = "lambda_function"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = {{ version = "1.0.219", features = ["derive"] }}
serde_json = "1.0.145"
schemars = {{ version = "0.8", features = ["derive"] }}
chrono = {{ version = "0.4", features = ["serde"] }}
lambda-fn-macro = {{ path = "{}" }}

[profile.release]
opt-level = "s"
lto = true
panic = "abort"
strip = "symbols"
"#,
            std::env::current_dir()
                .unwrap()
                .parent()
                .unwrap()
                .join("lambda-fn-macro")
                .display()
        );

        let cargo_toml_path = temp_dir.join("Cargo.toml");
        fs::write(&cargo_toml_path, cargo_toml)
            .map_err(|e| AppError::internal(&format!("Failed to write Cargo.toml: {}", e)))?;

        // Process user source to fix outer doc comments only
        let processed_source = self.fix_outer_doc_comments(source_code);

        // Write processed source code to lib.rs
        let lib_path = src_dir.join("lib.rs");
        fs::write(&lib_path, processed_source)
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

        // Extract metadata from compiled WASM before cleanup
        let metadata = self
            .extract_lambda_metadata(&wasm_bytes)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to extract lambda metadata during compilation");
                None
            });

        // Cleanup temporary directory
        let _ = fs::remove_dir_all(&temp_dir);

        Ok((wasm_bytes, metadata))
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

    /// Save compiled WASM bytes and metadata
    pub(crate) async fn save_compiled_wasm(
        &self,
        lambda_name: &str,
        wasm_bytes: &[u8],
        metadata: Option<crate::business_logic::lambdas::dto::LambdaMetadata>,
    ) -> AppResult<PathBuf> {
        let wasm_dir = self.wasm_dir();

        // Ensure WASM directory exists
        fs::create_dir_all(&wasm_dir)
            .map_err(|e| AppError::internal(&format!("Failed to create WASM directory: {}", e)))?;

        // Save WASM file with lambda name
        let wasm_path = wasm_dir.join(format!("{}.wasm", lambda_name));
        fs::write(&wasm_path, wasm_bytes)
            .map_err(|e| AppError::internal(&format!("Failed to save WASM file: {}", e)))?;

        // Save metadata file alongside WASM
        if let Some(metadata) = metadata {
            let metadata_path = wasm_dir.join(format!("{}.json", lambda_name));
            let metadata_json = serde_json::to_string_pretty(&metadata)
                .map_err(|e| AppError::internal(&format!("Failed to serialize metadata: {}", e)))?;

            fs::write(&metadata_path, metadata_json)
                .map_err(|e| AppError::internal(&format!("Failed to save metadata file: {}", e)))?;

            tracing::info!(
                lambda_name = %lambda_name,
                metadata_path = %metadata_path.display(),
                "Saved lambda metadata to file"
            );
        }

        Ok(wasm_path)
    }

    /// Load metadata for a lambda if it exists
    pub(crate) async fn load_lambda_metadata(
        &self,
        lambda_name: &str,
    ) -> AppResult<Option<crate::business_logic::lambdas::dto::LambdaMetadata>> {
        let wasm_dir = self.wasm_dir();
        let metadata_path = wasm_dir.join(format!("{}.json", lambda_name));

        if !metadata_path.exists() {
            return Ok(None);
        }

        let metadata_content = fs::read_to_string(&metadata_path)
            .map_err(|e| AppError::internal(&format!("Failed to read metadata file: {}", e)))?;

        let metadata: crate::business_logic::lambdas::dto::LambdaMetadata =
            serde_json::from_str(&metadata_content).map_err(|e| {
                AppError::internal(&format!("Failed to parse metadata JSON: {}", e))
            })?;

        tracing::debug!(
            lambda_name = %lambda_name,
            metadata_path = %metadata_path.display(),
            "Loaded lambda metadata from file"
        );

        Ok(Some(metadata))
    }

    /// Compile source code to WASM with metadata
    pub(crate) async fn compile(
        &self,
        source_code: &str,
        runtime: &str,
    ) -> AppResult<(
        Vec<u8>,
        Option<crate::business_logic::lambdas::dto::LambdaMetadata>,
    )> {
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

    /// Fix outer doc comments that can't appear in the middle of a file
    fn fix_outer_doc_comments(&self, user_source: &str) -> String {
        user_source
            .lines()
            .map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("//!") {
                    // Convert outer doc comments to regular comments
                    line.replace("//!", "//")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Compile source file to WASM with metadata
    pub(crate) async fn compile_source_file_impl(
        &self,
        lambda_name: &str,
    ) -> AppResult<(
        Vec<u8>,
        Option<crate::business_logic::lambdas::dto::LambdaMetadata>,
    )> {
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

        // Compile to WASM and return both WASM bytes and metadata
        self.compile(&source_code, runtime).await
    }

    /// Extract essential metadata from compiled WASM lambda
    #[instrument(skip_all, fields(wasm_size = wasm_bytes.len()))]
    pub(crate) async fn extract_lambda_metadata(
        &self,
        wasm_bytes: &[u8],
    ) -> AppResult<Option<crate::business_logic::lambdas::dto::LambdaMetadata>> {
        use crate::business_logic::lambdas::dto::{LambdaFeatures, LambdaMetadata};
        use wasmer::{Instance, Module, Store, imports};

        tracing::info!("Extracting metadata from compiled lambda WASM");

        // Create WASM runtime environment
        let mut store = Store::default();
        let module = Module::new(&store, wasm_bytes).map_err(|e| {
            AppError::internal(&format!(
                "Failed to load WASM module for metadata extraction: {}",
                e
            ))
        })?;

        // Create stub host functions for HTTP and environment-enabled lambdas
        // These are dummy implementations - they won't be called during metadata extraction
        let host_http_request =
            wasmer::Function::new_typed(&mut store, |_: i32, _: i32, _: i32, _: i32| -> i32 {
                // Stub implementation - not called during metadata extraction
                -1
            });

        let host_get_env =
            wasmer::Function::new_typed(&mut store, |_: i32, _: i32, _: i32, _: i32| -> i32 {
                // Stub implementation - not called during metadata extraction
                -1
            });

        let wasm_free = wasmer::Function::new_typed(&mut store, |_: i32, _: i32| {
            // Stub implementation - not called during metadata extraction
        });

        // Create instance with the required host functions
        let instance = Instance::new(
            &mut store,
            &module,
            &imports! {
                "env" => {
                    "host_http_request" => host_http_request,
                    "host_get_env" => host_get_env,
                    "wasm_free" => wasm_free,
                }
            },
        )
        .map_err(|e| {
            AppError::internal(&format!(
                "Failed to create WASM instance for metadata extraction: {}",
                e
            ))
        })?;

        // Extract metadata by calling the lambda's metadata functions
        let raw_metadata = match self
            .call_wasm_function(&mut store, &instance, "get_lambda_metadata")
            .await
        {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to extract lambda metadata - lambda may not support metadata");
                return Ok(None);
            }
        };

        // Extract input schema
        let raw_input_schema = match self
            .call_wasm_function(&mut store, &instance, "get_input_schema")
            .await
        {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to extract input schema");
                return Ok(None);
            }
        };

        // Extract output schema
        let raw_output_schema = match self
            .call_wasm_function(&mut store, &instance, "get_output_schema")
            .await
        {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to extract output schema");
                return Ok(None);
            }
        };

        // Parse the metadata JSON
        let metadata_json: serde_json::Value =
            serde_json::from_str(&raw_metadata).map_err(|e| {
                AppError::internal(&format!("Failed to parse lambda metadata JSON: {}", e))
            })?;

        // Parse the input schema JSON
        let input_schema: serde_json::Value =
            serde_json::from_str(&raw_input_schema).map_err(|e| {
                AppError::internal(&format!("Failed to parse input schema JSON: {}", e))
            })?;

        // Parse the output schema JSON
        let output_schema: serde_json::Value =
            serde_json::from_str(&raw_output_schema).map_err(|e| {
                AppError::internal(&format!("Failed to parse output schema JSON: {}", e))
            })?;

        // Build complete metadata structure with schemas
        let lambda_metadata = LambdaMetadata {
            features: LambdaFeatures {
                http_enabled: metadata_json["features"]["http_enabled"]
                    .as_bool()
                    .unwrap_or(false),
                env_enabled: metadata_json["features"]["env_enabled"]
                    .as_bool()
                    .unwrap_or(false),
                experimental_features: vec![], // Future extensibility
            },
            input_type: metadata_json["input_type"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            output_type: metadata_json["output_type"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            input_schema,
            output_schema,
            macro_version: metadata_json["macro_version"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            compiled_at: metadata_json["compiled_at"]
                .as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
        };

        tracing::info!(
            input_type = %lambda_metadata.input_type,
            output_type = %lambda_metadata.output_type,
            http_enabled = lambda_metadata.features.http_enabled,
            macro_version = %lambda_metadata.macro_version,
            input_schema_size = lambda_metadata.input_schema.to_string().len(),
            output_schema_size = lambda_metadata.output_schema.to_string().len(),
            "Successfully extracted lambda metadata with full schemas"
        );

        Ok(Some(lambda_metadata))
    }

    /// Call a WASM function and return the result as a string
    #[instrument(skip_all, fields(function_name = %function_name))]
    async fn call_wasm_function(
        &self,
        store: &mut wasmer::Store,
        instance: &wasmer::Instance,
        function_name: &str,
    ) -> AppResult<String> {
        use wasmer::{Function, Value};

        // Get the function from exports
        let func = instance
            .exports
            .get::<Function>(function_name)
            .map_err(|e| {
                AppError::internal(&format!(
                    "Function '{}' not found in WASM exports: {}",
                    function_name, e
                ))
            })?;

        // Allocate buffer for the result
        let buffer_size = 8192; // 8KB should be enough for metadata
        let malloc_fn = instance
            .exports
            .get::<Function>("wasm_malloc")
            .map_err(|e| AppError::internal(&format!("wasm_malloc function not found: {}", e)))?;

        let buffer_ptr = malloc_fn
            .call(store, &[Value::I32(buffer_size as i32)])
            .map_err(|e| AppError::internal(&format!("Failed to allocate WASM memory: {}", e)))?[0]
            .i32()
            .ok_or_else(|| AppError::internal("wasm_malloc returned invalid pointer"))?;

        // Call the function
        let actual_size = func
            .call(
                store,
                &[Value::I32(buffer_ptr), Value::I32(buffer_size as i32)],
            )
            .map_err(|e| {
                AppError::internal(&format!(
                    "Failed to call WASM function '{}': {}",
                    function_name, e
                ))
            })?[0]
            .i32()
            .ok_or_else(|| {
                AppError::internal(&format!(
                    "Function '{}' returned invalid size",
                    function_name
                ))
            })?;

        if actual_size == 0 {
            return Err(AppError::internal(&format!(
                "Function '{}' returned empty result",
                function_name
            )));
        }

        // Read the result from WASM memory
        let memory = instance
            .exports
            .get_memory("memory")
            .map_err(|e| AppError::internal(&format!("WASM memory not found: {}", e)))?;

        let result_bytes = memory
            .view(store)
            .copy_range_to_vec(
                buffer_ptr as u64..(buffer_ptr + actual_size.min(buffer_size as i32)) as u64,
            )
            .map_err(|e| AppError::internal(&format!("Failed to read WASM memory: {}", e)))?;

        // Free the allocated memory
        if let Ok(free_fn) = instance.exports.get::<Function>("wasm_free_impl") {
            let _ = free_fn.call(
                store,
                &[Value::I32(buffer_ptr), Value::I32(buffer_size as i32)],
            );
        }

        // Convert to string
        String::from_utf8(result_bytes).map_err(|e| {
            AppError::internal(&format!("Invalid UTF-8 in WASM function result: {}", e))
        })
    }
}

impl Default for LambdaStorage {
    fn default() -> Self {
        Self::new()
    }
}
