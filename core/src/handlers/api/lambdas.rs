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
        CompileLambdaRequest, CompileLambdaResponse, ExecuteLambdaRequest, ExecuteLambdaResponse,
        LambdasService,
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
    auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Path(lambda_name): Path<String>,
) -> AppResult<Json<CompileLambdaResponse>> {
    let user = auth_session.user.unwrap(); // Ensure authenticated

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
    auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Path(lambda_name): Path<String>,
) -> AppResult<Json<ExecuteLambdaResponse>> {
    let user = auth_session.user.unwrap(); // Ensure authenticated

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

    // Create request from path parameter - no input data for GET requests
    let request = ExecuteLambdaRequest {
        function_name: Some(lambda_name.clone()),
        input_data: Vec::new(), // Empty input for GET requests
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
