//! Route definitions and smart routing

use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{StatusCode, header},
    middleware,
    response::Response,
    routing::{get, post, put},
};
use axum_login::login_required;
use std::path::Path;
use tower_http::{services::ServeDir, trace::TraceLayer};

use tracing::error_span;

use crate::{
    auth::{SessionManager, dto::AuthBackend},
    error_handling::AppResult,
    handlers::{api, pages},
    routing::{RequestId, request_id_middleware},
    state::AppState,
};

pub fn create_private_router() -> Router<AppState> {
    Router::new()
        .route("/", get(pages::home_handler))
        .route("/api-gateway", get(pages::api_gateway_handler))
        .route("/lambda", get(pages::lambda_handler))
        .route("/monitoring", get(pages::monitoring_handler))
        .route("/step-functions", get(pages::step_functions_handler))
        .route(
            "/step-functions/create",
            get(pages::create_step_function_handler).post(pages::create_workflow_form_handler),
        )
        .route(
            "/step-functions/edit/{workflow_name}",
            get(pages::edit_step_function_handler),
        )
        .route("/lambda/create", get(pages::create_lambda_handler))
        .route(
            "/lambda/edit/{lambda_name}",
            get(pages::edit_lambda_handler),
        )
        .route(
            "/lambda/metrics/{lambda_name}",
            get(pages::lambda_metrics_handler),
        )
        .route(
            "/lambda/delete/{lambda_name}",
            post(pages::delete_lambda_handler),
        )
        .route(
            "/step-functions/delete/{workflow_name}",
            post(pages::delete_step_function_handler),
        )
        .route(
            "/api/lambda/compile/{lambda_name}",
            post(api::lambdas::compile_lambda_handler),
        )
        .route(
            "/api/lambda/execute/{lambda_name}",
            post(api::lambdas::execute_lambda_handler),
        )
        .route(
            "/api/lambda/create",
            post(api::lambdas::create_lambda_handler),
        )
        .route(
            "/api/lambda/update/{lambda_name}",
            put(api::lambdas::update_lambda_handler),
        )
        .route(
            "/api/lambda/metrics",
            get(api::lambdas::get_lambda_metrics_handler),
        )
        .route(
            "/api/lambda/metadata/{lambda_name}",
            get(api::lambdas::get_lambda_metadata_handler),
        )
        .route(
            "/api/workflow/execute/{workflow_name}",
            post(api::workflows::execute_workflow_handler),
        )
        .route(
            "/api/workflow/definition/{workflow_name}",
            get(api::workflows::get_workflow_definition_handler),
        )
        .route(
            "/api/workflow/update/{workflow_name}",
            put(api::workflows::update_workflow_handler),
        )
        .route(
            "/api/workflow/graph",
            post(api::workflows::generate_workflow_graph_handler),
        )
        .route(
            "/step-functions/react/bundle.js",
            get(serve_step_functions_bundle),
        )
        .route_layer(login_required!(AuthBackend, login_url = "/login"))
}

pub fn create_auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", get(pages::login_handler))
        .route("/auth/login", post(pages::login_form_handler))
        .route("/auth/logout", post(pages::logout_handler))
}

/// Creates the application router with all routes
pub async fn create_router(state: AppState, session_manager: SessionManager) -> AppResult<Router> {
    let router = Router::new()
        .merge(create_private_router())
        .merge(create_auth_router())
        .nest_service("/assets", ServeDir::new("assets/public"))
        .layer(session_manager)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                let request_id = request
                    .extensions()
                    .get::<RequestId>()
                    .map(|r| r.0.clone())
                    .unwrap_or_else(|| "unknown".into());
                error_span!(
                    "request",
                    id = %request_id,
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        )
        // Add request ID middleware (should be one of the first layers)
        .layer(middleware::from_fn(request_id_middleware))
        .with_state(state);

    Ok(router)
}

/// Serve the React bundle for step functions graph
async fn serve_step_functions_bundle() -> Result<Response<String>, StatusCode> {
    let bundle_path = Path::new("src/ui/step_functions/react/dist/bundle.js");

    match tokio::fs::read_to_string(bundle_path).await {
        Ok(content) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/javascript")
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(content)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(response)
        }
        Err(_) => {
            // Return a minimal JavaScript that logs an error
            let fallback_js = r#"
                console.error('Step Functions React bundle not found. Please run: cd src/ui/step_functions/react && npm install && npm run build');
                window.initStepFunctionsGraph = function() { 
                    console.error('React bundle not available'); 
                };
            "#;

            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/javascript")
                .body(fallback_js.to_string())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(response)
        }
    }
}
