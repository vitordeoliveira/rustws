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
    pub source_code: String,
    pub wasm_bytes: Option<Vec<u8>>,
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

/// Request to update an existing lambda function
#[derive(Debug, Deserialize)]
pub struct UpdateLambdaRequest {
    pub source_code: Option<String>,
}

/// Request to execute a lambda function
#[derive(Debug, Deserialize)]
pub struct ExecuteLambdaRequest {
    pub function_name: Option<String>,
    pub input_data: Vec<u8>,
}

/// Response from lambda execution
#[derive(Debug, Serialize)]
pub enum ExecuteLambdaResponse {
    Success {
        output_data: Vec<u8>,
        execution_time_ms: u64,
        execution_timestamp: chrono::DateTime<chrono::Utc>,
    },
    Failed {
        error_message: String,
        execution_time_ms: u64,
        execution_timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Request to create a new lambda function
#[derive(Debug, Deserialize)]
pub struct CreateLambdaRequest {
    /// Function name (must be valid identifier)
    #[serde(rename = "functionName")]
    pub function_name: String,

    /// Runtime environment (e.g., "rs", "js", "py")
    pub runtime: String,

    /// Source code content
    #[serde(rename = "sourceCode")]
    pub source_code: String,
}

/// Response from creating a lambda function
#[derive(Debug, Serialize)]
pub struct CreateLambdaResponse {
    /// Whether creation was successful
    pub success: bool,

    /// Success or error message
    pub message: String,

    /// Created function name
    pub function_name: String,

    /// File path where source was saved
    pub source_path: Option<String>,

    /// WASM file path if compilation succeeded
    pub wasm_path: Option<String>,

    /// Size of compiled WASM in bytes
    pub wasm_size_bytes: Option<u64>,

    /// Compilation time in milliseconds
    pub compilation_time_ms: Option<u64>,
}

/// Response from updating a lambda function
#[derive(Debug, Serialize)]
pub struct UpdateLambdaResponse {
    /// Whether update was successful
    pub success: bool,

    /// Success or error message
    pub message: String,

    /// Updated function name
    pub function_name: String,
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

/// Request to compile a lambda function
#[derive(Debug, Deserialize)]
pub struct CompileLambdaRequest {
    pub lambda_name: String,
}

/// Response from lambda compilation
#[derive(Debug, Serialize)]
pub struct CompileLambdaResponse {
    pub success: bool,
    pub lambda_name: String,
    pub wasm_size_bytes: Option<usize>,
    pub wasm_path: Option<String>,
    pub compilation_time_ms: u64,
    pub message: String,
}
