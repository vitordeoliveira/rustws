//! Resource storage implementation
//!
//! This module provides the core storage functionality for RUSTWS resources
//! and implements the ResourceRepository trait.

use std::path::PathBuf;
use tracing::instrument;

use crate::business_logic::resource::{Resource, ResourceRepository, ServiceType};
use crate::error_handling::types::{AppError, AppResult};

/// Resource storage implementation
#[derive(Debug, Clone)]
pub struct ResourceStorage {
    /// Base path for resource storage
    base_path: PathBuf,
}

impl ResourceStorage {
    /// Create new resource storage manager
    #[instrument(skip_all, fields(operation = "create_resource_storage"))]
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("src/infrastructure"),
        }
    }

    /// Get the base path for resource storage
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }
}

/// Implementation of ResourceRepository trait for ResourceStorage
impl ResourceRepository for ResourceStorage {
    /// Get all resources in the system
    #[instrument(skip_all, fields(operation = "get_all_resources"))]
    async fn get_all_resources(&self) -> AppResult<Vec<Resource>> {
        tracing::info!("Fetching all resources from storage");

        let mut all_resources = Vec::new();

        // Get lambda resources
        let lambda_resources = self.get_lambda_resources().await?;
        all_resources.extend(lambda_resources);

        // Get workflow resources
        let workflow_resources = self.get_workflow_resources().await?;
        all_resources.extend(workflow_resources);

        tracing::info!(total_count = all_resources.len(), "Retrieved all resources");
        Ok(all_resources)
    }

    /// Get all lambda resources
    #[instrument(skip_all, fields(operation = "get_lambda_resources"))]
    async fn get_lambda_resources(&self) -> AppResult<Vec<Resource>> {
        tracing::info!("Scanning lambda infrastructure for resources");

        let lambda_path = self.base_path.join("lambdas/source");
        let wasm_path = self.base_path.join("lambdas/wasm");
        let mut resources = Vec::new();

        if lambda_path.exists() {
            let entries = std::fs::read_dir(&lambda_path).map_err(|e| {
                AppError::internal(&format!("Failed to read lambda directory: {}", e))
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    AppError::internal(&format!("Failed to read lambda directory entry: {}", e))
                })?;
                let path = entry.path();

                if path.is_file() {
                    // Skip mod.rs files - they are not lambda functions
                    if path.file_name().and_then(|s| s.to_str()) == Some("mod.rs") {
                        continue;
                    }

                    // Extract filename without extension as lambda name
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        // Try to read metadata from wasm directory
                        let (input_type, output_type) = self.read_lambda_metadata(&wasm_path, name);

                        let resource = Resource {
                            namespace: "rustws".to_string(),
                            service_type: ServiceType::Lambda,
                            resource_name: name.to_string(),
                            input_type,
                            output_type,
                        };
                        resources.push(resource);
                    }
                }
            }
        }

        tracing::info!(count = resources.len(), "Found lambda resources");
        Ok(resources)
    }

    /// Get all workflow resources
    #[instrument(skip_all, fields(operation = "get_workflow_resources"))]
    async fn get_workflow_resources(&self) -> AppResult<Vec<Resource>> {
        tracing::info!("Scanning workflow infrastructure for resources");

        let workflow_path = self.base_path.join("step_functions/workflows");
        let mut resources = Vec::new();

        if workflow_path.exists() {
            let entries = std::fs::read_dir(&workflow_path).map_err(|e| {
                AppError::internal(&format!("Failed to read workflow directory: {}", e))
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    AppError::internal(&format!("Failed to read workflow directory entry: {}", e))
                })?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    // Extract filename without extension as workflow name
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        let resource = Resource {
                            namespace: "rustws".to_string(),
                            service_type: ServiceType::Workflow,
                            resource_name: name.to_string(),
                            input_type: None,
                            output_type: None,
                        };
                        resources.push(resource);
                    }
                }
            }
        }

        tracing::info!(count = resources.len(), "Found workflow resources");
        Ok(resources)
    }
}

impl ResourceStorage {
    /// Read lambda metadata from JSON file
    fn read_lambda_metadata(
        &self,
        wasm_path: &std::path::Path,
        lambda_name: &str,
    ) -> (Option<String>, Option<String>) {
        let metadata_file = wasm_path.join(format!("{}.json", lambda_name));

        if !metadata_file.exists() {
            tracing::debug!(lambda_name = %lambda_name, "No metadata file found");
            return (None, None);
        }

        match std::fs::read_to_string(&metadata_file) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(metadata) => {
                    let input_type = metadata
                        .get("input_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let output_type = metadata
                        .get("output_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    tracing::debug!(
                        lambda_name = %lambda_name,
                        input_type = ?input_type,
                        output_type = ?output_type,
                        "Read lambda metadata"
                    );

                    (input_type, output_type)
                }
                Err(e) => {
                    tracing::warn!(
                        lambda_name = %lambda_name,
                        error = %e,
                        "Failed to parse lambda metadata JSON"
                    );
                    (None, None)
                }
            },
            Err(e) => {
                tracing::warn!(
                    lambda_name = %lambda_name,
                    error = %e,
                    "Failed to read lambda metadata file"
                );
                (None, None)
            }
        }
    }
}
