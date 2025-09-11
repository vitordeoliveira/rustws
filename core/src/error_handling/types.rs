//! Core error types

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    #[error("Authentication error: {message}")]
    Authentication { message: String },

    #[error("Authorization error: {message}")]
    Authorization { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Internal server error: {message}")]
    Internal { 
        message: String,
        error_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub error_id: String,
    pub timestamp: String,
}

impl AppError {
    pub fn validation(message: &str) -> Self {
        Self::Validation {
            message: message.to_string(),
            field: None,
        }
    }

    pub fn not_found(resource: &str) -> Self {
        Self::NotFound {
            resource: resource.to_string(),
        }
    }

    pub fn internal(message: &str) -> Self {
        Self::Internal {
            message: message.to_string(),
            error_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Create a validation error with context and automatic logging
    pub fn validation_with_context(
        message: &str,
        field: Option<&str>,
        context: &str,
    ) -> Self {
        error!(
            message = %message,
            field = ?field,
            context = %context,
            "Validation error occurred"
        );
        
        Self::Validation {
            message: message.to_string(),
            field: field.map(|f| f.to_string()),
        }
    }

    /// Create an authentication error with context and automatic logging
    pub fn authentication_with_context(
        message: &str,
        user_context: Option<&str>,
        context: &str,
    ) -> Self {
        error!(
            message = %message,
            user_context = ?user_context,
            context = %context,
            "Authentication error occurred"
        );
        
        Self::Authentication {
            message: message.to_string(),
        }
    }

    /// Create an authorization error with context and automatic logging
    pub fn authorization_with_context(
        message: &str,
        resource: Option<&str>,
        user_context: Option<&str>,
        context: &str,
    ) -> Self {
        error!(
            message = %message,
            resource = ?resource,
            user_context = ?user_context,
            context = %context,
            "Authorization error occurred"
        );
        
        Self::Authorization {
            message: message.to_string(),
        }
    }

    /// Create a not found error with context and automatic logging
    pub fn not_found_with_context(
        resource: &str,
        search_context: Option<&str>,
        context: &str,
    ) -> Self {
        error!(
            resource = %resource,
            search_context = ?search_context,
            context = %context,
            "Resource not found"
        );
        
        Self::NotFound {
            resource: resource.to_string(),
        }
    }

    /// Create a database error with context and automatic logging
    pub fn database_with_context(
        message: &str,
        operation: Option<&str>,
        table: Option<&str>,
        context: &str,
    ) -> Self {
        error!(
            message = %message,
            operation = ?operation,
            table = ?table,
            context = %context,
            "Database error occurred"
        );
        
        Self::Database {
            message: message.to_string(),
        }
    }

    /// Create an internal error with context and automatic logging
    pub fn internal_with_context(
        message: &str,
        operation: Option<&str>,
        context: &str,
    ) -> Self {
        let error_id = uuid::Uuid::new_v4().to_string();
        
        error!(
            message = %message,
            operation = ?operation,
            error_id = %error_id,
            context = %context,
            "Internal server error occurred"
        );
        
        Self::Internal {
            message: message.to_string(),
            error_id,
        }
    }

    /// Create a configuration error with path context and automatic logging
    pub fn configuration_with_path(
        message: &str,
        path: &std::path::Path,
        context: &str,
    ) -> Self {
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("<unknown>"));
        let full_path = current_dir.join(path);
        
        let detailed_message = format!(
            "{}. Path: '{}', Current dir: '{}', Full path: '{}', Exists: {}",
            message,
            path.display(),
            current_dir.display(),
            full_path.display(),
            full_path.exists()
        );
        
        error!(
            message = %message,
            path = %path.display(),
            current_dir = %current_dir.display(),
            full_path = %full_path.display(),
            exists = %full_path.exists(),
            context = %context,
            "Configuration error with path context"
        );
        
        Self::Configuration {
            message: detailed_message,
        }
    }

    /// Create a configuration error from an IO error with automatic logging
    pub fn configuration_from_io_error(
        io_error: std::io::Error,
        path: &std::path::Path,
        context: &str,
    ) -> Self {
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("<unknown>"));
        let full_path = current_dir.join(path);
        
        error!(
            io_error = %io_error,
            path = %path.display(),
            current_dir = %current_dir.display(),
            full_path = %full_path.display(),
            exists = %full_path.exists(),
            context = %context,
            "IO error while accessing configuration file"
        );
        
        Self::Configuration {
            message: format!(
                "IO error: {}. Path: '{}', Current dir: '{}', Full path: '{}', Exists: {}",
                io_error,
                path.display(),
                current_dir.display(),
                full_path.display(),
                full_path.exists()
            ),
        }
    }

    /// Create a configuration error from a parse error with automatic logging
    pub fn configuration_from_parse_error<E: std::fmt::Display>(
        parse_error: E,
        file_path: &str,
        context: &str,
    ) -> Self {
        error!(
            parse_error = %parse_error,
            file_path = %file_path,
            context = %context,
            "Failed to parse configuration file"
        );
        
        Self::Configuration {
            message: format!("Failed to parse config file '{}': {}", file_path, parse_error),
        }
    }

    /// Create a configuration error from environment variable issues with automatic logging
    pub fn configuration_from_env_error(
        env_var: &str,
        value: Option<&str>,
        error_message: &str,
        context: &str,
    ) -> Self {
        error!(
            env_var = %env_var,
            value = ?value,
            error_message = %error_message,
            context = %context,
            "Environment variable configuration error"
        );
        
        Self::Configuration {
            message: format!("Environment variable '{}' error: {}", env_var, error_message),
        }
    }

    /// Create a configuration error from a generic error with automatic logging
    pub fn configuration_from_error<E: std::fmt::Display>(
        error: E,
        context: &str,
    ) -> Self {
        error!(
            error = %error,
            context = %context,
            "Configuration error occurred"
        );
        
        Self::Configuration {
            message: error.to_string(),
        }
    }

    /// Add additional context to an existing AppError with logging
    pub fn with_additional_context(self, additional_context: &str) -> Self {
        match &self {
            Self::Validation { message, field } => {
                error!(
                    error_type = "Validation",
                    message = %message,
                    field = ?field,
                    additional_context = %additional_context,
                    "Validation error with additional context"
                );
            }
            Self::Authentication { message } => {
                error!(
                    error_type = "Authentication",
                    message = %message,
                    additional_context = %additional_context,
                    "Authentication error with additional context"
                );
            }
            Self::Authorization { message } => {
                error!(
                    error_type = "Authorization", 
                    message = %message,
                    additional_context = %additional_context,
                    "Authorization error with additional context"
                );
            }
            Self::NotFound { resource } => {
                error!(
                    error_type = "NotFound",
                    resource = %resource,
                    additional_context = %additional_context,
                    "Not found error with additional context"
                );
            }
            Self::Database { message } => {
                error!(
                    error_type = "Database",
                    message = %message,
                    additional_context = %additional_context,
                    "Database error with additional context"
                );
            }
            Self::Configuration { message } => {
                error!(
                    error_type = "Configuration",
                    message = %message,
                    additional_context = %additional_context,
                    "Configuration error with additional context"
                );
            }
            Self::Internal { message, error_id } => {
                error!(
                    error_type = "Internal",
                    message = %message,
                    error_id = %error_id,
                    additional_context = %additional_context,
                    "Internal error with additional context"
                );
            }
        }
        self
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Validation { .. } => "VALIDATION_ERROR",
            Self::Authentication { .. } => "AUTHENTICATION_ERROR",
            Self::Authorization { .. } => "AUTHORIZATION_ERROR",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Database { .. } => "DATABASE_ERROR",
            Self::Configuration { .. } => "CONFIGURATION_ERROR",
            Self::Internal { .. } => "INTERNAL_SERVER_ERROR",
        }
    }

    pub fn status_code(&self) -> u16 {
        match self {
            Self::Validation { .. } => 400,
            Self::Authentication { .. } => 401,
            Self::Authorization { .. } => 403,
            Self::NotFound { .. } => 404,
            Self::Database { .. } => 500,
            Self::Configuration { .. } => 500,
            Self::Internal { .. } => 500,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>; 