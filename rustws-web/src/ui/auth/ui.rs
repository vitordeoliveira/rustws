//! Auth domain UI logic

use crate::{
    error_handling::types::AppResult,
    ui::{shared::layouts::BaseLayoutProps, Ui, TeraEngine, TeraRenderer},
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
            layout: BaseLayoutProps::new().title("Login - Matrix"),
        }
    }
}

impl Ui for LoginPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let context = self.layout.to_context()?;
        tera.render_template("auth/login.html", &context)
    }
}

#[derive(Debug, Serialize)]
pub struct SignUpPageUi {
    layout: BaseLayoutProps,
}

impl SignUpPageUi {
    pub fn new() -> Self {
        Self {
            layout: BaseLayoutProps::new().title("Sign Up - Matrix"),
        }
    }
}

impl Ui for SignUpPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let context = self.layout.to_context()?;
        tera.render_template("auth/signup.html", &context)
    }
}
