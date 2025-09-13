//! Lambda service for business logic orchestration

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use std::time::Instant;

use super::dto::{
    CompileLambdaRequest, CompileLambdaResponse, CreateLambdaRequest, CreateLambdaResponse,
    ExecuteLambdaRequest, ExecuteLambdaResponse, LambdaSummary,
};
use super::repository::LambdaRepository;
use crate::error_handling::types::{AppError, AppResult};
use crate::infrastructure::lambdas::LambdaStorage;
use crate::state::AppState;

/// Lambda service for managing lambda functions
pub(crate) struct LambdasService<R> {
    repository: R,
}

impl<R> LambdasService<R>
where
    R: LambdaRepository,
{
    /// Create a new lambda service with the given repository
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Get all lambda functions
    pub async fn get_all(&self) -> AppResult<Vec<LambdaSummary>> {
        self.repository.get_all().await
    }

    /// Compile a lambda function from source to WASM
    pub async fn compile(&self, request: CompileLambdaRequest) -> AppResult<CompileLambdaResponse> {
        let start_time = Instant::now();

        match self
            .repository
            .compile_source_file(&request.lambda_name)
            .await
        {
            Ok(wasm_bytes) => {
                // Save the compiled WASM
                let wasm_path = self
                    .repository
                    .save_compiled_wasm(&request.lambda_name, &wasm_bytes)
                    .await?;

                let compilation_time = start_time.elapsed().as_millis() as u64;

                Ok(CompileLambdaResponse {
                    success: true,
                    lambda_name: request.lambda_name.clone(),
                    wasm_size_bytes: Some(wasm_bytes.len()),
                    wasm_path: Some(wasm_path.to_string_lossy().to_string()),
                    compilation_time_ms: compilation_time,
                    message: format!(
                        "Successfully compiled '{}' to WASM ({} bytes)",
                        request.lambda_name,
                        wasm_bytes.len()
                    ),
                })
            }
            Err(e) => {
                let compilation_time = start_time.elapsed().as_millis() as u64;

                Ok(CompileLambdaResponse {
                    success: false,
                    lambda_name: request.lambda_name.clone(),
                    wasm_size_bytes: None,
                    wasm_path: None,
                    compilation_time_ms: compilation_time,
                    message: format!("Compilation failed: {}", e),
                })
            }
        }
    }

    /// Execute a lambda function
    pub async fn execute(&self, request: ExecuteLambdaRequest) -> AppResult<ExecuteLambdaResponse> {
        self.repository.execute(request).await
    }

    /// Create a new lambda function
    pub async fn create(&self, request: CreateLambdaRequest) -> AppResult<CreateLambdaResponse> {
        self.repository.create(request).await
    }

    /// Delete a lambda function by name
    pub async fn delete(&self, lambda_name: &str) -> AppResult<()> {
        self.repository.delete(lambda_name).await
    }
}

/// Extract LambdasService directly from request using FromRequestParts
impl FromRequestParts<AppState> for LambdasService<LambdaStorage> {
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let lambda_storage = LambdaStorage::new();
        Ok(Self::new(lambda_storage))
    }
}
