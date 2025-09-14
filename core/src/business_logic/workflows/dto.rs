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
