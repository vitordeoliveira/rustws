//! Lambda repository trait for data access and WASM execution

use std::path::PathBuf;

use super::dto::{
    CreateLambdaRequest,
    CreateLambdaResponse,
    ExecuteLambdaRequest,
    ExecuteLambdaResponse,
    // Future DTOs for commented methods:
    // Lambda, UpdateLambdaRequest,
    LambdaSummary,
};
use crate::error_handling::types::AppResult;

/// Lambda repository trait for managing WASM-based serverless functions
pub trait LambdaRepository: Send + Sync {
    /// Get all lambda functions
    async fn get_all(&self) -> AppResult<Vec<LambdaSummary>>;

    /// Compile source code to WASM
    async fn compile(&self, source_code: &str, runtime: &str) -> AppResult<Vec<u8>>;

    /// Compile source file to WASM
    async fn compile_source_file(&self, lambda_name: &str) -> AppResult<Vec<u8>>;

    /// Save compiled WASM bytes
    async fn save_compiled_wasm(&self, lambda_name: &str, wasm_bytes: &[u8]) -> AppResult<PathBuf>;

    /// Execute a lambda function
    async fn execute(&self, request: ExecuteLambdaRequest) -> AppResult<ExecuteLambdaResponse>;

    // /// Get a lambda function by ID
    // async fn get_by_id(&self, id: Uuid) -> AppResult<Option<Lambda>>;

    // /// Get a lambda function by name
    // async fn get_by_name(&self, name: &str) -> AppResult<Option<Lambda>>;

    /// Create a new lambda function
    async fn create(&self, request: CreateLambdaRequest) -> AppResult<CreateLambdaResponse>;

    // /// Update an existing lambda function
    // async fn update(&self, id: Uuid, request: UpdateLambdaRequest) -> AppResult<Lambda>;

    // /// Delete a lambda function
    // async fn delete(&self, id: Uuid) -> AppResult<()>;
}
