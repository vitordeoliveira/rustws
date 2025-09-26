//! Main page handlers

use axum::{
    Form,
    extract::{Path, State},
    response::Html,
};
use axum_login::AuthSession;
use serde::Deserialize;
use tracing::instrument;

use crate::{
    auth::dto::AuthBackend,
    business_logic::{
        api_gateway::ApiGatewayRepository,
        lambdas::LambdasService,
        workflows::{CreateWorkflowRequest, WorkflowSummary, WorkflowsService},
    },
    error_handling::types::AppResult,
    infrastructure::{
        api_gateways::ApiGatewayStorage,
        lambdas::{LambdaStorage, metrics::ExecutionStatus},
        step_functions::StepFunctionStorage,
    },
    state::AppState,
    ui::{
        Ui,
        api_gateway::ApiGatewayPageUi,
        home::HomePageUi,
        lambda::{CreateLambdaPageUi, EditLambdaPageUi, LambdaMetricsPageUi, LambdaPageUi},
        monitoring::{MonitoringPageUi, get_mock_monitoring_data},
        reports::{ReportSummary, ReportsPageUi},
        step_functions::{CreateStepFunctionPageUi, EditStepFunctionPageUi, StepFunctionsPageUi},
    },
};

#[derive(Deserialize)]
pub struct CreateWorkflowFormData {
    #[serde(rename = "functionName")]
    pub function_name: String,
    pub description: Option<String>,
    #[serde(rename = "workflowDefinition")]
    pub workflow_definition: String,
}

/// Home page handler - delegates all UI concerns to UI layer
#[instrument(skip_all, fields(handler = "home", operation = "page_render"))]
pub async fn home_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();
    let home_page_ui = HomePageUi::new(user);
    let html = home_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Lambda page handler - delegates all UI concerns to UI layer
#[instrument(skip_all, fields(handler = "lambda", operation = "page_render"))]
pub async fn lambda_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Get lambda data from service
    let lambdas = lambda_service.get_all().await?;
    let metrics = lambda_service.get_metrics().clone();

    let lambda_page_ui = LambdaPageUi::new(user, lambdas, metrics);
    let html = lambda_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Create lambda page handler - renders lambda creation form
#[instrument(skip_all, fields(handler = "create_lambda", operation = "page_render"))]
pub async fn create_lambda_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    let create_lambda_page_ui = CreateLambdaPageUi::new(user);
    let html = create_lambda_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Delete lambda handler - deletes lambda and returns lambda index page
#[instrument(skip_all, fields(handler = "delete_lambda", operation = "page_delete"))]
pub async fn delete_lambda_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Path(lambda_name): Path<String>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Delete the lambda function
    if let Err(e) = lambda_service.delete(&lambda_name).await {
        tracing::error!(
            lambda_name = %lambda_name,
            error = %e,
            "Failed to delete lambda function"
        );
        // Continue to render the page even if deletion failed
        // The user will see the lambda still exists in the list
    } else {
        tracing::info!(
            lambda_name = %lambda_name,
            "Lambda function deleted successfully"
        );
    }

    // Get updated lambda list (after potential deletion)
    let lambdas = lambda_service.get_all().await?;
    let metrics = lambda_service.get_metrics().clone();

    // Render lambda index page with updated list
    let lambda_page_ui = LambdaPageUi::new(user, lambdas, metrics);
    let html = lambda_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Edit lambda page handler - renders lambda edit form with current source code
#[instrument(skip_all, fields(handler = "edit_lambda", operation = "page_render"))]
pub async fn edit_lambda_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Path(lambda_name): Path<String>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    tracing::info!(
        lambda_name = %lambda_name,
        "Loading lambda for editing"
    );

    // Load lambda data from business logic
    let lambda = lambda_service.get_by_name(&lambda_name).await?;

    match lambda {
        Some(lambda_data) => {
            tracing::debug!(
                lambda_name = %lambda_name,
                source_length = lambda_data.source_code.len(),
                has_wasm = lambda_data.wasm_bytes.is_some(),
                status = ?lambda_data.status,
                "Lambda data loaded successfully for editing"
            );

            let edit_lambda_page_ui =
                EditLambdaPageUi::new(user, lambda_data.name, lambda_data.source_code);
            let html = edit_lambda_page_ui.render_html(&state.tera)?;

            Ok(html)
        }
        None => {
            tracing::warn!(
                lambda_name = %lambda_name,
                "Lambda function not found for editing"
            );

            Err(crate::error_handling::types::AppError::not_found(&format!(
                "Lambda function '{}' not found",
                lambda_name
            )))
        }
    }
}

