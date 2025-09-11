//! Main page handlers

use axum::{extract::State, response::Html};
use tracing::instrument;

use crate::{
    error_handling::types::AppResult,
    state::AppState,
    ui::{Ui, home::HomePageUi},
};

/// Home page handler - delegates all UI concerns to UI layer
#[instrument(skip_all, fields(handler = "home", operation = "page_render"))]
pub async fn home_handler(State(state): State<AppState>) -> AppResult<Html<String>> {
    let home_page_ui = HomePageUi::new();
    let html = home_page_ui.render_html(&state.tera)?;

    Ok(html)
}
