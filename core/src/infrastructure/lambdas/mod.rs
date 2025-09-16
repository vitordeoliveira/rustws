//! Lambda infrastructure implementation
//!
//! This module provides the concrete implementation of lambda repository

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::instrument;
use uuid::Uuid;

use crate::business_logic::lambdas::{
    CreateLambdaRequest, CreateLambdaResponse, ExecuteLambdaRequest, ExecuteLambdaResponse, Lambda,
    LambdaRepository, LambdaStatus, LambdaSummary, UpdateLambdaRequest,
};
use crate::error_handling::types::{AppError, AppResult};

/// Metrics tracking for lambda executions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LambdasMetrics {
    /// Total number of lambda executions
    pub total_executions: u64,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Most recently executed lambda name
    pub last_executed_lambda: Option<String>,
    /// Timestamp of last execution
    pub last_execution_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for LambdasMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            total_execution_time_ms: 0,
            successful_executions: 0,
            failed_executions: 0,
            last_executed_lambda: None,
            last_execution_time: None,
        }
    }
}

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
    fn source_dir(&self) -> PathBuf {
        self.base_path.join("source")
    }

    /// Get path to WASM files directory
    fn wasm_dir(&self) -> PathBuf {
        self.base_path.join("wasm")
    }

    /// Get path to metrics JSON file
    fn metrics_file(&self) -> PathBuf {
        self.base_path.join("metrics.json")
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
    fn save_metrics(&self) -> AppResult<()> {
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
    async fn compile_rust(&self, source_code: &str) -> AppResult<Vec<u8>> {
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
    fn check_rust_wasm_toolchain(&self) -> AppResult<()> {
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

                // Check if corresponding WASM file exists
                let wasm_dir = self.wasm_dir();
                let wasm_path = wasm_dir.join(format!("{}.wasm", name));
                let status = if wasm_path.exists() {
                    LambdaStatus::Active
                } else {
                    LambdaStatus::Inactive
                };

                // Create a LambdaSummary for the source file
                let summary = LambdaSummary {
                    id: Uuid::new_v4(),
                    name,
                    description: Some(format!("Lambda function from source file")),
                    runtime,
                    memory_mb: 128,      // Default memory
                    timeout_seconds: 30, // Default timeout
                    status,              // Active if WASM exists, Inactive otherwise
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
    async fn compile_source_file(&self, lambda_name: &str) -> AppResult<Vec<u8>> {
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

    /// Save compiled WASM bytes
    async fn save_compiled_wasm(&self, lambda_name: &str, wasm_bytes: &[u8]) -> AppResult<PathBuf> {
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

    /// Execute a lambda function
    async fn execute(&mut self, request: ExecuteLambdaRequest) -> AppResult<ExecuteLambdaResponse> {
        let start_time = std::time::Instant::now();
        // Determine which lambda to execute
        let lambda_name = request
            .function_name
            .ok_or_else(|| AppError::validation("function_name is required for execution"))?;

        // Get path to compiled WASM file
        let wasm_dir = self.wasm_dir();
        let wasm_path = wasm_dir.join(format!("{}.wasm", lambda_name));

        if !wasm_path.exists() {
            return Ok(ExecuteLambdaResponse::Failed {
                error_message: format!(
                    "WASM file not found for lambda '{}'. Compile the lambda first.",
                    lambda_name
                ),
            });
        }

        // Read WASM bytes
        let wasm_bytes = match fs::read(&wasm_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Ok(ExecuteLambdaResponse::Failed {
                    error_message: format!("Failed to read WASM file: {}", e),
                });
            }
        };

        // Execute WASM using wasmer
        let result = match self.execute_wasm(&wasm_bytes, &request.input_data).await {
            Ok(output) => ExecuteLambdaResponse::Success {
                output_data: output,
            },
            Err(e) => ExecuteLambdaResponse::Failed {
                error_message: format!("WASM execution failed: {}", e),
            },
        };

        // Update metrics
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        self.metrics.total_executions += 1;
        self.metrics.total_execution_time_ms += execution_time_ms;
        self.metrics.last_executed_lambda = Some(lambda_name.clone());
        self.metrics.last_execution_time = Some(chrono::Utc::now());

        match &result {
            ExecuteLambdaResponse::Success { .. } => {
                self.metrics.successful_executions += 1;
                tracing::info!(
                    lambda_name = %lambda_name,
                    execution_time_ms = execution_time_ms,
                    "Lambda execution successful"
                );
            }
            ExecuteLambdaResponse::Failed { .. } => {
                self.metrics.failed_executions += 1;
                tracing::warn!(
                    lambda_name = %lambda_name,
                    execution_time_ms = execution_time_ms,
                    "Lambda execution failed"
                );
            }
        }

        // Save metrics to file
        if let Err(e) = self.save_metrics() {
            tracing::warn!(error = %e, "Failed to save metrics to file");
        }

        Ok(result)
    }

    /// Create a new lambda function - saves source code and optionally compiles to WASM
    async fn create(&self, request: CreateLambdaRequest) -> AppResult<CreateLambdaResponse> {
        let start_time = std::time::Instant::now();

        tracing::info!(
            function_name = %request.function_name,
            runtime = %request.runtime,
            "Creating new lambda function"
        );

        // Validate function name (basic validation for file system safety)
        if request.function_name.trim().is_empty() {
            return Ok(CreateLambdaResponse {
                success: false,
                message: "Function name cannot be empty".to_string(),
                function_name: request.function_name,
                source_path: None,
                wasm_path: None,
                wasm_size_bytes: None,
                compilation_time_ms: None,
            });
        }

        // Check for valid identifier pattern
        if !request
            .function_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            return Ok(CreateLambdaResponse {
                success: false,
                message: "Function name must contain only letters, numbers, and underscores"
                    .to_string(),
                function_name: request.function_name,
                source_path: None,
                wasm_path: None,
                wasm_size_bytes: None,
                compilation_time_ms: None,
            });
        }

        // Determine file extension based on runtime
        let file_extension = match request.runtime.as_str() {
            "rs" | "rust" => "rs",
            _ => {
                return Ok(CreateLambdaResponse {
                    success: false,
                    message: format!("Unsupported runtime: {}", request.runtime),
                    function_name: request.function_name,
                    source_path: None,
                    wasm_path: None,
                    wasm_size_bytes: None,
                    compilation_time_ms: None,
                });
            }
        };

        // Create source directory if it doesn't exist
        let source_dir = self.source_dir();
        if let Err(e) = fs::create_dir_all(&source_dir) {
            tracing::error!(
                function_name = %request.function_name,
                error = %e,
                "Failed to create source directory"
            );
            return Ok(CreateLambdaResponse {
                success: false,
                message: format!("Failed to create source directory: {}", e),
                function_name: request.function_name,
                source_path: None,
                wasm_path: None,
                wasm_size_bytes: None,
                compilation_time_ms: None,
            });
        }

        // Create WASM directory if it doesn't exist
        let wasm_dir = self.wasm_dir();
        if let Err(e) = fs::create_dir_all(&wasm_dir) {
            tracing::error!(
                function_name = %request.function_name,
                error = %e,
                "Failed to create WASM directory"
            );
            return Ok(CreateLambdaResponse {
                success: false,
                message: format!("Failed to create WASM directory: {}", e),
                function_name: request.function_name,
                source_path: None,
                wasm_path: None,
                wasm_size_bytes: None,
                compilation_time_ms: None,
            });
        }

        // Save source code to file
        let source_file_name = format!("{}.{}", request.function_name, file_extension);
        let source_path = source_dir.join(&source_file_name);

        if let Err(e) = fs::write(&source_path, &request.source_code) {
            tracing::error!(
                function_name = %request.function_name,
                source_path = %source_path.display(),
                error = %e,
                "Failed to save source code"
            );
            return Ok(CreateLambdaResponse {
                success: false,
                message: format!("Failed to save source code: {}", e),
                function_name: request.function_name,
                source_path: None,
                wasm_path: None,
                wasm_size_bytes: None,
                compilation_time_ms: None,
            });
        }

        tracing::info!(
            function_name = %request.function_name,
            source_path = %source_path.display(),
            "Source code saved successfully"
        );

        // Attempt to compile to WASM
        let (wasm_path, wasm_size_bytes, compilation_success) =
            match self.compile(&request.source_code, &request.runtime).await {
                Ok(wasm_bytes) => {
                    // Save compiled WASM
                    match self
                        .save_compiled_wasm(&request.function_name, &wasm_bytes)
                        .await
                    {
                        Ok(wasm_file_path) => {
                            tracing::info!(
                                function_name = %request.function_name,
                                wasm_path = %wasm_file_path.display(),
                                wasm_size = wasm_bytes.len(),
                                "WASM compilation and save successful"
                            );
                            (
                                Some(wasm_file_path.to_string_lossy().to_string()),
                                Some(wasm_bytes.len() as u64),
                                true,
                            )
                        }
                        Err(e) => {
                            tracing::warn!(
                                function_name = %request.function_name,
                                error = %e,
                                "Failed to save compiled WASM, but source was saved"
                            );
                            (None, None, false)
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        function_name = %request.function_name,
                        error = %e,
                        "Failed to compile source to WASM, but source was saved"
                    );
                    (None, None, false)
                }
            };

        let compilation_time_ms = start_time.elapsed().as_millis() as u64;

        let (success, message) = if compilation_success {
            (
                true,
                format!(
                    "Lambda '{}' created and compiled successfully",
                    request.function_name
                ),
            )
        } else {
            (
                true, // Still success since source was saved
                format!(
                    "Lambda '{}' created successfully, but compilation failed. Check logs for details.",
                    request.function_name
                ),
            )
        };

        Ok(CreateLambdaResponse {
            success,
            message,
            function_name: request.function_name,
            source_path: Some(source_path.to_string_lossy().to_string()),
            wasm_path,
            wasm_size_bytes,
            compilation_time_ms: Some(compilation_time_ms),
        })
    }

    /// Get a lambda function by name
    async fn get_by_name(&self, name: &str) -> AppResult<Option<Lambda>> {
        tracing::info!(
            lambda_name = %name,
            "Getting lambda function by name"
        );

        // Basic validation
        if name.trim().is_empty() {
            return Err(AppError::validation("Lambda name cannot be empty"));
        }

        // Validate lambda name pattern
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(AppError::validation(
                "Lambda name must contain only letters, numbers, and underscores",
            ));
        }

        let source_dir = self.source_dir();
        let wasm_dir = self.wasm_dir();

        // Look for source file (mandatory)
        let mut source_path = None;
        let mut source_code = None;

        // Try different extensions for the source file
        for extension in ["rs", "rust"] {
            let path = source_dir.join(format!("{}.{}", name, extension));
            if path.exists() {
                source_path = Some(path.clone());
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        source_code = Some(content);
                        break;
                    }
                    Err(e) => {
                        tracing::error!(
                            lambda_name = %name,
                            source_path = %path.display(),
                            error = %e,
                            "Failed to read source file"
                        );
                        return Err(AppError::validation(&format!(
                            "Failed to read source file: {}",
                            e
                        )));
                    }
                }
            }
        }

        // If no source file found, lambda doesn't exist
        let source_code = match source_code {
            Some(code) => code,
            None => {
                tracing::warn!(
                    lambda_name = %name,
                    "Lambda function not found - no source file exists"
                );
                return Ok(None);
            }
        };

        // Try to load WASM file (optional)
        let wasm_path = wasm_dir.join(format!("{}.wasm", name));
        let wasm_bytes = if wasm_path.exists() {
            match fs::read(&wasm_path) {
                Ok(bytes) => {
                    tracing::debug!(
                        lambda_name = %name,
                        wasm_size = bytes.len(),
                        "Found compiled WASM file"
                    );
                    Some(bytes)
                }
                Err(e) => {
                    tracing::warn!(
                        lambda_name = %name,
                        wasm_path = %wasm_path.display(),
                        error = %e,
                        "WASM file exists but failed to read - treating as None"
                    );
                    None
                }
            }
        } else {
            tracing::debug!(
                lambda_name = %name,
                "No WASM file found - lambda may not be compiled yet"
            );
            None
        };

        // Determine runtime from source file extension
        let runtime = if let Some(ref path) = source_path {
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("rs") | Some("rust") => "rust".to_string(),
                _ => "unknown".to_string(),
            }
        } else {
            "unknown".to_string()
        };

        // Determine status based on WASM availability
        let has_wasm = wasm_bytes.is_some();
        let status = if has_wasm {
            LambdaStatus::Active
        } else {
            LambdaStatus::Inactive
        };

        // Get file metadata for timestamps
        let metadata = source_path.as_ref().and_then(|path| path.metadata().ok());

        let (created_at, updated_at) = if let Some(meta) = metadata {
            let created = meta
                .created()
                .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);
            let modified = meta
                .modified()
                .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);

            let created_dt = chrono::DateTime::from_timestamp(
                created
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
                0,
            )
            .unwrap_or_else(chrono::Utc::now);

            let modified_dt = chrono::DateTime::from_timestamp(
                modified
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
                0,
            )
            .unwrap_or_else(chrono::Utc::now);

            (created_dt, modified_dt)
        } else {
            let now = chrono::Utc::now();
            (now, now)
        };

        // Create Lambda object
        let lambda = Lambda {
            id: Uuid::new_v4(), // Generate a new UUID for this instance
            name: name.to_string(),
            description: None, // TODO: Extract description from comments or metadata
            runtime,
            memory_mb: 128,      // Default values - could be extracted from metadata
            timeout_seconds: 30, // Default values - could be extracted from metadata
            source_code,
            wasm_bytes,
            environment_vars: std::collections::HashMap::new(),
            created_at,
            updated_at,
            status: status.clone(),
        };

        tracing::info!(
            lambda_name = %name,
            has_wasm = has_wasm,
            status = ?status,
            "Successfully retrieved lambda function"
        );

        Ok(Some(lambda))
    }

    /// Delete a lambda function by name
    async fn delete(&self, lambda_name: &str) -> AppResult<()> {
        tracing::info!(
            lambda_name = %lambda_name,
            "Deleting lambda function"
        );

        // Validate function name (basic validation for file system safety)
        if lambda_name.trim().is_empty() {
            return Err(AppError::validation("Function name cannot be empty"));
        }

        // Check for valid identifier pattern
        if !lambda_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(AppError::validation(
                "Function name must contain only letters, numbers, and underscores",
            ));
        }

        let source_dir = self.source_dir();
        let wasm_dir = self.wasm_dir();

        // Find and delete source file
        let possible_extensions = ["rs", "rust"];
        let mut source_deleted = false;

        for ext in &possible_extensions {
            let source_path = source_dir.join(format!("{}.{}", lambda_name, ext));
            if source_path.exists() {
                if let Err(e) = fs::remove_file(&source_path) {
                    tracing::error!(
                        lambda_name = %lambda_name,
                        source_path = %source_path.display(),
                        error = %e,
                        "Failed to delete source file"
                    );
                    return Err(AppError::internal(&format!(
                        "Failed to delete source file: {}",
                        e
                    )));
                }
                tracing::info!(
                    lambda_name = %lambda_name,
                    source_path = %source_path.display(),
                    "Source file deleted successfully"
                );
                source_deleted = true;
                break;
            }
        }

        // Delete WASM file if it exists
        let wasm_path = wasm_dir.join(format!("{}.wasm", lambda_name));
        if wasm_path.exists() {
            if let Err(e) = fs::remove_file(&wasm_path) {
                tracing::error!(
                    lambda_name = %lambda_name,
                    wasm_path = %wasm_path.display(),
                    error = %e,
                    "Failed to delete WASM file"
                );
                return Err(AppError::internal(&format!(
                    "Failed to delete WASM file: {}",
                    e
                )));
            }
            tracing::info!(
                lambda_name = %lambda_name,
                wasm_path = %wasm_path.display(),
                "WASM file deleted successfully"
            );
        }

        if !source_deleted {
            return Err(AppError::not_found(&format!(
                "Lambda '{}' not found. No source file exists for this lambda.",
                lambda_name
            )));
        }

        tracing::info!(
            lambda_name = %lambda_name,
            "Lambda function deleted successfully"
        );

        Ok(())
    }

    /// Update an existing lambda function
    #[instrument(
        skip_all,
        fields(repository = "lambda_storage", operation = "update_lambda")
    )]
    async fn update(&self, lambda_name: &str, request: UpdateLambdaRequest) -> AppResult<()> {
        tracing::info!(
            lambda_name = %lambda_name,
            "Updating lambda function"
        );

        // Basic validation
        if lambda_name.trim().is_empty() {
            return Err(AppError::validation("Lambda name cannot be empty"));
        }

        // Validate lambda name pattern
        if !lambda_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(AppError::validation(
                "Lambda name must contain only letters, numbers, and underscores",
            ));
        }

        let source_dir = self.source_dir();
        let wasm_dir = self.wasm_dir();

        // Find existing source file
        let mut source_path = None;
        for extension in ["rs", "rust"] {
            let path = source_dir.join(format!("{}.{}", lambda_name, extension));
            if path.exists() {
                source_path = Some(path);
                break;
            }
        }

        let source_path = source_path.ok_or_else(|| {
            AppError::not_found(&format!("Lambda function '{}' not found", lambda_name))
        })?;

        // Update source code if provided
        if let Some(new_source_code) = request.source_code.as_ref() {
            tracing::debug!(
                lambda_name = %lambda_name,
                source_path = %source_path.display(),
                "Updating source code"
            );

            if let Err(e) = fs::write(&source_path, new_source_code) {
                tracing::error!(
                    lambda_name = %lambda_name,
                    source_path = %source_path.display(),
                    error = %e,
                    "Failed to write updated source code"
                );
                return Err(AppError::validation(&format!(
                    "Failed to update source code: {}",
                    e
                )));
            }

            tracing::info!(
                lambda_name = %lambda_name,
                source_size = new_source_code.len(),
                "Source code updated successfully"
            );

            // If WASM file exists, delete it to force recompilation
            let wasm_path = wasm_dir.join(format!("{}.wasm", lambda_name));
            if wasm_path.exists() {
                if let Err(e) = fs::remove_file(&wasm_path) {
                    tracing::warn!(
                        lambda_name = %lambda_name,
                        wasm_path = %wasm_path.display(),
                        error = %e,
                        "Failed to remove existing WASM file - recompilation may be needed"
                    );
                } else {
                    tracing::info!(
                        lambda_name = %lambda_name,
                        "Existing WASM file removed - recompilation required"
                    );
                }
            }
        }

        // Update other metadata if provided (currently no-op as we don't store metadata separately)
        // TODO: If we add metadata storage (JSON/TOML files), update them here

        tracing::info!(
            lambda_name = %lambda_name,
            "Lambda function updated successfully"
        );

        Ok(())
    }

    /// Get lambda execution metrics
    fn get_metrics(&self) -> &LambdasMetrics {
        &self.metrics
    }
}

