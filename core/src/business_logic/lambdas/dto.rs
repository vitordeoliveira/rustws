//! Lambda Data Transfer Objects

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lambda function metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lambda {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub runtime: String,
    pub memory_mb: u32,
    pub timeout_seconds: u32,
    pub wasm_bytes: Vec<u8>,
    pub environment_vars: std::collections::HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub status: LambdaStatus,
}

/// Lambda function status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LambdaStatus {
    Active,
    Inactive,
    Error,
}

/// Request to create a new lambda function
#[derive(Debug, Deserialize)]
pub struct CreateLambdaRequest {
    pub name: String,
    pub description: Option<String>,
    pub runtime: String,
    pub memory_mb: u32,
    pub timeout_seconds: u32,
    pub wasm_bytes: Vec<u8>,
    pub environment_vars: Option<std::collections::HashMap<String, String>>,
}

/// Request to update an existing lambda function
#[derive(Debug, Deserialize)]
pub struct UpdateLambdaRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub memory_mb: Option<u32>,
    pub timeout_seconds: Option<u32>,
    pub wasm_bytes: Option<Vec<u8>>,
    pub environment_vars: Option<std::collections::HashMap<String, String>>,
    pub status: Option<LambdaStatus>,
}

/// Request to execute a lambda function
#[derive(Debug, Deserialize)]
pub struct ExecuteLambdaRequest {
    pub function_name: Option<String>,
    pub input_data: Vec<u8>,
}

/// Response from lambda execution
#[derive(Debug, Serialize)]
pub struct ExecuteLambdaResponse {
    pub output_data: Vec<u8>,
    pub execution_time_ms: u64,
    pub memory_used_mb: u32,
    pub status: ExecutionStatus,
    pub error_message: Option<String>,
}

/// Lambda execution status
#[derive(Debug, Serialize)]
pub enum ExecutionStatus {
    Success,
    Timeout,
    MemoryExceeded,
    RuntimeError,
}

/// Lambda function summary for listing
#[derive(Debug, Serialize)]
pub struct LambdaSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub runtime: String,
    pub memory_mb: u32,
    pub timeout_seconds: u32,
    pub status: LambdaStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Lambda> for LambdaSummary {
    fn from(lambda: Lambda) -> Self {
        Self {
            id: lambda.id,
            name: lambda.name,
            description: lambda.description,
            runtime: lambda.runtime,
            memory_mb: lambda.memory_mb,
            timeout_seconds: lambda.timeout_seconds,
            status: lambda.status,
            created_at: lambda.created_at,
            updated_at: lambda.updated_at,
        }
    }
}
