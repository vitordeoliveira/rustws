//! Lambda service for business logic orchestration

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use super::dto::LambdaSummary;
use super::repository::LambdaRepository;
use crate::error_handling::types::{AppError, AppResult};
use crate::infrastructure::lambdas::LambdaStorage;
use crate::state::AppState;

/// Lambda service for managing lambda functions
pub(crate) struct LambdasService<R> {
    repository: R,
}

impl<R> LambdasService<R>
where
    R: LambdaRepository,
{
    /// Create a new lambda service with the given repository
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Get all lambda functions
    pub async fn get_all(&self) -> AppResult<Vec<LambdaSummary>> {
        self.repository.get_all().await
    }
}

/// Extract LambdasService directly from request using FromRequestParts
impl FromRequestParts<AppState> for LambdasService<LambdaStorage> {
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let lambda_storage = LambdaStorage::new();
        Ok(Self::new(lambda_storage))
    }
}
