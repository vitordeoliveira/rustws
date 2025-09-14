//! Lambda API handlers

use axum::{
    extract::{Path, State},
    response::Json,
};
use axum_login::AuthSession;
use tracing::{error, info, instrument, warn};

use crate::{
    auth::dto::AuthBackend,
    business_logic::lambdas::{
        CompileLambdaRequest, CompileLambdaResponse, CreateLambdaRequest, CreateLambdaResponse,
        ExecuteLambdaRequest, ExecuteLambdaResponse, LambdasService, UpdateLambdaRequest,
        UpdateLambdaResponse,
    },
    error_handling::types::AppResult,
    infrastructure::lambdas::LambdaStorage,
    state::AppState,
};

/// Compile a lambda function from source code to WASM
#[instrument(
    skip_all,
    fields(
        handler = "compile_lambda", 
        operation = "api_compile",
        lambda_name = %lambda_name,
    )
)]
pub async fn compile_lambda_handler(
    State(_state): State<AppState>,
    _auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Path(lambda_name): Path<String>,
) -> AppResult<Json<CompileLambdaResponse>> {
    // Record user info in tracing span

    info!(
        lambda_name = %lambda_name,
        "Lambda compilation request received"
    );

    // Validate lambda name
    if lambda_name.trim().is_empty() {
        warn!(
            lambda_name = %lambda_name,
            "Invalid lambda name provided - empty or whitespace"
        );
        return Err(crate::error_handling::types::AppError::validation(
            "Lambda name cannot be empty or contain only whitespace",
        ));
    }

    // Create request from path parameter
    let request = CompileLambdaRequest {
        lambda_name: lambda_name.clone(),
    };

    info!(
        lambda_name = %lambda_name,
        "Starting lambda compilation process"
    );

    // Use the service to compile the lambda
    match lambda_service.compile(request).await {
        Ok(response) => {
            if response.success {
                info!(
                    lambda_name = %lambda_name,
                    wasm_size_bytes = response.wasm_size_bytes,
                    compilation_time_ms = response.compilation_time_ms,
                    wasm_path = %response.wasm_path.as_deref().unwrap_or("unknown"),
                    "Lambda compilation completed successfully"
                );
            } else {
                warn!(
                    lambda_name = %lambda_name,
                    compilation_time_ms = response.compilation_time_ms,
                    error_message = %response.message,
                    "Lambda compilation failed"
                );
            }
            Ok(Json(response))
        }
        Err(e) => {
            error!(
                lambda_name = %lambda_name,
                error = %e,
                "Lambda compilation service error"
            );
            Err(e)
        }
    }
}

/// Execute a lambda function
#[instrument(
    skip_all,
    fields(
        handler = "execute_lambda", 
        operation = "api_execute",
        lambda_name = %lambda_name
    )
)]
pub async fn execute_lambda_handler(
    State(_state): State<AppState>,
    _auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Path(lambda_name): Path<String>,
) -> AppResult<Json<ExecuteLambdaResponse>> {
    info!(
        lambda_name = %lambda_name,
        "Lambda execution request received"
    );

    // Validate lambda name
    if lambda_name.trim().is_empty() {
        warn!(
            lambda_name = %lambda_name,
            "Invalid lambda name provided - empty or whitespace"
        );
        return Err(crate::error_handling::types::AppError::validation(
            "Lambda name cannot be empty or contain only whitespace",
        ));
    }

    // Create sample HelloWorld input for testing
    let sample_input = serde_json::json!({
        "text": "Test input from API",
        "count": 42
    });

    let input_data = serde_json::to_vec(&sample_input).map_err(|e| {
        crate::error_handling::types::AppError::internal(&format!(
            "Failed to serialize input: {}",
            e
        ))
    })?;

    let request = ExecuteLambdaRequest {
        function_name: Some(lambda_name.clone()),
        input_data,
    };

    info!(
        lambda_name = %lambda_name,
        "Starting lambda execution process"
    );

    // Use the service to execute the lambda
    match lambda_service.execute(request).await {
        Ok(response) => {
            match &response {
                ExecuteLambdaResponse::Success { output_data } => {
                    info!(
                        lambda_name = %lambda_name,
                        output_size_bytes = output_data.len(),
                        "Lambda execution completed successfully"
                    );
                }
                ExecuteLambdaResponse::Failed { error_message } => {
                    warn!(
                        lambda_name = %lambda_name,
                        error_message = %error_message,
                        "Lambda execution failed"
                    );
                }
            }
            Ok(Json(response))
        }
        Err(e) => {
            error!(
                lambda_name = %lambda_name,
                error = %e,
                "Lambda execution service error"
            );
            Err(e)
        }
    }
}

