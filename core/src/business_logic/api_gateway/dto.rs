//! API Gateway Data Transfer Objects

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Individual API endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: Uuid,
    pub path: String,
    pub method: HttpMethod,
    pub target: EndpointTarget,
    pub auth_required: bool,
    pub rate_limit: Option<RateLimit>,
    pub cors_enabled: bool,
    pub description: Option<String>,
    pub request_transformations: HashMap<String, String>,
    pub response_transformations: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// HTTP methods supported by endpoints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    OPTIONS,
    HEAD,
}

/// What handles this endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndpointTarget {
    /// Route to a Lambda function
    Lambda { function_name: String },
    /// Route to a Step Function workflow
    Workflow { workflow_name: String },
    /// Direct HTTP proxy to another service
    HttpProxy { url: String },
    /// Static response
    Static { response: serde_json::Value },
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst_limit: u32,
}

/// API Gateway main entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiGateway {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: ApiGatewayStatus,
    pub stage: String,
    pub base_url: String,
    pub endpoints: Vec<Endpoint>,
    pub last_deployment: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// API Gateway status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiGatewayStatus {
    Active,
    Inactive,
    Deploying,
    Warning,
    Error,
}

/// API Gateway summary for listing/dashboard views
/// Lightweight representation of API Gateway information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiGatewaySummary {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: ApiGatewayStatus,
    pub stage: String,
    pub base_url: String,
    pub endpoints_count: u32,
    pub last_deployment: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create a new API Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiGatewayRequest {
    pub name: String,
    pub description: String,
    pub stage: String,
}

/// Response for API Gateway creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiGatewayResponse {
    pub success: bool,
    pub message: String,
    pub api_gateway: Option<ApiGateway>,
}

/// Request to update an existing API Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApiGatewayRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub stage: Option<String>,
    pub status: Option<ApiGatewayStatus>,
}

/// Request to deploy an API Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployApiGatewayRequest {
    pub stage: String,
}

/// Response for API Gateway deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployApiGatewayResponse {
    pub success: bool,
    pub message: String,
    pub deployment_id: Option<String>,
    pub base_url: Option<String>,
}

/// Request to create a new endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEndpointRequest {
    pub path: String,
    pub method: HttpMethod,
    pub target: EndpointTarget,
    pub auth_required: bool,
    pub rate_limit: Option<RateLimit>,
    pub cors_enabled: bool,
    pub description: Option<String>,
    pub request_transformations: HashMap<String, String>,
    pub response_transformations: HashMap<String, String>,
}

/// Request to update an existing endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEndpointRequest {
    pub path: Option<String>,
    pub method: Option<HttpMethod>,
    pub target: Option<EndpointTarget>,
    pub auth_required: Option<bool>,
    pub rate_limit: Option<RateLimit>,
    pub cors_enabled: Option<bool>,
    pub description: Option<String>,
    pub request_transformations: Option<HashMap<String, String>>,
    pub response_transformations: Option<HashMap<String, String>>,
}

/// Response for endpoint operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointResponse {
    pub success: bool,
    pub message: String,
    pub endpoint: Option<Endpoint>,
}

impl ApiGateway {
    /// Get the number of endpoints
    pub fn endpoints_count(&self) -> u32 {
        self.endpoints.len() as u32
    }

    /// Find endpoint by ID
    pub fn find_endpoint(&self, endpoint_id: &Uuid) -> Option<&Endpoint> {
        self.endpoints.iter().find(|ep| ep.id == *endpoint_id)
    }

    /// Find endpoint by path and method
    pub fn find_endpoint_by_path(&self, path: &str, method: &HttpMethod) -> Option<&Endpoint> {
        self.endpoints
            .iter()
            .find(|ep| ep.path == path && ep.method == *method)
    }

    /// Add a new endpoint
    pub fn add_endpoint(&mut self, endpoint: Endpoint) {
        self.endpoints.push(endpoint);
        self.updated_at = chrono::Utc::now();
    }

    /// Remove endpoint by ID
    pub fn remove_endpoint(&mut self, endpoint_id: &Uuid) -> bool {
        if let Some(pos) = self.endpoints.iter().position(|ep| ep.id == *endpoint_id) {
            self.endpoints.remove(pos);
            self.updated_at = chrono::Utc::now();
            true
        } else {
            false
        }
    }
}

impl ApiGatewaySummary {
    /// Convert from full ApiGateway entity to summary
    pub fn from_api_gateway(api_gateway: ApiGateway) -> Self {
        Self {
            id: api_gateway.id,
            name: api_gateway.name,
            description: api_gateway.description,
            status: api_gateway.status,
            stage: api_gateway.stage,
            base_url: api_gateway.base_url,
            endpoints_count: api_gateway.endpoints.len() as u32,
            last_deployment: api_gateway.last_deployment,
            created_at: api_gateway.created_at,
        }
    }
}

impl Endpoint {
    /// Create a new endpoint with defaults
    pub fn new(path: String, method: HttpMethod, target: EndpointTarget) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            path,
            method,
            target,
            auth_required: false,
            rate_limit: None,
            cors_enabled: false,
            description: None,
            request_transformations: HashMap::new(),
            response_transformations: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if endpoint requires authentication
    pub fn is_authenticated(&self) -> bool {
        self.auth_required
    }

    /// Check if endpoint has rate limiting enabled
    pub fn has_rate_limit(&self) -> bool {
        self.rate_limit.is_some()
    }
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            burst_limit: 10,
        }
    }
}

impl Default for ApiGatewayStatus {
    fn default() -> Self {
        Self::Inactive
    }
}
