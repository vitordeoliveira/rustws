//! Home domain UI logic

use crate::{
    auth::dto::User,
    error_handling::types::AppResult,
    ui::{TeraEngine, TeraRenderer, Ui, shared::layouts::BaseLayoutProps},
};
use axum::response::Html;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HomePageUi {
    layout: BaseLayoutProps,
}

impl HomePageUi {
    pub fn new(user: User) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("RUSTWS Core - Infrastructure Dashboard")
                .description("RUSTWS core infrastructure services - API Gateway, Lambda Functions, and Step Functions")
                .keywords("rustws, api-gateway, lambda, step-functions, infrastructure, serverless")
                .user(Some(user)),
        }
    }
}

impl Ui for HomePageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let context = self.layout.to_context()?;
        tera.render_template("home/index.html", &context)
    }
}