/// Host function for HTTP requests from WASM
fn host_http_request(
    mut env: wasmer::FunctionEnvMut<WasmEnv>,
    req_ptr: u32,
    req_len: u32,
    out_ptr_ptr: u32,
    out_len_ptr: u32,
) -> i32 {
    tracing::debug!("HTTP request from WASM lambda");

    // Get memory from the environment
    let memory = match env.data().memory.clone() {
        Some(mem) => mem,
        None => {
            tracing::error!("Failed to get WASM memory in HTTP host function");
            return -1;
        }
    };

    // Read request from WASM memory
    let request_bytes = {
        let view = memory.view(&env);
        let mut bytes = Vec::with_capacity(req_len as usize);
        for i in 0..req_len {
            match view.read_u8((req_ptr + i) as u64) {
                Ok(byte) => bytes.push(byte),
                Err(e) => {
                    tracing::error!("Failed to read request from WASM memory: {}", e);
                    return -1;
                }
            }
        }
        bytes
    };

    // Parse request JSON
    let request: std::collections::HashMap<String, serde_json::Value> =
        match serde_json::from_slice(&request_bytes) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to parse HTTP request JSON: {}", e);
                return -1;
            }
        };

    // Extract request fields
    let method = request
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET");
    let url = match request.get("url").and_then(|v| v.as_str()) {
        Some(url) => url,
        None => {
            tracing::error!("Request missing URL field");
            return -1;
        }
    };

    tracing::info!(method = %method, url = %url, "Making real HTTP request from lambda");

    // Clone data for the blocking thread
    let method = method.to_string();
    let url = url.to_string();
    let headers = request.get("headers").and_then(|v| v.as_object()).cloned();
    let body = request
        .get("body")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Execute HTTP request in a blocking thread to avoid tokio runtime conflicts
    let http_result = std::thread::spawn(move || {
        // Create HTTP client in the blocking thread
        let client = reqwest::blocking::Client::new();
        let mut req_builder = match method.as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            "PATCH" => client.patch(&url),
            "HEAD" => client.head(&url),
            "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url),
            _ => client.get(&url), // Default to GET
        };

        // Add headers if present
        if let Some(headers) = headers {
            for (key, value) in headers {
                if let Some(value_str) = value.as_str() {
                    req_builder = req_builder.header(key, value_str);
                }
            }
        }

        // Add body if present
        if let Some(body) = body {
            req_builder = req_builder.body(body);
        }

        // Execute HTTP request
        let response = req_builder.send()?;
        let status = response.status().as_u16();
        let body = response.text()?;

        Ok::<(u16, String), reqwest::Error>((status, body))
    });

    let (status, body) = match http_result.join() {
        Ok(Ok((status, body))) => (status, body),
        Ok(Err(e)) => {
            tracing::error!("HTTP request failed: {}", e);
            return -1;
        }
        Err(_) => {
            tracing::error!("HTTP request thread panicked");
            return -1;
        }
    };

    // Create response JSON
    let response_json = serde_json::json!({
        "status": status,
        "body": body
    });

    let response_bytes = match serde_json::to_vec(&response_json) {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to serialize response: {}", e);
            return -1;
        }
    };

    tracing::info!(status = %status, response_size = response_bytes.len(), "HTTP request completed");

    // Use a simple approach: allocate in existing WASM memory and return pointer
    let response_len = response_bytes.len();

    // Get current memory size and use it as allocation point
    let memory_view = memory.view(&env);
    let current_size = memory_view.data_size() as usize;
    let alloc_ptr = current_size as u32;

    // Grow memory if needed
    let needed_pages = ((current_size + response_len + 65535) / 65536) - (current_size / 65536);
    if needed_pages > 0 {
        if let Err(e) = memory.grow(&mut env, needed_pages as u32) {
            tracing::error!("Failed to grow WASM memory: {}", e);
            return -1;
        }
    }

    // Write response data to allocated memory
    let memory_view = memory.view(&env);
    for (i, &byte) in response_bytes.iter().enumerate() {
        if let Err(e) = memory_view.write_u8((alloc_ptr + i as u32) as u64, byte) {
            tracing::error!("Failed to write response to WASM memory: {}", e);
            return -1;
        }
    }

    // Write response pointer and length back to WASM using byte-by-byte approach
    let alloc_ptr_bytes = alloc_ptr.to_le_bytes();
    for (i, &byte) in alloc_ptr_bytes.iter().enumerate() {
        if let Err(e) = memory_view.write_u8((out_ptr_ptr + i as u32) as u64, byte) {
            tracing::error!("Failed to write response pointer: {}", e);
            return -1;
        }
    }

    let response_len_bytes = (response_len as u32).to_le_bytes();
    for (i, &byte) in response_len_bytes.iter().enumerate() {
        if let Err(e) = memory_view.write_u8((out_len_ptr + i as u32) as u64, byte) {
            tracing::error!("Failed to write response length: {}", e);
            return -1;
        }
    }

    0 // Success
}

