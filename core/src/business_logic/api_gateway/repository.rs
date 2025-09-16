//! API Gateway repository trait for data access and management

use super::dto::ApiGatewaySummary;
use crate::error_handling::types::AppResult;

/// API Gateway repository trait for managing API Gateway instances
///
/// Following RUSTWS clean architecture principles:
/// - Repository handles data access and API Gateway operations
/// - Clean separation from business logic and infrastructure
/// - Async operations for scalable data access
/// - Thread-safe design with Send + Sync
pub trait ApiGatewayRepository: Send + Sync {
    /// Get all API Gateway summaries
    /// Returns lightweight API Gateway information for listing/dashboard views
    async fn get_api_gateways(&self) -> AppResult<Vec<ApiGatewaySummary>>;
}