/// Create a new lambda function
#[instrument(skip_all, fields(handler = "create_lambda", operation = "api_create"))]
pub async fn create_lambda_handler(
    State(_state): State<AppState>,
    _auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Json(request): Json<CreateLambdaRequest>,
) -> AppResult<Json<CreateLambdaResponse>> {
    info!(
        function_name = %request.function_name,
        runtime = %request.runtime,
        "Lambda creation request received"
    );

    // Validate request
    if request.function_name.trim().is_empty() {
        warn!(
            function_name = %request.function_name,
            "Invalid function name provided - empty or whitespace"
        );
        return Err(crate::error_handling::types::AppError::validation(
            "Function name cannot be empty or contain only whitespace",
        ));
    }

    if request.source_code.trim().is_empty() {
        warn!(
            function_name = %request.function_name,
            "No source code provided"
        );
        return Err(crate::error_handling::types::AppError::validation(
            "Source code cannot be empty",
        ));
    }

    info!(
        function_name = %request.function_name,
        "Starting lambda creation process"
    );

    // Use the service to create the lambda
    match lambda_service.create(request).await {
        Ok(response) => {
            if response.success {
                info!(
                    function_name = %response.function_name,
                    source_path = ?response.source_path,
                    wasm_path = ?response.wasm_path,
                    wasm_size = ?response.wasm_size_bytes,
                    compilation_time_ms = ?response.compilation_time_ms,
                    "Lambda creation completed successfully"
                );
            } else {
                warn!(
                    function_name = %response.function_name,
                    message = %response.message,
                    "Lambda creation completed with issues"
                );
            }
            Ok(Json(response))
        }
        Err(e) => {
            error!(
                error = %e,
                "Lambda creation service error"
            );
            Err(e)
        }
    }
}

/// Update an existing lambda function
#[instrument(
    skip_all,
    fields(
        handler = "update_lambda", 
        operation = "api_update",
        lambda_name = %lambda_name
    )
)]
pub async fn update_lambda_handler(
    State(_state): State<AppState>,
    _auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Path(lambda_name): Path<String>,
    Json(request): Json<UpdateLambdaRequest>,
) -> AppResult<Json<UpdateLambdaResponse>> {
    info!(
        lambda_name = %lambda_name,
        has_source_code = request.source_code.is_some(),
        "Lambda update request received"
    );

    // Validate lambda name
    if lambda_name.trim().is_empty() {
        warn!(
            lambda_name = %lambda_name,
            "Invalid lambda name provided - empty or whitespace"
        );
        return Err(crate::error_handling::types::AppError::validation(
            "Lambda name cannot be empty or contain only whitespace",
        ));
    }

    // Validate source code if provided
    if let Some(ref source_code) = request.source_code {
        if source_code.trim().is_empty() {
            warn!(
                lambda_name = %lambda_name,
                "Empty source code provided for update"
            );
            return Err(crate::error_handling::types::AppError::validation(
                "Source code cannot be empty",
            ));
        }
    }

    info!(
        lambda_name = %lambda_name,
        "Starting lambda update process"
    );

    // Use the service to update the lambda
    match lambda_service.update(&lambda_name, request).await {
        Ok(()) => {
            info!(
                lambda_name = %lambda_name,
                "Lambda update completed successfully"
            );

            let response = UpdateLambdaResponse {
                success: true,
                message: format!("Lambda function '{}' updated successfully", lambda_name),
                function_name: lambda_name,
            };

            Ok(Json(response))
        }
        Err(e) => {
            error!(
                lambda_name = %lambda_name,
                error = %e,
                "Lambda update service error"
            );

            // Convert service error to update response for consistent API
            let response = UpdateLambdaResponse {
                success: false,
                message: format!("Failed to update lambda '{}': {}", lambda_name, e),
                function_name: lambda_name,
            };

            Ok(Json(response))
        }
    }
}
