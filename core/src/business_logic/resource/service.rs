//! Resource service implementation
//!
//! This module provides business logic for resource management.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use tracing::instrument;

use super::{Resource, ResourceRepository};
use crate::error_handling::types::{AppError, AppResult};
use crate::infrastructure::resources::ResourceStorage;
use crate::state::AppState;

/// Resource service for managing RUSTWS resources
pub(crate) struct ResourceService<R> {
    repository: R,
}

impl<R> ResourceService<R>
where
    R: ResourceRepository,
{
    /// Create a new resource service with the given repository
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Get all resources in the system
    #[instrument(skip_all, fields(operation = "get_all_resources"))]
    pub async fn get_all_resources(&self) -> AppResult<Vec<Resource>> {
        tracing::info!("Service: Fetching all resources");
        self.repository.get_all_resources().await
    }

    /// Get all lambda resources
    #[instrument(skip_all, fields(operation = "get_lambda_resources"))]
    pub async fn get_lambda_resources(&self) -> AppResult<Vec<Resource>> {
        tracing::info!("Service: Fetching lambda resources");
        self.repository.get_lambda_resources().await
    }

    /// Get all workflow resources
    #[instrument(skip_all, fields(operation = "get_workflow_resources"))]
    pub async fn get_workflow_resources(&self) -> AppResult<Vec<Resource>> {
        tracing::info!("Service: Fetching workflow resources");
        self.repository.get_workflow_resources().await
    }
}

/// Extract ResourceService directly from request using FromRequestParts
impl FromRequestParts<AppState> for ResourceService<ResourceStorage> {
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let resource_storage = ResourceStorage::new();
        Ok(Self::new(resource_storage))
    }
}
