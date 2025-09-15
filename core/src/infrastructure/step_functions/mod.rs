use std::fs;
use std::path::{Path, PathBuf};
use tracing::instrument;

use crate::business_logic::lambdas::{
    ExecuteLambdaRequest, ExecuteLambdaResponse, LambdaRepository, Resource,
};
use crate::business_logic::workflows::{
    ExecuteWorkflowRequest, ExecuteWorkflowResponse, WorkflowRepository, WorkflowStatus,
    WorkflowSummary,
};
use crate::error_handling::types::{AppError, AppResult};
use crate::infrastructure::lambdas::LambdaStorage;
use crate::infrastructure::step_functions::workflow::{State, TaskState, Workflow};

pub mod workflow;

/// Step Functions storage implementation
///
/// Manages workflow definitions stored as JSON files in the filesystem.
/// Similar to LambdaStorage but for step function workflows.
#[derive(Debug, Clone)]
pub struct StepFunctionStorage {
    /// Base path for step functions infrastructure
    base_path: PathBuf,
    /// Lambda storage for executing lambda functions in workflows
    lambda_storage: LambdaStorage,
}

impl StepFunctionStorage {
    /// Create a new StepFunctionStorage with default path and lambda storage
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("src/infrastructure/step_functions"),
            lambda_storage: LambdaStorage::new(),
        }
    }

    /// Create a new StepFunctionStorage with custom base path and default lambda storage
    pub fn with_base_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            base_path: path.as_ref().to_path_buf(),
            lambda_storage: LambdaStorage::new(),
        }
    }

    /// Create a new StepFunctionStorage with custom lambda storage
    pub fn with_lambda_storage(lambda_storage: LambdaStorage) -> Self {
        Self {
            base_path: PathBuf::from("src/infrastructure/step_functions"),
            lambda_storage,
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

        let start_time = std::time::Instant::now();

        // Load and parse the workflow definition
        let workflow = match self.load_workflow(&request.workflow_name).await {
            Ok(Some(workflow)) => workflow,
            Ok(None) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                return Ok(ExecuteWorkflowResponse::Failed {
                    error_message: format!("Workflow '{}' not found", request.workflow_name),
                    failed_at_state: None,
                    execution_time_ms: execution_time,
                });
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                return Ok(ExecuteWorkflowResponse::Failed {
                    error_message: format!("Failed to load workflow: {}", e),
                    failed_at_state: None,
                    execution_time_ms: execution_time,
                });
            }
        };

        // Execute the workflow starting from the specified start state
        match self
            .execute_workflow_states(&workflow, request.input_data)
            .await
        {
            Ok((output_data, states_executed)) => {
                let execution_time = start_time.elapsed().as_millis() as u64;

                tracing::info!(
                    workflow_name = %request.workflow_name,
                    states_executed = states_executed.len(),
                    execution_time_ms = execution_time,
                    "Workflow execution completed successfully"
                );

                Ok(ExecuteWorkflowResponse::Success {
                    output_data,
                    execution_time_ms: execution_time,
                    states_executed,
                })
            }
            Err((error_message, failed_at_state)) => {
                let execution_time = start_time.elapsed().as_millis() as u64;

                tracing::error!(
                    workflow_name = %request.workflow_name,
                    error = %error_message,
                    failed_at_state = ?failed_at_state,
                    execution_time_ms = execution_time,
                    "Workflow execution failed"
                );

                Ok(ExecuteWorkflowResponse::Failed {
                    error_message,
                    failed_at_state,
                    execution_time_ms: execution_time,
                })
            }
        }
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

    /// Load and parse a workflow definition from JSON file
    #[instrument(skip_all, fields(operation = "load_workflow", workflow_name = %workflow_name))]
    async fn load_workflow(&self, workflow_name: &str) -> AppResult<Option<Workflow>> {
        let workflows_dir = self.workflows_dir();
        let workflow_path = workflows_dir.join(format!("{}.json", workflow_name));

        if !workflow_path.exists() {
            tracing::warn!(
                workflow_name = %workflow_name,
                workflow_path = %workflow_path.display(),
                "Workflow file not found"
            );
            return Ok(None);
        }

        // Read and parse the workflow JSON file
        let content = fs::read_to_string(&workflow_path).map_err(|e| {
            AppError::internal(&format!(
                "Failed to read workflow file '{}': {}",
                workflow_name, e
            ))
        })?;

        let workflow: Workflow = serde_json::from_str(&content).map_err(|e| {
            AppError::internal(&format!(
                "Failed to parse workflow '{}' as JSON: {}",
                workflow_name, e
            ))
        })?;

        tracing::debug!(
            workflow_name = %workflow_name,
            start_at = %workflow.start_at,
            states_count = workflow.states.len(),
            "Workflow loaded successfully"
        );

        Ok(Some(workflow))
    }

    /// Execute workflow states sequentially with shared memory context
    #[instrument(skip_all, fields(operation = "execute_workflow_states"))]
    async fn execute_workflow_states(
        &self,
        workflow: &Workflow,
        initial_input: serde_json::Value,
    ) -> Result<(serde_json::Value, Vec<String>), (String, Option<String>)> {
        use wasmer::{Engine, Store};

        tracing::info!(
            start_at = %workflow.start_at,
            states_count = workflow.states.len(),
            "Starting workflow execution with shared memory context"
        );

        // Create shared wasmer engine and store for all lambda executions
        let engine = Engine::default();
        let mut _store = Store::new(engine); // Store for future shared memory usage

        let mut current_input = initial_input;
        let mut current_state_name = workflow.start_at.clone();
        let mut states_executed = Vec::new();

        // Execute states in sequence until we reach an end state
        loop {
            tracing::debug!(
                current_state = %current_state_name,
                "Executing workflow state"
            );

            // Find the current state in the workflow
            let current_state = match workflow.states.get(&current_state_name) {
                Some(state) => state,
                None => {
                    return Err((
                        format!("State '{}' not found in workflow", current_state_name),
                        Some(current_state_name),
                    ));
                }
            };

            states_executed.push(current_state_name.clone());

            match current_state {
                State::Task(task_state) => {
                    // Execute the lambda function referenced by this task state
                    match self.execute_task_state(task_state, &current_input).await {
                        Ok(output) => {
                            current_input = output;

                            // Determine next state
                            if let Some(next) = &task_state.next {
                                current_state_name = next.clone();
                            } else {
                                // This is a terminal state
                                tracing::info!(
                                    final_state = %current_state_name,
                                    states_executed = states_executed.len(),
                                    "Workflow execution completed - reached terminal task state"
                                );
                                break;
                            }
                        }
                        Err(e) => {
                            return Err((
                                format!(
                                    "Failed to execute task state '{}': {}",
                                    current_state_name, e
                                ),
                                Some(current_state_name),
                            ));
                        }
                    }
                }
                State::Succeed(_) => {
                    // Succeed state - workflow completed successfully
                    tracing::info!(
                        final_state = %current_state_name,
                        states_executed = states_executed.len(),
                        "Workflow execution completed - reached succeed state"
                    );
                    break;
                }
                State::Fail(fail_state) => {
                    // Fail state - workflow failed
                    let error_message = fail_state.error.clone().unwrap_or_else(|| {
                        format!("Workflow failed at state '{}'", current_state_name)
                    });

                    return Err((error_message, Some(current_state_name)));
                }
                State::Choice(_) => {
                    // Choice state - not implemented yet
                    return Err((
                        format!(
                            "Choice states are not yet implemented (state: '{}')",
                            current_state_name
                        ),
                        Some(current_state_name),
                    ));
                }
                State::Wait(_) => {
                    // Wait state - not implemented yet
                    return Err((
                        format!(
                            "Wait states are not yet implemented (state: '{}')",
                            current_state_name
                        ),
                        Some(current_state_name),
                    ));
                }
            }
        }

        tracing::info!(
            states_executed = states_executed.len(),
            "Workflow execution completed successfully"
        );

        Ok((current_input, states_executed))
    }

    /// Execute a single task state by calling the referenced lambda function
    #[instrument(skip_all, fields(operation = "execute_task_state", resource = %task_state.resource))]
    async fn execute_task_state(
        &self,
        task_state: &TaskState,
        input_data: &serde_json::Value,
    ) -> AppResult<serde_json::Value> {
        tracing::info!(
            resource = %task_state.resource,
            "Executing task state with lambda function"
        );

        // Parse the lambda name from the resource URN
        let lambda_name = match Resource::parse_lambda_name(&task_state.resource) {
            Some(name) => {
                tracing::debug!(
                    resource = %task_state.resource,
                    lambda_name = %name,
                    "Successfully extracted lambda name from resource"
                );
                name
            }
            None => {
                return Err(AppError::validation(&format!(
                    "Invalid lambda resource format: '{}'",
                    task_state.resource
                )));
            }
        };

        // Convert JSON input to bytes for lambda execution
        let input_bytes = serde_json::to_vec(input_data)
            .map_err(|e| AppError::validation(&format!("Failed to serialize input data: {}", e)))?;

        // Create execution request
        let execute_request = ExecuteLambdaRequest {
            function_name: Some(lambda_name.clone()),
            input_data: input_bytes,
        };

        // Execute the lambda using lambda_storage
        let response = self.lambda_storage.execute(execute_request).await?;

        // Process the response
        match response {
            ExecuteLambdaResponse::Success { output_data } => {
                // Parse the output bytes back to JSON
                let output_json: serde_json::Value =
                    serde_json::from_slice(&output_data).map_err(|e| {
                        AppError::validation(&format!(
                            "Failed to parse lambda output as JSON: {}",
                            e
                        ))
                    })?;

                tracing::debug!(
                    lambda_name = %lambda_name,
                    output_size = output_data.len(),
                    "Lambda execution successful"
                );

                Ok(output_json)
            }
            ExecuteLambdaResponse::Failed { error_message } => Err(AppError::validation(&format!(
                "Lambda '{}' execution failed: {}",
                lambda_name, error_message
            ))),
        }
    }
}
