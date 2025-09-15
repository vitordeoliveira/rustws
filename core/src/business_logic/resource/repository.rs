//! Resource repository trait

use super::dto::Resource;
use crate::error_handling::types::AppResult;

/// Repository trait for resource management
pub trait ResourceRepository: Send + Sync {
    /// Get all available resources in the system
    async fn get_all_resources(&self) -> AppResult<Vec<Resource>>;
}