/// Lambda metrics page handler - shows execution metrics for a specific lambda
#[instrument(
    skip_all,
    fields(handler = "lambda_metrics", operation = "page_render")
)]
pub async fn lambda_metrics_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Path(lambda_name): Path<String>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    tracing::info!(
        lambda_name = %lambda_name,
        "Loading metrics for lambda"
    );

    // We need to access the ledger entries, but the current service only gives us aggregated metrics
    // Let's access the storage directly to get the ledger
    let lambda_storage = LambdaStorage::new();
    let ledger = lambda_storage.get_metrics_ledger();

    let lambda_executions: Vec<_> = ledger
        .entries
        .iter()
        .filter(|entry| entry.lambda_name == lambda_name)
        .cloned()
        .collect();

    tracing::debug!(
        lambda_name = %lambda_name,
        execution_count = lambda_executions.len(),
        "Filtered lambda execution entries"
    );

    // Get lambda metadata for schema information
    let lambda_data = lambda_service.get_by_name(&lambda_name).await?;
    let lambda_metadata = lambda_data.and_then(|lambda| lambda.metadata);

    tracing::debug!(
        lambda_name = %lambda_name,
        has_metadata = lambda_metadata.is_some(),
        "Retrieved lambda metadata for schema display"
    );

    let lambda_metrics_page_ui = LambdaMetricsPageUi::new(
        user,
        lambda_name.clone(),
        lambda_executions,
        lambda_metadata,
    );
    let html = lambda_metrics_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Step Functions page handler - delegates all UI concerns to UI layer
#[instrument(
    skip_all,
    fields(handler = "step_functions", operation = "page_render")
)]
pub async fn step_functions_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    workflows_service: WorkflowsService<StepFunctionStorage>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Get workflow data from service
    let workflows = workflows_service.get_all().await?;

    let step_functions_page_ui = StepFunctionsPageUi::new(user, workflows);
    let html = step_functions_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Create step function page handler - renders step function creation form
#[instrument(
    skip_all,
    fields(handler = "create_step_function", operation = "page_render")
)]
pub async fn create_step_function_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    let create_step_function_page_ui = CreateStepFunctionPageUi::new(user);
    let html = create_step_function_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Edit step function page handler - renders step function editing form  
#[instrument(
    skip_all,
    fields(handler = "edit_step_function", operation = "page_render", workflow_name = %workflow_name)
)]
pub async fn edit_step_function_handler(
    Path(workflow_name): Path<String>,
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    workflows_service: WorkflowsService<StepFunctionStorage>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Get workflow data from service
    let workflow = workflows_service.get_by_name(&workflow_name).await?;

    match workflow {
        Some(workflow) => {
            // Convert business logic Workflow to WorkflowSummary for UI compatibility
            let workflow_summary = WorkflowSummary {
                name: workflow.name.clone(),
                description: workflow.description.clone(),
                status: workflow.status.clone(),
                state_count: workflow.definition.matches("\"Type\"").count() as u32, // Quick state count
                created_at: workflow.created_at,
                updated_at: workflow.updated_at,
            };

            let edit_step_function_page_ui =
                EditStepFunctionPageUi::new(user, workflow_summary, workflow.definition);
            let html = edit_step_function_page_ui.render_html(&state.tera)?;
            Ok(html)
        }
        None => {
            // Workflow not found, redirect to step functions list
            Err(crate::error_handling::types::AppError::not_found(&format!(
                "Workflow '{}' not found",
                workflow_name
            )))
        }
    }
}

