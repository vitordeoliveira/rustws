//! Lambda infrastructure implementation
//!
//! This module provides the concrete implementation of lambda repository

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

use crate::business_logic::lambdas::{
    ExecuteLambdaRequest, ExecuteLambdaResponse, LambdaRepository, LambdaStatus, LambdaSummary,
};
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

    /// Get path to WASM files directory
    fn wasm_dir(&self) -> PathBuf {
        self.base_path.join("wasm")
    }

    /// Compile Rust source code to WASM
    async fn compile_rust(&self, source_code: &str) -> AppResult<Vec<u8>> {
        // Create temporary directory for compilation
        let temp_dir = std::env::temp_dir().join(format!("lambda_compile_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir)
            .map_err(|e| AppError::internal(&format!("Failed to create temp dir: {}", e)))?;

        // Write source code to temp file
        let source_path = temp_dir.join("main.rs");
        fs::write(&source_path, source_code)
            .map_err(|e| AppError::internal(&format!("Failed to write source: {}", e)))?;

        // Compile with rustc to WASM using proper flags
        let wasm_output = temp_dir.join("output.wasm");
        let output = Command::new("rustc")
            .args([
                "--target",
                "wasm32-unknown-unknown",
                "--crate-type",
                "cdylib",
                "-C",
                "opt-level=s", // Optimize for size
                "-C",
                "lto=yes", // Enable link-time optimization
                "-C",
                "panic=abort", // Use abort instead of unwind for WASM
                "-C",
                "strip=symbols", // Strip debug symbols
                "--edition",
                "2021", // Use Rust 2021 edition
                source_path.to_str().unwrap(),
                "-o",
                wasm_output.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| AppError::internal(&format!("Failed to run rustc: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            let _ = fs::remove_dir_all(&temp_dir); // Cleanup on error
            return Err(AppError::validation(&format!(
                "Rust compilation failed: {}",
                error
            )));
        }

        // Read compiled WASM bytes
        let wasm_bytes = fs::read(&wasm_output)
            .map_err(|e| AppError::internal(&format!("Failed to read WASM output: {}", e)))?;

        // Cleanup temporary directory
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(wasm_bytes)
    }

    /// Check if rustc with WASM target is available
    fn check_rust_wasm_toolchain(&self) -> AppResult<()> {
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
    async fn execute(&self, request: ExecuteLambdaRequest) -> AppResult<ExecuteLambdaResponse> {
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
        match self.execute_wasm(&wasm_bytes, &request.input_data).await {
            Ok(output) => Ok(ExecuteLambdaResponse::Success {
                output_data: output,
            }),
            Err(e) => Ok(ExecuteLambdaResponse::Failed {
                error_message: format!("WASM execution failed: {}", e),
            }),
        }
    }
}

impl LambdaStorage {
    /// Execute WASM bytecode using wasmer
    async fn execute_wasm(&self, wasm_bytes: &[u8], input_data: &[u8]) -> AppResult<Vec<u8>> {
        use wasmer::{Engine, Instance, Module, Store};

        // Create wasmer engine and store using universal engine (avoids unwind issues)
        let engine = Engine::default();
        let mut store = Store::new(engine);

        // Compile WASM module
        let module = Module::new(&store, wasm_bytes)
            .map_err(|e| AppError::validation(&format!("Failed to compile WASM module: {}", e)))?;

        // Create instance
        let instance = Instance::new(&mut store, &module, &wasmer::imports! {})
            .map_err(|e| AppError::validation(&format!("Failed to create WASM instance: {}", e)))?;

        // Get the exported handler function
        let handler = instance
            .exports
            .get_function("handler")
            .map_err(|e| AppError::validation(&format!("Handler function not found: {}", e)))?;

        // For now, we'll implement a simple execution that calls the handler
        // In the future, we can implement memory passing for input_data
        let _input_data = input_data; // TODO: Pass input data to WASM function

        // Call the handler function (assuming it takes no params for now)
        let results = handler
            .call(&mut store, &[])
            .map_err(|e| AppError::validation(&format!("WASM function call failed: {}", e)))?;

        // For now, return empty output data
        // TODO: Extract output data from WASM memory or return values
        let output_data = if results.is_empty() {
            b"Hello from WASM!".to_vec()
        } else {
            // Convert first result to bytes if it's a number
            match results[0].ty() {
                wasmer::Type::I32 => {
                    let val = results[0].unwrap_i32();
                    val.to_le_bytes().to_vec()
                }
                wasmer::Type::I64 => {
                    let val = results[0].unwrap_i64();
                    val.to_le_bytes().to_vec()
                }
                _ => b"Unsupported return type".to_vec(),
            }
        };

        Ok(output_data)
    }
}
