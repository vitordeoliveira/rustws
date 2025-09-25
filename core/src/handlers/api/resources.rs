//! Resources API handlers

use axum::response::Json;
use tracing::instrument;

use crate::{
    business_logic::resource::{Resource, ResourceService},
    error_handling::types::AppResult,
    infrastructure::resources::ResourceStorage,
};

/// Get all resources in the system
#[instrument(
    skip_all,
    fields(handler = "get_all_resources", operation = "api_get_all_resources")
)]
pub async fn get_all_resources_handler(
    resource_service: ResourceService<ResourceStorage>,
) -> AppResult<Json<Vec<Resource>>> {
    tracing::info!("API request to get all resources");

    let resources = resource_service.get_all_resources().await?;

    tracing::info!(
        count = resources.len(),
        "Retrieved all resources for API response"
    );
    Ok(Json(resources))
}
