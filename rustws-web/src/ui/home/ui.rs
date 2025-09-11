//! Home domain UI logic

use crate::{
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
    pub fn new() -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("rustws - Open Source Cloud Platform")
                .description("An open-source, AWS-like cloud platform designed to run on your VPS")
                .keywords("rust, cloud, vps, open-source, aws, infrastructure"),
        }
    }
}

impl Ui for HomePageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let context = self.layout.to_context()?;
        tera.render_template("home/index.html", &context)
    }
}
