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
        ExecuteWorkflowRequest, ExecuteWorkflowResponse, UpdateWorkflowRequest, Workflow,
        WorkflowsService,
    },
    error_handling::types::AppResult,
    infrastructure::step_functions::StepFunctionStorage,
    state::AppState,
    ui::step_functions::EditStepFunctionPageUi,
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
    mut workflows_service: WorkflowsService<StepFunctionStorage>,
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
                    output_data: _,
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

/// Get workflow definition details for viewing/inspection
#[instrument(
    skip_all,
    fields(
        handler = "get_workflow_definition", 
        operation = "api_get_definition",
        workflow_name = %workflow_name
    )
)]
pub async fn get_workflow_definition_handler(
    State(_state): State<AppState>,
    _auth_session: AuthSession<AuthBackend>,
    workflows_service: WorkflowsService<StepFunctionStorage>,
    Path(workflow_name): Path<String>,
) -> AppResult<Json<Workflow>> {
    info!(
        workflow_name = %workflow_name,
        "Workflow definition request received"
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

    // Get workflow definition from service
    match workflows_service.get_by_name(&workflow_name).await {
        Ok(Some(workflow)) => {
            info!(
                workflow_name = %workflow_name,
                definition_size = workflow.definition.len(),
                "Successfully retrieved workflow definition"
            );
            Ok(Json(workflow))
        }
        Ok(None) => {
            warn!(
                workflow_name = %workflow_name,
                "Workflow not found"
            );
            Err(crate::error_handling::types::AppError::not_found(&format!(
                "Workflow '{}' not found",
                workflow_name
            )))
        }
        Err(e) => {
            error!(
                workflow_name = %workflow_name,
                error = %e,
                "Failed to retrieve workflow definition"
            );
            Err(e)
        }
    }
}

/// Update a workflow definition
#[instrument(
    skip_all,
    fields(
        handler = "update_workflow", 
        operation = "api_update",
        workflow_name = %workflow_name
    )
)]
pub async fn update_workflow_handler(
    State(_state): State<AppState>,
    _auth_session: AuthSession<AuthBackend>,
    workflows_service: WorkflowsService<StepFunctionStorage>,
    Path(workflow_name): Path<String>,
    Json(request): Json<UpdateWorkflowRequest>,
) -> AppResult<Json<()>> {
    info!(
        workflow_name = %workflow_name,
        "Workflow update request received"
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

    // Update workflow through service
    match workflows_service.update(&workflow_name, request).await {
        Ok(()) => {
            info!(
                workflow_name = %workflow_name,
                "Successfully updated workflow"
            );
            Ok(Json(()))
        }
        Err(e) => {
            error!(
                workflow_name = %workflow_name,
                error = %e,
                "Failed to update workflow"
            );
            Err(e)
        }
    }
}

/// Generate workflow graph visualization
#[instrument(
    skip_all,
    fields(handler = "generate_workflow_graph", operation = "api_generate_graph")
)]
pub async fn generate_workflow_graph_handler(
    State(_state): State<AppState>,
    _auth_session: AuthSession<AuthBackend>,
    Json(request): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    info!("Workflow graph generation request received");

    // Extract definition from request
    let definition = match request.get("definition").and_then(|d| d.as_str()) {
        Some(def) => def,
        None => {
            warn!("No definition provided in graph generation request");
            return Err(crate::error_handling::types::AppError::validation(
                "Definition is required for graph generation",
            ));
        }
    };

    // Validate JSON
    let _workflow: serde_json::Value = match serde_json::from_str(definition) {
        Ok(w) => w,
        Err(e) => {
            warn!(error = %e, "Invalid JSON provided for graph generation");
            return Err(crate::error_handling::types::AppError::validation(
                &format!("Invalid JSON: {}", e),
            ));
        }
    };

    // Generate workflow graph using static method

    let graph_svg = EditStepFunctionPageUi::render_workflow_graph_static(definition);

    info!("Successfully generated workflow graph");

    Ok(Json(serde_json::json!({
        "success": true,
        "graph": graph_svg
    })))
}
