//! Workflow repository trait for step function management and execution

use super::dto::{
    WorkflowSummary,
    // CreateWorkflowRequest, CreateWorkflowResponse, ExecuteWorkflowRequest, ExecuteWorkflowResponse,
    // UpdateWorkflowRequest, Workflow, WorkflowValidationResult,
};
use crate::error_handling::types::AppResult;

/// Workflow repository trait for managing step function workflows
///
/// Following RUSTWS clean architecture principles:
/// - Repository handles data access and workflow file operations
/// - Validation logic for workflow JSON structure
/// - Execution engine for running step functions
/// - Clean separation from business logic
pub trait WorkflowRepository: Send + Sync {
    /// Get all workflow summaries
    /// Returns lightweight workflow information for listing/dashboard views
    async fn get_all(&self) -> AppResult<Vec<WorkflowSummary>>;

    // /// Get full workflow definition by name
    // /// Returns complete workflow JSON and metadata for editing/execution
    // async fn get_by_name(&self, name: &str) -> AppResult<Option<Workflow>>;

    // /// Create a new workflow
    // /// Saves workflow JSON to filesystem and validates structure
    // async fn create(&self, request: CreateWorkflowRequest) -> AppResult<CreateWorkflowResponse>;

    // /// Update an existing workflow
    // /// Updates workflow JSON and re-validates structure
    // async fn update(&self, workflow_name: &str, request: UpdateWorkflowRequest) -> AppResult<()>;

    // /// Delete a workflow by name
    // /// Removes workflow JSON file and any associated metadata
    // async fn delete(&self, workflow_name: &str) -> AppResult<()>;

    // /// Validate workflow structure and dependencies
    // /// Checks JSON syntax, state references, and lambda availability
    // async fn validate(&self, workflow_name: &str) -> AppResult<WorkflowValidationResult>;

    // /// Validate workflow JSON content directly
    // /// Useful for validating before saving (e.g., in create/update operations)
    // async fn validate_json(&self, workflow_json: &str) -> AppResult<WorkflowValidationResult>;

    // /// Execute a workflow with input data
    // /// Runs the step function workflow and returns execution results
    // async fn execute(&self, request: ExecuteWorkflowRequest) -> AppResult<ExecuteWorkflowResponse>;

    // /// Check if a workflow exists by name
    // /// Utility method for quick existence checks
    // async fn exists(&self, workflow_name: &str) -> AppResult<bool>;

    // /// List available workflow names
    // /// Returns just the names for quick reference/validation
    // async fn list_names(&self) -> AppResult<Vec<String>>;
}
