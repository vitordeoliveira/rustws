//! Lambda repository trait for data access and WASM execution

use uuid::Uuid; // For future methods

use super::dto::{
    LambdaSummary,
    // Future DTOs for commented methods:
    // CreateLambdaRequest, ExecuteLambdaRequest, ExecuteLambdaResponse,
    // Lambda, UpdateLambdaRequest,
};
use crate::error_handling::types::AppResult;

/// Lambda repository trait for managing WASM-based serverless functions
pub trait LambdaRepository: Send + Sync {
    /// Get all lambda functions
    async fn get_all(&self) -> AppResult<Vec<LambdaSummary>>;

    /// Compile source code to WASM
    async fn compile(&self, source_code: &str, runtime: &str) -> AppResult<Vec<u8>>;

    // /// Get a lambda function by ID
    // async fn get_by_id(&self, id: Uuid) -> AppResult<Option<Lambda>>;

    // /// Get a lambda function by name
    // async fn get_by_name(&self, name: &str) -> AppResult<Option<Lambda>>;

    // /// Execute a lambda function
    // async fn execute(&self, request: ExecuteLambdaRequest) -> AppResult<ExecuteLambdaResponse>;

    // /// Create a new lambda function
    // async fn create(&self, request: CreateLambdaRequest) -> AppResult<Lambda>;

    // /// Update an existing lambda function
    // async fn update(&self, id: Uuid, request: UpdateLambdaRequest) -> AppResult<Lambda>;

    // /// Delete a lambda function
    // async fn delete(&self, id: Uuid) -> AppResult<()>;
}
