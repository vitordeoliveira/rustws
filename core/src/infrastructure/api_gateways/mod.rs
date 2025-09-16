//! API Gateway infrastructure implementation
//!
//! This module provides the concrete implementation of API Gateway repository

use std::fs;
use std::path::PathBuf;
use tracing::instrument;

use crate::business_logic::api_gateway::{ApiGateway, ApiGatewayRepository, ApiGatewaySummary};
use crate::error_handling::types::{AppError, AppResult};

/// API Gateway repository implementation with file-based storage
#[derive(Debug, Clone)]
pub struct ApiGatewayStorage {
    /// Base path for API Gateway storage
    base_path: PathBuf,
}

impl ApiGatewayStorage {
    /// Create new API Gateway storage manager
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("src/infrastructure/api_gateways"),
        }
    }

    /// Get path to API Gateway definitions directory
    fn gateways_dir(&self) -> PathBuf {
        self.base_path.join("gateways")
    }

    /// Get path to specific API Gateway file
    fn gateway_file_path(&self, gateway_name: &str) -> PathBuf {
        self.gateways_dir().join(format!("{}.json", gateway_name))
    }

    /// Load API Gateway from file
    #[instrument(skip_all, fields(operation = "load_gateway", gateway_name = %gateway_name))]
    async fn load_gateway(&self, gateway_name: &str) -> AppResult<Option<ApiGateway>> {
        let file_path = self.gateway_file_path(gateway_name);

        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| AppError::internal(&format!("Failed to read gateway file: {}", e)))?;

        let gateway: ApiGateway = serde_json::from_str(&content)
            .map_err(|e| AppError::validation(&format!("Failed to parse gateway JSON: {}", e)))?;

        tracing::debug!("Successfully loaded API Gateway");
        Ok(Some(gateway))
    }

    /// Get API Gateway by name
    #[instrument(skip_all, fields(operation = "get_gateway_by_name", gateway_name = %name))]
    pub async fn get_gateway_by_name(&self, name: &str) -> AppResult<Option<ApiGateway>> {
        self.load_gateway(name).await
    }

    /// Get API Gateway summary by name
    #[instrument(skip_all, fields(operation = "get_gateway_summary_by_name", gateway_name = %name))]
    pub async fn get_gateway_summary_by_name(
        &self,
        name: &str,
    ) -> AppResult<Option<ApiGatewaySummary>> {
        if let Some(gateway) = self.load_gateway(name).await? {
            Ok(Some(ApiGatewaySummary::from_api_gateway(gateway)))
        } else {
            Ok(None)
        }
    }
}

impl Default for ApiGatewayStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiGatewayRepository for ApiGatewayStorage {
    /// Get all API Gateway summaries
    #[instrument(skip_all, fields(operation = "get_all_gateways"))]
    async fn get_api_gateways(&self) -> AppResult<Vec<ApiGatewaySummary>> {
        let gateways_dir = self.gateways_dir();

        if !gateways_dir.exists() {
            tracing::warn!("Gateways directory does not exist");
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&gateways_dir).map_err(|e| {
            AppError::internal(&format!("Failed to read gateways directory: {}", e))
        })?;

        let mut summaries = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                AppError::internal(&format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let gateway_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                if let Ok(Some(gateway)) = self.load_gateway(&gateway_name).await {
                    summaries.push(ApiGatewaySummary::from_api_gateway(gateway));
                }
            }
        }

        tracing::info!(count = summaries.len(), "Retrieved API Gateway summaries");
        Ok(summaries)
    }
}