/// Environment data for WASM functions
#[derive(Clone)]
struct WasmEnv {
    memory: Option<wasmer::Memory>,
}

impl WasmEnv {
    fn new() -> Self {
        Self { memory: None }
    }

    fn with_memory(memory: wasmer::Memory) -> Self {
        Self {
            memory: Some(memory),
        }
    }
}

impl LambdaStorage {
    /// Execute WASM bytecode using wasmer with structured data support
    #[instrument(
        skip_all,
        fields(infrastructure = "lambda_storage", operation = "execute_wasm")
    )]
    async fn execute_wasm(&self, wasm_bytes: &[u8], input_data: &[u8]) -> AppResult<Vec<u8>> {
        use wasmer::{Engine, Instance, Module, Store, Value};

        tracing::info!("Executing WASM lambda with structured data support");

        // Create wasmer engine and store using universal engine (avoids unwind issues)
        let engine = Engine::default();
        let mut store = Store::new(engine);

        // Compile WASM module
        let module = Module::new(&store, wasm_bytes)
            .map_err(|e| AppError::validation(&format!("Failed to compile WASM module: {}", e)))?;

        // Create function environment for host functions
        let env = wasmer::FunctionEnv::new(&mut store, WasmEnv::new());

        // Create instance first to get memory
        let imports = wasmer::imports! {
            "env" => {
                "host_http_request" => wasmer::Function::new_typed_with_env(&mut store, &env, host_http_request),
                "wasm_free" => wasmer::Function::new_typed(&mut store, |ptr: u32, size: u32| {
                    // This is called by WASM to free host-allocated memory
                    // For now, we'll just log it since we're using simple allocation
                    tracing::debug!(ptr = ptr, size = size, "WASM requested to free host memory");
                }),
            }
        };

        // Create instance
        let instance = Instance::new(&mut store, &module, &imports)
            .map_err(|e| AppError::validation(&format!("Failed to create WASM instance: {}", e)))?;

        // Get the exported handler function
        let handler = instance
            .exports
            .get_function("handler")
            .map_err(|e| AppError::validation(&format!("Handler function not found: {}", e)))?;

        // Get WASM memory for input/output operations
        let memory = instance
            .exports
            .get_memory("memory")
            .map_err(|_| AppError::validation("WASM memory not found"))?;

        // Update the environment with memory so host functions can access it
        env.as_mut(&mut store).memory = Some(memory.clone());

        // Allocate memory for input data
        let input_len = input_data.len();
        let input_ptr = {
            // Get current memory size in pages (64KB each)
            let memory_view = memory.view(&store);
            let current_size = memory_view.data_size() as usize;

            // Use current memory end as input pointer
            let ptr = current_size;

            // Grow memory if needed to accommodate input
            let needed_pages = ((ptr + input_len + 65535) / 65536) - (current_size / 65536);
            if needed_pages > 0 {
                memory.grow(&mut store, needed_pages as u32).map_err(|e| {
                    AppError::validation(&format!("Failed to grow WASM memory: {}", e))
                })?;
            }

            ptr
        };

        // Write input data to WASM memory
        {
            let memory_view = memory.view(&store);
            for (i, &byte) in input_data.iter().enumerate() {
                memory_view
                    .write_u8((input_ptr + i) as u64, byte)
                    .map_err(|e| {
                        AppError::validation(&format!(
                            "Failed to write input to WASM memory: {}",
                            e
                        ))
                    })?;
            }
        }

        // Allocate memory for output buffer (max 1MB)
        let max_output_len = 1024 * 1024; // 1MB max output
        let output_ptr = {
            let memory_view = memory.view(&store);
            let current_size = memory_view.data_size() as usize;
            let ptr = current_size;

            // Grow memory if needed to accommodate output buffer
            let needed_pages = ((ptr + max_output_len + 65535) / 65536) - (current_size / 65536);
            if needed_pages > 0 {
                memory.grow(&mut store, needed_pages as u32).map_err(|e| {
                    AppError::validation(&format!("Failed to grow WASM memory for output: {}", e))
                })?;
            }

            ptr
        };

        tracing::debug!(
            input_len = input_len,
            input_ptr = input_ptr,
            output_ptr = output_ptr,
            max_output_len = max_output_len,
            "Calling handler with pointer-based interface"
        );

        // Call the handler function with pointer-based interface
        let results = handler
            .call(
                &mut store,
                &[
                    Value::I32(input_ptr as i32),      // ptr_in
                    Value::I32(input_len as i32),      // len_in
                    Value::I32(output_ptr as i32),     // ptr_out
                    Value::I32(max_output_len as i32), // max_out_len
                ],
            )
            .map_err(|e| AppError::validation(&format!("WASM function call failed: {}", e)))?;

        if results.is_empty() {
            return Err(AppError::validation("Handler function returned no results"));
        }

        // Extract the actual output length from results
        let actual_output_len = match results[0] {
            Value::I32(len) => len as usize,
            _ => {
                return Err(AppError::validation(
                    "Handler function must return output length (i32)",
                ));
            }
        };

        if actual_output_len > max_output_len {
            return Err(AppError::validation(&format!(
                "Handler returned invalid output length: {} > {}",
                actual_output_len, max_output_len
            )));
        }

        // Read the output data from WASM memory
        let output_data = {
            let memory_view = memory.view(&store);
            let mut output_bytes = Vec::with_capacity(actual_output_len);

            for i in 0..actual_output_len {
                let byte = memory_view.read_u8((output_ptr + i) as u64).map_err(|e| {
                    AppError::validation(&format!("Failed to read output from WASM memory: {}", e))
                })?;
                output_bytes.push(byte);
            }

            output_bytes
        };

        tracing::debug!(
            output_len = actual_output_len,
            "Successfully received output from lambda"
        );

        Ok(output_data)
    }
}
