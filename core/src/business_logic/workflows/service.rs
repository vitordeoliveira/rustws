//! Workflow service for business logic orchestration

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use tracing::instrument;

use super::dto::{ExecuteWorkflowRequest, ExecuteWorkflowResponse, WorkflowSummary};
use super::repository::WorkflowRepository;
use crate::error_handling::types::{AppError, AppResult};
use crate::infrastructure::step_functions::StepFunctionStorage;
use crate::state::AppState;

/// Workflow service for managing step function workflows
pub(crate) struct WorkflowsService<R> {
    repository: R,
}

impl<R> WorkflowsService<R>
where
    R: WorkflowRepository,
{
    /// Create a new workflow service with the given repository
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Get all workflow summaries
    #[instrument(skip_all, fields(service = "workflows", operation = "get_all"))]
    pub async fn get_all(&self) -> AppResult<Vec<WorkflowSummary>> {
        tracing::info!("Fetching all workflow summaries");
        let summaries = self.repository.get_all().await?;
        tracing::info!(count = summaries.len(), "Retrieved workflow summaries");
        Ok(summaries)
    }

    // Future methods will be added here as repository methods are uncommented
    //
    // /// Get full workflow definition by name
    // #[instrument(skip_all, fields(service = "workflows", operation = "get_by_name", workflow_name = %name))]
    // pub async fn get_by_name(&self, name: &str) -> AppResult<Option<Workflow>> {
    //     self.repository.get_by_name(name).await
    // }
    //
    // /// Create a new workflow
    // #[instrument(skip_all, fields(service = "workflows", operation = "create"))]
    // pub async fn create(&self, request: CreateWorkflowRequest) -> AppResult<CreateWorkflowResponse> {
    //     self.repository.create(request).await
    // }
    //
    // /// Update an existing workflow
    // #[instrument(skip_all, fields(service = "workflows", operation = "update", workflow_name = %workflow_name))]
    // pub async fn update(&self, workflow_name: &str, request: UpdateWorkflowRequest) -> AppResult<()> {
    //     self.repository.update(workflow_name, request).await
    // }
    //
    // /// Delete a workflow by name
    // #[instrument(skip_all, fields(service = "workflows", operation = "delete", workflow_name = %workflow_name))]
    // pub async fn delete(&self, workflow_name: &str) -> AppResult<()> {
    //     self.repository.delete(workflow_name).await
    // }
    //
    // /// Validate workflow structure and dependencies
    // #[instrument(skip_all, fields(service = "workflows", operation = "validate", workflow_name = %workflow_name))]
    // pub async fn validate(&self, workflow_name: &str) -> AppResult<WorkflowValidationResult> {
    //     self.repository.validate(workflow_name).await
    // }
    //
    /// Execute a workflow with input data
    #[instrument(skip_all, fields(service = "workflows", operation = "execute"))]
    pub async fn execute(
        &mut self,
        request: ExecuteWorkflowRequest,
    ) -> AppResult<ExecuteWorkflowResponse> {
        tracing::info!(workflow_name = %request.workflow_name, "Executing workflow through service");
        self.repository.execute(request).await
    }
}

/// Extract WorkflowsService directly from request using FromRequestParts
impl FromRequestParts<AppState> for WorkflowsService<StepFunctionStorage> {
    type Rejection = AppError;

    #[instrument(
        skip_all,
        fields(service = "workflows", operation = "from_request_parts")
    )]
    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let step_function_storage = StepFunctionStorage::new();
        Ok(Self::new(step_function_storage))
    }
}
