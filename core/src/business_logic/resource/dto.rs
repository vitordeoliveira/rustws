//! Resource Data Transfer Objects

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

/// Service types available in the RUSTWS system
#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, AsRefStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ServiceType {
    Lambda,
    Workflow,
}

/// Resource identifier for RUSTWS services
/// Handles URN format: "rustws:lambda:hello_world"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Namespace/system prefix (e.g., "rustws")
    pub namespace: String,
    /// Service type (e.g., Lambda, Workflow)
    pub service_type: ServiceType,
    /// Specific resource name (e.g., "hello_world")
    pub resource_name: String,
}

impl Resource {
    /// Create a new Resource
    pub fn new(namespace: String, service_type: ServiceType, resource_name: String) -> Self {
        Self {
            namespace,
            service_type,
            resource_name,
        }
    }

    /// Parse a resource URN string and extract the lambda name
    /// Expected format: "rustws:lambda:function_name"
    /// Returns just the function name part
    pub fn parse_lambda_name(resource_urn: &str) -> Option<String> {
        use std::str::FromStr;

        let trimmed = resource_urn.trim();

        // If no colons, treat as direct lambda name
        if !trimmed.contains(':') {
            return Some(trimmed.to_string());
        }

        // Split by ':' and validate the expected format: rustws:lambda:function_name
        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() == 3 && parts[0] == "rustws" {
            // Use strum to parse the service type
            if let Ok(service_type) = ServiceType::from_str(parts[1]) {
                // Verify it's a lambda service type
                if matches!(service_type, ServiceType::Lambda) {
                    return Some(parts[2].trim().to_string());
                }
            }
        }

        None
    }
}
