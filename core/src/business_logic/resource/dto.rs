//! Resource Data Transfer Objects

use serde::{Deserialize, Serialize};

/// Resource identifier for RUSTWS services
/// Handles URN format: "rustws:lambda:hello_world"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Namespace/system prefix (e.g., "rustws")
    pub namespace: String,
    /// Service type (e.g., "lambda", "workflow")
    pub service_type: String,
    /// Specific resource name (e.g., "hello_world")
    pub resource_name: String,
}

impl Resource {
    /// Parse a resource URN string and extract the lambda name
    /// Expected format: "rustws:lambda:function_name"
    /// Returns just the function name part
    pub fn parse_lambda_name(resource_urn: &str) -> Option<String> {
        let trimmed = resource_urn.trim();

        // If no colons, treat as direct lambda name
        if !trimmed.contains(':') {
            return Some(trimmed.to_string());
        }

        // Split by ':' and validate the expected format: rustws:lambda:function_name
        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() == 3 && parts[0] == "rustws" && parts[1] == "lambda" {
            Some(parts[2].trim().to_string())
        } else {
            None
        }
    }
}
