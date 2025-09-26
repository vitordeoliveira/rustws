//! Tera Template Engine Utilities

use crate::configuration::UIConfig;
use axum::response::Html;
use tera::Tera;
use tracing::instrument;

pub mod api_gateway;
pub mod auth;
pub mod engine;
pub mod filters;
pub mod home;
pub mod lambda;
pub mod monitoring;
pub mod reports;
pub mod shared;
pub mod step_functions;
pub mod testers;

// Re-export engine types for convenient access
pub use engine::{TeraEngine, TeraRenderer};

/// Initialize UI module with Tera template engine
#[instrument(skip_all, fields(operation = "init_ui"))]
pub fn init_ui(ui_config: UIConfig) -> crate::error_handling::types::AppResult<Tera> {
    let mut tera = Tera::new(&ui_config.template_path).map_err(|e| {
        crate::error_handling::types::AppError::configuration_from_error(
            e,
            "UI template initialization",
        )
    })?;

    // Register template helpers from templates module
    tera.register_tester("is_some", testers::is_some);
    tera.register_filter("truncate", filters::truncate);
    tera.register_filter("get_module_status", filters::get_module_status);
    tera.register_filter("svg_safe", filters::svg_safe);

    println!("🎨 UI Module initialized with Tera template engine");
    Ok(tera)
}

pub trait Ui {
    fn render_html(
        self,
        tera: &TeraEngine,
    ) -> crate::error_handling::types::AppResult<Html<String>>;
}
