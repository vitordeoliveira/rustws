//! Lambda repository implementation
//!
//! This module implements the LambdaRepository trait for file-based storage.

use std::fs;
use tracing::instrument;
use uuid::Uuid;

use crate::business_logic::lambdas::{
    CreateLambdaRequest, CreateLambdaResponse, ExecuteLambdaRequest, ExecuteLambdaResponse, Lambda,
    LambdaRepository, LambdaStatus, LambdaSummary, UpdateLambdaRequest,
};
use crate::error_handling::types::{AppError, AppResult};
use super::storage::LambdaStorage;
use super::wasm_runtime;

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
        self.compile(source_code, runtime).await
    }

    /// Compile source file to WASM
    async fn compile_source_file(&self, lambda_name: &str) -> AppResult<Vec<u8>> {
        self.compile_source_file(lambda_name).await
    }

    /// Save compiled WASM bytes
    async fn save_compiled_wasm(&self, lambda_name: &str, wasm_bytes: &[u8]) -> AppResult<std::path::PathBuf> {
        self.save_compiled_wasm(lambda_name, wasm_bytes).await
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
        let result = match wasm_runtime::execute_wasm(&wasm_bytes, &request.input_data).await {
            Ok(output) => ExecuteLambdaResponse::Success {
                output_data: output,
            },
            Err(e) => ExecuteLambdaResponse::Failed {
                error_message: format!("WASM execution failed: {}", e),
            },
        };

        // Record execution in metrics ledger
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let ledger = self.get_metrics_ledger_mut();

        match &result {
            ExecuteLambdaResponse::Success { .. } => {
                ledger.record_success(lambda_name.clone(), execution_time_ms);
                tracing::info!(
                    lambda_name = %lambda_name,
                    execution_time_ms = execution_time_ms,
                    "Lambda execution successful"
                );
            }
            ExecuteLambdaResponse::Failed { .. } => {
                ledger.record_failure(lambda_name.clone(), execution_time_ms);
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
    fn get_metrics(&self) -> &super::metrics::LambdasMetrics {
        // Note: This method signature requires a reference, but we calculate on-demand
        // This is a bit of a hack - consider changing the trait to return owned values
        Box::leak(Box::new(LambdaStorage::get_metrics(self)))
    }
}
