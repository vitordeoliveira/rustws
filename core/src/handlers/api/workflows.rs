//! Workflow (Step Functions) API handlers

use axum::{
    extract::{Path, State},
    response::Json,
};
use axum_login::AuthSession;
use tracing::{error, info, instrument, warn};

use crate::{
    auth::dto::AuthBackend,
    business_logic::workflows::{
        ExecuteWorkflowRequest, ExecuteWorkflowResponse, WorkflowsService,
    },
    error_handling::types::AppResult,
    infrastructure::step_functions::StepFunctionStorage,
    state::AppState,
};

/// Execute a workflow (step function) with custom JSON input
#[instrument(
    skip_all,
    fields(
        handler = "execute_workflow", 
        operation = "api_execute",
        workflow_name = %workflow_name
    )
)]
pub async fn execute_workflow_handler(
    State(_state): State<AppState>,
    _auth_session: AuthSession<AuthBackend>,
    workflows_service: WorkflowsService<StepFunctionStorage>,
    Path(workflow_name): Path<String>,
    Json(input_json): Json<serde_json::Value>,
) -> AppResult<Json<ExecuteWorkflowResponse>> {
    info!(
        workflow_name = %workflow_name,
        "Workflow execution request received with custom input"
    );

    // Validate workflow name
    if workflow_name.trim().is_empty() {
        warn!(
            workflow_name = %workflow_name,
            "Invalid workflow name provided - empty or whitespace"
        );
        return Err(crate::error_handling::types::AppError::validation(
            "Workflow name cannot be empty or contain only whitespace",
        ));
    }

    let request = ExecuteWorkflowRequest {
        workflow_name: workflow_name.clone(),
        input_data: input_json,
    };

    info!(
        workflow_name = %workflow_name,
        "Starting workflow execution process"
    );

    // Use the service to execute the workflow
    match workflows_service.execute(request).await {
        Ok(response) => {
            match &response {
                ExecuteWorkflowResponse::Success {
                    output_data,
                    execution_time_ms,
                    states_executed,
                } => {
                    info!(
                        workflow_name = %workflow_name,
                        execution_time_ms = execution_time_ms,
                        states_executed = states_executed.len(),
                        "Workflow execution completed successfully"
                    );
                }
                ExecuteWorkflowResponse::Failed {
                    error_message,
                    execution_time_ms,
                    ..
                } => {
                    warn!(
                        workflow_name = %workflow_name,
                        error_message = %error_message,
                        execution_time_ms = execution_time_ms,
                        "Workflow execution failed"
                    );
                }
            }
            Ok(Json(response))
        }
        Err(e) => {
            error!(
                workflow_name = %workflow_name,
                error = %e,
                "Workflow execution service error"
            );
            Err(e)
        }
    }
}
