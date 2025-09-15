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
