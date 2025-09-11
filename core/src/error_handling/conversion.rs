//! Error conversion utilities

use crate::error_handling::types::AppError;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

// Add error conversions as needed
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal {
            message: format!("IO error: {}", err),
            error_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(_err: serde_json::Error) -> Self {
        AppError::validation("Invalid JSON format")
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database {
            message: err.to_string(),
        }
    }
}

// Axum integration - convert AppError to HTTP responses
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // Log the detailed error for developers/administrators
        tracing::error!("{self}");

        match self {
            // Client errors - return specific error pages (safe to show details)
            AppError::Validation { message, .. } => (
                StatusCode::BAD_REQUEST,
                Html(format!("<h1>Validation Error</h1><p>{}</p>", message)),
            ),

            AppError::Authentication { message } => (
                StatusCode::UNAUTHORIZED,
                Html(format!("<h1>Authentication Required</h1><p>{}</p>", message)),
            ),

            AppError::Authorization { message } => (
                StatusCode::FORBIDDEN,
                Html(format!("<h1>Access Denied</h1><p>{}</p>", message)),
            ),

            AppError::NotFound { resource } => (
                StatusCode::NOT_FOUND,
                Html(format!("<h1>Not Found</h1><p>{} not found</p>", resource)),
            ),

            // Server errors - return generic error page (hide internal details)
            AppError::Database { .. }
            | AppError::Configuration { .. }
            | AppError::Internal { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Internal Server Error</h1><p>Something went wrong. Please try again later.</p>".to_string()),
            ),
        }
        .into_response()
    }
}

// Add more conversions for database errors, HTTP errors, etc. 