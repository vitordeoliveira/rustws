//! Main page handlers

use axum::{
    extract::{Path, State},
    response::Html,
};
use axum_login::AuthSession;
use tracing::instrument;

use crate::{
    auth::dto::AuthBackend,
    business_logic::lambdas::LambdasService,
    error_handling::types::AppResult,
    infrastructure::lambdas::LambdaStorage,
    state::AppState,
    ui::{
        Ui,
        home::HomePageUi,
        lambda::{CreateLambdaPageUi, EditLambdaPageUi, LambdaPageUi},
        step_functions::{CreateStepFunctionPageUi, StepFunctionsPageUi, get_mock_step_functions},
    },
};

/// Home page handler - delegates all UI concerns to UI layer
#[instrument(skip_all, fields(handler = "home", operation = "page_render"))]
pub async fn home_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();
    let home_page_ui = HomePageUi::new(user);
    let html = home_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Lambda page handler - delegates all UI concerns to UI layer
#[instrument(skip_all, fields(handler = "lambda", operation = "page_render"))]
pub async fn lambda_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Get lambda data from service
    let lambdas = lambda_service.get_all().await?;

    let lambda_page_ui = LambdaPageUi::new(user, lambdas);
    let html = lambda_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Create lambda page handler - renders lambda creation form
#[instrument(skip_all, fields(handler = "create_lambda", operation = "page_render"))]
pub async fn create_lambda_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    let create_lambda_page_ui = CreateLambdaPageUi::new(user);
    let html = create_lambda_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Delete lambda handler - deletes lambda and returns lambda index page
#[instrument(skip_all, fields(handler = "delete_lambda", operation = "page_delete"))]
pub async fn delete_lambda_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Path(lambda_name): Path<String>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Delete the lambda function
    if let Err(e) = lambda_service.delete(&lambda_name).await {
        tracing::error!(
            lambda_name = %lambda_name,
            error = %e,
            "Failed to delete lambda function"
        );
        // Continue to render the page even if deletion failed
        // The user will see the lambda still exists in the list
    } else {
        tracing::info!(
            lambda_name = %lambda_name,
            "Lambda function deleted successfully"
        );
    }

    // Get updated lambda list (after potential deletion)
    let lambdas = lambda_service.get_all().await?;

    // Render lambda index page with updated list
    let lambda_page_ui = LambdaPageUi::new(user, lambdas);
    let html = lambda_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Edit lambda page handler - renders lambda edit form with current source code
#[instrument(skip_all, fields(handler = "edit_lambda", operation = "page_render"))]
pub async fn edit_lambda_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
    lambda_service: LambdasService<LambdaStorage>,
    Path(lambda_name): Path<String>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    tracing::info!(
        lambda_name = %lambda_name,
        "Loading lambda for editing"
    );

    // Load lambda data from business logic
    let lambda = lambda_service.get_by_name(&lambda_name).await?;

    match lambda {
        Some(lambda_data) => {
            tracing::debug!(
                lambda_name = %lambda_name,
                source_length = lambda_data.source_code.len(),
                has_wasm = lambda_data.wasm_bytes.is_some(),
                status = ?lambda_data.status,
                "Lambda data loaded successfully for editing"
            );

            let edit_lambda_page_ui =
                EditLambdaPageUi::new(user, lambda_data.name, lambda_data.source_code);
            let html = edit_lambda_page_ui.render_html(&state.tera)?;

            Ok(html)
        }
        None => {
            tracing::warn!(
                lambda_name = %lambda_name,
                "Lambda function not found for editing"
            );

            Err(crate::error_handling::types::AppError::not_found(&format!(
                "Lambda function '{}' not found",
                lambda_name
            )))
        }
    }
}

/// Step Functions page handler - delegates all UI concerns to UI layer
#[instrument(
    skip_all,
    fields(handler = "step_functions", operation = "page_render")
)]
pub async fn step_functions_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // Get mock step functions for UI demonstration
    let step_functions = get_mock_step_functions();

    let step_functions_page_ui = StepFunctionsPageUi::new(user, step_functions);
    let html = step_functions_page_ui.render_html(&state.tera)?;

    Ok(html)
}

/// Create step function page handler - renders step function creation form
#[instrument(
    skip_all,
    fields(handler = "create_step_function", operation = "page_render")
)]
pub async fn create_step_function_handler(
    State(state): State<AppState>,
    auth_session: AuthSession<AuthBackend>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    let create_step_function_page_ui = CreateStepFunctionPageUi::new(user);
    let html = create_step_function_page_ui.render_html(&state.tera)?;

    Ok(html)
}
