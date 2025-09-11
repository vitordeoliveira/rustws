//! Request middleware for matrix interface

use axum::{
    extract::{Request, State},
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use uuid::Uuid;

use crate::{error_handling::AppResult, state::AppState};

/// Request ID wrapper for use in handlers
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Middleware that generates a unique request ID for each request
///
/// This middleware:
/// - Generates a UUID for each incoming request
/// - Stores it in request extensions for handler access
/// - Adds it to response headers following industry standards
/// - Enables request/error correlation throughout the request lifecycle
// #[instrument(skip_all, fields(operation = "request_id_middleware"))]
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Generate unique request ID
    let request_id = Uuid::new_v4().to_string();

    // Store request ID in request extensions for handler access
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // Process the request
    let mut response = next.run(request).await;

    // Add request ID to response headers (industry standard)
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-request-id"), header_value);
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, middleware::from_fn, routing::get, Extension, Router};
    use tower::ServiceExt;

    async fn test_handler(Extension(request_id): Extension<RequestId>) -> String {
        format!("Request ID: {}", request_id.0)
    }

    #[tokio::test]
    async fn test_request_id_middleware() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(from_fn(request_id_middleware));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Check that x-request-id header is present
        assert!(response.headers().contains_key("x-request-id"));

        // Check that the response status is success
        assert!(response.status().is_success());
    }
}
