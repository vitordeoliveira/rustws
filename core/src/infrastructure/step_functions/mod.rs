use std::fs;
use std::path::{Path, PathBuf};
use tracing::instrument;

use crate::business_logic::workflows::{
    ExecuteWorkflowRequest, ExecuteWorkflowResponse, WorkflowRepository, WorkflowStatus,
    WorkflowSummary,
};
use crate::error_handling::types::{AppError, AppResult};
use crate::infrastructure::step_functions::workflow::Workflow;

pub mod workflow;

/// Step Functions storage implementation
///
/// Manages workflow definitions stored as JSON files in the filesystem.
/// Similar to LambdaStorage but for step function workflows.
#[derive(Debug, Clone)]
pub struct StepFunctionStorage {
    /// Base path for step functions infrastructure
    base_path: PathBuf,
}

impl StepFunctionStorage {
    /// Create a new StepFunctionStorage with default path
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("src/infrastructure/step_functions"),
        }
    }

    /// Create a new StepFunctionStorage with custom base path
    pub fn with_base_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            base_path: path.as_ref().to_path_buf(),
        }
    }

    /// Get the workflows directory path
    pub fn workflows_dir(&self) -> PathBuf {
        self.base_path.join("workflows")
    }
}

impl Default for StepFunctionStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowRepository for StepFunctionStorage {
    #[instrument(
        skip_all,
        fields(repository = "step_functions", operation = "get_all_workflows")
    )]
    async fn get_all(&self) -> AppResult<Vec<WorkflowSummary>> {
        let workflows_dir = self.workflows_dir();

        // Return empty list if workflows directory doesn't exist
        if !workflows_dir.exists() {
            tracing::info!("Workflows directory does not exist, returning empty list");
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&workflows_dir).map_err(|e| {
            AppError::internal(&format!("Failed to read workflows directory: {}", e))
        })?;

        let mut summaries = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                AppError::internal(&format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            // Only process JSON files
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.process_workflow_file(&path).await {
                    Ok(Some(summary)) => {
                        tracing::debug!(workflow_name = %summary.name, "Found workflow");
                        summaries.push(summary);
                    }
                    Ok(None) => {
                        tracing::warn!(file = ?path, "Skipped invalid workflow file");
                    }
                    Err(e) => {
                        tracing::error!(file = ?path, error = %e, "Failed to process workflow file");
                        // Continue processing other files even if one fails
                    }
                }
            }
        }

        tracing::info!(count = summaries.len(), "Found workflows");
        Ok(summaries)
    }

    #[instrument(
        skip_all,
        fields(
            repository = "step_functions", 
            operation = "execute_workflow",
            workflow_name = %request.workflow_name
        )
    )]
    async fn execute(&self, request: ExecuteWorkflowRequest) -> AppResult<ExecuteWorkflowResponse> {
        tracing::info!(
            workflow_name = %request.workflow_name,
            "Workflow execution request received"
        );

        // TODO: Implement workflow execution logic
        // This is a placeholder implementation that will be expanded later

        let start_time = std::time::Instant::now();

        // For now, return a simple success response
        let execution_time = start_time.elapsed().as_millis() as u64;

        tracing::info!(
            workflow_name = %request.workflow_name,
            execution_time_ms = execution_time,
            "Workflow execution completed (placeholder)"
        );

        Ok(ExecuteWorkflowResponse::Success {
            output_data: serde_json::json!({
                "message": format!("Workflow '{}' executed successfully", request.workflow_name),
                "input_received": request.input_data
            }),
            execution_time_ms: execution_time,
            states_executed: vec![format!("{}_placeholder_state", request.workflow_name)],
        })
    }
}

impl StepFunctionStorage {
    /// Process a single workflow JSON file and create a WorkflowSummary
    #[instrument(skip_all, fields(operation = "process_workflow_file"))]
    async fn process_workflow_file(&self, path: &Path) -> AppResult<Option<WorkflowSummary>> {
        // Extract workflow name from filename
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Read and parse the workflow JSON file
        let content = fs::read_to_string(path).map_err(|e| {
            AppError::internal(&format!("Failed to read workflow file '{}': {}", name, e))
        })?;

        // Parse JSON to validate structure and extract metadata
        let workflow: Workflow = serde_json::from_str(&content).map_err(|e| {
            AppError::internal(&format!(
                "Failed to parse workflow '{}' as JSON: {}",
                name, e
            ))
        })?;

        // Get file metadata for timestamps
        let metadata = fs::metadata(path).map_err(|e| {
            AppError::internal(&format!(
                "Failed to read file metadata for '{}': {}",
                name, e
            ))
        })?;

        // Convert file times to chrono DateTime
        let created_at = metadata
            .created()
            .or_else(|_| metadata.modified())
            .map(|time| chrono::DateTime::<chrono::Utc>::from(time))
            .unwrap_or_else(|_| chrono::Utc::now());

        let updated_at = metadata
            .modified()
            .map(|time| chrono::DateTime::<chrono::Utc>::from(time))
            .unwrap_or_else(|_| chrono::Utc::now());

        // Validate workflow structure to determine status
        let status = match workflow.validate() {
            Ok(_) => WorkflowStatus::Active,
            Err(_) => WorkflowStatus::Error,
        };

        // Count the number of states
        let state_count = workflow.states.len() as u32;

        let summary = WorkflowSummary {
            name,
            description: workflow.comment,
            status,
            state_count,
            created_at,
            updated_at,
        };

        Ok(Some(summary))
    }
}
