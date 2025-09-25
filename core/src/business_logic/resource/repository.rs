//! Resource repository trait for managing RUSTWS resources

use super::dto::Resource;
use crate::error_handling::types::AppResult;

/// Resource repository trait for managing RUSTWS resources
///
/// Following RUSTWS clean architecture principles:
/// - Repository handles data access and resource management
/// - Clean separation from business logic
/// - Consistent error handling with AppResult
/// - Proper instrumentation should be added to implementations
pub trait ResourceRepository: Send + Sync {
    /// Get all resources in the system
    /// Returns all available resources across all service types
    async fn get_all_resources(&self) -> AppResult<Vec<Resource>>;

    /// Get all lambda resources
    /// Returns only resources of type Lambda
    async fn get_lambda_resources(&self) -> AppResult<Vec<Resource>>;

    /// Get all workflow resources
    /// Returns only resources of type Workflow
    async fn get_workflow_resources(&self) -> AppResult<Vec<Resource>>;
}