/// Delete step function handler - deletes workflow and returns step functions index page
#[instrument(
    skip_all,
    fields(handler = "delete_step_function", operation = "page_delete")
)]
pub async fn delete_step_function_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    workflows_service: WorkflowsService<StepFunctionStorage>,
    Path(workflow_name): Path<String>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Delete the workflow
    if let Err(e) = workflows_service.delete(&workflow_name).await {
        tracing::error!(
            workflow_name = %workflow_name,
            error = %e,
            "Failed to delete workflow"
        );
        // Continue to render the page even if deletion failed
        // The user will see the workflow still exists in the list
    } else {
        tracing::info!(
            workflow_name = %workflow_name,
            "Workflow deleted successfully"
        );
    }

    // Get updated workflow list (after potential deletion)
    let workflows = workflows_service.get_all().await?;

    // Render step functions index page with updated list
    let step_functions_page_ui = StepFunctionsPageUi::new(user, workflows);
    let html = step_functions_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Create workflow form handler - processes form data and creates workflow
#[instrument(
    skip_all,
    fields(handler = "create_workflow_form", operation = "form_submission")
)]
pub async fn create_workflow_form_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    workflows_service: WorkflowsService<StepFunctionStorage>,
    Form(form_data): Form<CreateWorkflowFormData>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    tracing::info!(
        workflow_name = %form_data.function_name,
        "Processing workflow creation form submission"
    );

    // Validate workflow name
    if form_data.function_name.trim().is_empty() {
        tracing::warn!(
            workflow_name = %form_data.function_name,
            "Invalid workflow name - empty or whitespace"
        );
        // Return to create page with error - for now just return to list
        let workflows = workflows_service.get_all().await?;
        let step_functions_page_ui = StepFunctionsPageUi::new(user, workflows);
        return step_functions_page_ui.render_html(&state.tera);
    }

    // Validate JSON syntax
    if let Err(e) = serde_json::from_str::<serde_json::Value>(&form_data.workflow_definition) {
        tracing::warn!(
            workflow_name = %form_data.function_name,
            error = %e,
            "Invalid workflow JSON definition"
        );
        // Return to create page with error - for now just return to list
        let workflows = workflows_service.get_all().await?;
        let step_functions_page_ui = StepFunctionsPageUi::new(user, workflows);
        return step_functions_page_ui.render_html(&state.tera);
    }

    // Create workflow request
    let create_request = CreateWorkflowRequest {
        name: form_data.function_name.clone(),
        description: form_data.description.clone(),
        definition: form_data.workflow_definition,
    };

    // Create the workflow
    match workflows_service.create(create_request).await {
        Ok(response) => {
            if response.success {
                tracing::info!(
                    workflow_name = %form_data.function_name,
                    "Workflow created successfully"
                );
            } else {
                tracing::warn!(
                    workflow_name = %form_data.function_name,
                    message = %response.message,
                    "Workflow creation failed"
                );
            }
        }
        Err(e) => {
            tracing::error!(
                workflow_name = %form_data.function_name,
                error = %e,
                "Failed to create workflow"
            );
        }
    }

    // Get updated workflow list and return to step functions page
    let workflows = workflows_service.get_all().await?;
    let step_functions_page_ui = StepFunctionsPageUi::new(user, workflows);
    let html = step_functions_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// API Gateway page handler - delegates all UI concerns to UI layer
#[instrument(skip_all, fields(handler = "api_gateway", operation = "page_render"))]
pub async fn api_gateway_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Get real API gateways from storage
    let api_gateway_storage = ApiGatewayStorage::new();
    let api_gateways = api_gateway_storage.get_api_gateways().await?;

    let api_gateway_page_ui = ApiGatewayPageUi::new(user, api_gateways);
    let html = api_gateway_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Monitoring page handler - delegates all UI concerns to UI layer
#[instrument(skip_all, fields(handler = "monitoring", operation = "page_render"))]
pub async fn monitoring_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Get mock monitoring data for UI demonstration
    let (system_metrics, service_health, recent_alerts, recent_logs, performance_metrics) =
        get_mock_monitoring_data();

    let monitoring_page_ui = MonitoringPageUi::new(
        user,
        system_metrics,
        service_health,
        recent_alerts,
        recent_logs,
        performance_metrics,
    );
    let html = monitoring_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Reports page handler - displays vulnerability assessment reports
