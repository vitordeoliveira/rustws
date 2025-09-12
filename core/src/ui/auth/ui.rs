//! Auth domain UI logic

use crate::{
    error_handling::types::AppResult,
    ui::{TeraEngine, TeraRenderer, Ui, shared::layouts::BaseLayoutProps},
};
use axum::response::Html;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LoginPageUi {
    layout: BaseLayoutProps,
}

impl LoginPageUi {
    pub fn new() -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("Sign In - RUSTWS Core")
                .description("Access your RUSTWS infrastructure dashboard")
                .keywords("rustws, login, infrastructure, cloud, serverless"),
        }
    }
}

impl Ui for LoginPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let context = self.layout.to_context()?;
        tera.render_template("auth/login.html", &context)
    }
}
