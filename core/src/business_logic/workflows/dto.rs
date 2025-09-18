//! Workflow Data Transfer Objects

use serde::{Deserialize, Serialize};

/// Workflow summary for listing/dashboard views
/// Lightweight representation of workflow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummary {
    pub name: String,
    pub description: Option<String>,
    pub status: WorkflowStatus,
    pub state_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Workflow status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Active,
    Inactive,
    Draft,
    Error,
}

/// Request to execute a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub workflow_name: String,
    pub input_data: serde_json::Value,
}

/// Request to create a new workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub definition: String, // JSON string of the workflow definition
}

/// Response from creating a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowResponse {
    pub success: bool,
    pub message: String,
    pub workflow_name: String,
}

/// Request to update an existing workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub status: Option<WorkflowStatus>,
    pub definition: Option<String>, // JSON string of the updated workflow definition
}

/// Complete workflow definition with metadata
/// Used for editing and detailed workflow management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: Option<String>,
    pub status: WorkflowStatus,
    pub definition: String, // JSON string of the workflow definition
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Response from workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecuteWorkflowResponse {
    Success {
        output_data: serde_json::Value,
        execution_time_ms: u64,
        states_executed: Vec<String>,
    },
    Failed {
        error_message: String,
        failed_at_state: Option<String>,
        execution_time_ms: u64,
    },
}