#[instrument(skip_all, fields(handler = "reports", operation = "page_render"))]
pub async fn reports_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    _lambda_service: LambdasService<LambdaStorage>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Get vulnerability reporter executions from lambda metrics ledger
    let lambda_storage = LambdaStorage::new();
    let ledger = lambda_storage.get_metrics_ledger();
    let vulnerability_reports: Vec<ReportSummary> = ledger
        .entries
        .iter()
        .filter(|entry| {
            entry.lambda_name == "vulnerability_reporter"
                && matches!(entry.status, ExecutionStatus::Success)
        })
        .filter_map(|entry| {
            // Parse the output to extract report summary
            if let Some(output_data) = &entry.output_data {
                if let Ok(report_data) = serde_json::from_str::<serde_json::Value>(output_data) {
                    Some(ReportSummary {
                        report_id: report_data["report_id"]
                            .as_str()
                            .unwrap_or("unknown")
                            .to_string(),
                        execution_id: entry.execution_id.clone(),
                        generated_at: entry.timestamp.to_rfc3339(),
                        security_rating: report_data["executive_summary"]["security_rating"]
                            .as_str()
                            .unwrap_or("Unknown")
                            .to_string(),
                        critical_vulnerabilities:
                            report_data["executive_summary"]["critical_vulnerabilities"]
                                .as_u64()
                                .unwrap_or(0) as u32,
                        overall_robustness_score: extract_robustness_score(output_data)
                            .unwrap_or(0.0),
                        tests_analyzed: extract_tests_analyzed(output_data).unwrap_or(0),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .take(10) // Show last 10 reports
        .collect();

    let total_reports = vulnerability_reports.len() as u32;

    let reports_page_ui = ReportsPageUi::new(user, vulnerability_reports, total_reports);
    let html = reports_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Helper function to extract robustness score from nested JSON
fn extract_robustness_score(output_data: &str) -> Option<f64> {
    if let Ok(report_data) = serde_json::from_str::<serde_json::Value>(output_data) {
        // Try different paths where robustness score might be
        if let Some(score) =
            report_data["technical_appendices"]["raw_data_summary"]["robustness_score"].as_f64()
        {
            return Some(score);
        }
        // Check if it's embedded in the robustness_data_json
        if let Some(robustness_json) =
            report_data["technical_appendices"]["raw_data_summary"]["robustness_data_json"].as_str()
        {
            if let Ok(robustness_data) = serde_json::from_str::<serde_json::Value>(robustness_json)
            {
                if let Some(score) = robustness_data["overall_robustness_score"].as_f64() {
                    return Some(score);
                }
            }
        }
        // Fallback: try to infer from security rating
        if let Some(rating) = report_data["executive_summary"]["security_rating"].as_str() {
            return Some(match rating {
                "Excellent" => 95.0,
                "Good" => 80.0,
                "Fair" => 65.0,
                "Poor" => 45.0,
                "Critical" => 25.0,
                _ => 50.0,
            });
        }
    }
    None
}

/// Helper function to extract tests analyzed count
fn extract_tests_analyzed(output_data: &str) -> Option<u32> {
    if let Ok(report_data) = serde_json::from_str::<serde_json::Value>(output_data) {
        // Try different paths where tests count might be
        if let Some(count) =
            report_data["technical_appendices"]["raw_data_summary"]["total_tests"].as_u64()
        {
            return Some(count as u32);
        }
        // Check embedded robustness data
        if let Some(robustness_json) =
            report_data["technical_appendices"]["raw_data_summary"]["robustness_data_json"].as_str()
        {
            if let Ok(robustness_data) = serde_json::from_str::<serde_json::Value>(robustness_json)
            {
                if let Some(count) = robustness_data["tests_analyzed"].as_u64() {
                    return Some(count as u32);
                }
            }
        }
    }
    Some(5) // Default fallback
}
