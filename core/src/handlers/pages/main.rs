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
    Path(lambda_name): Path<String>,
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();

    // TODO: Replace with actual source code loading from business logic
    // For now, using a placeholder source code for UI demonstration
    let placeholder_source = format!(
        r#"//! Lambda Function: {}
//! 
//! This is an existing lambda function

/// Main handler function for the lambda
/// This function will be called by the wasmer runtime
#[no_mangle]
pub extern "C" fn handler() -> i32 {{
    // Your existing lambda logic here
    // TODO: Load actual source code from file
    42
}}

/// Example function - replace with your actual functions
#[no_mangle]
pub extern "C" fn example_function(input: i32) -> i32 {{
    input * 2
}}

// Your existing custom functions here
// (This is placeholder content until business logic is implemented)"#,
        lambda_name
    );

    let edit_lambda_page_ui = EditLambdaPageUi::new(user, lambda_name, placeholder_source);
    let html = edit_lambda_page_ui.render_html(&state.tera)?;

    Ok(html)
}
