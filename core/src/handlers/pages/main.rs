//! Main page handlers

use axum::{extract::State, response::Html};
use axum_login::AuthSession;
use tracing::instrument;

use crate::{
    auth::dto::AuthBackend,
    error_handling::types::AppResult,
    state::AppState,
    ui::{Ui, home::HomePageUi, lambda::LambdaPageUi},
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
) -> AppResult<Html<String>> {
    let user = auth_session.user.unwrap();
    let lambda_page_ui = LambdaPageUi::new(user);
    let html = lambda_page_ui.render_html(&state.tera)?;

    Ok(html)
}
