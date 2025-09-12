//! Lambda domain UI logic

use crate::{
    auth::dto::User,
    business_logic::lambdas::LambdaSummary,
    error_handling::types::AppResult,
    ui::{TeraEngine, TeraRenderer, Ui, shared::layouts::BaseLayoutProps},
};
use axum::response::Html;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LambdaPageUi {
    layout: BaseLayoutProps,
    lambdas: Vec<LambdaSummary>,
}

impl LambdaPageUi {
    pub fn new(user: User, lambdas: Vec<LambdaSummary>) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("Lambda Functions - RUSTWS Core")
                .description("Serverless compute functions with automatic scaling and event-driven execution")
                .keywords("lambda, serverless, functions, compute, scaling")
                .user(Some(user)),
            lambdas,
        }
    }
}

impl Ui for LambdaPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("lambdas", &self.lambdas);
        tera.render_template("lambda/index.html", &context)
    }
}
