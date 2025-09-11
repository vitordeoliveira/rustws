//! Tera Template Engine Abstraction
//!
//! Provides a conditional compilation wrapper around Tera:
//! - Debug mode: Arc<Mutex<Tera>> for live reload functionality
//! - Release mode: Direct Tera for zero-cost template rendering

use crate::error_handling::types::{AppError, AppResult};
use axum::response::Html;
use std::sync::{Arc, Mutex};
use tera::{Context, Tera};
use tracing::instrument;

#[cfg(debug_assertions)]
pub type TeraEngine = Arc<Mutex<Tera>>;

#[cfg(not(debug_assertions))]
pub type TeraEngine = Tera;

/// Unified interface for template rendering across debug/release modes
pub trait TeraRenderer {
    fn render_template(&self, template_name: &str, context: &Context) -> AppResult<Html<String>>;

    #[cfg(debug_assertions)]
    fn reload_templates(&self) -> AppResult<()>;
}

#[cfg(debug_assertions)]
impl TeraRenderer for TeraEngine {
    #[instrument(skip_all, fields(
        operation = "render_template_debug",
        template = %template_name
    ))]
    fn render_template(&self, template_name: &str, context: &Context) -> AppResult<Html<String>> {
        let tera_instance = self
            .lock()
            .map_err(|e| AppError::internal(&format!("Failed to acquire Tera lock: {}", e)))?;

        let html_string = tera_instance
            .render(template_name, context)
            .map_err(|e| AppError::configuration_from_error(e, "template rendering"))?;

        Ok(Html(html_string))
    }

    #[instrument(skip_all, fields(operation = "reload_templates"))]
    fn reload_templates(&self) -> AppResult<()> {
        let mut tera_instance = self.lock().map_err(|e| {
            AppError::internal(&format!("Failed to acquire Tera lock for reload: {}", e))
        })?;

        tera_instance
            .full_reload()
            .map_err(|e| AppError::configuration_from_error(e, "template reload"))?;

        tracing::warn!("Templates reloaded successfully");
        Ok(())
    }
}

#[cfg(not(debug_assertions))]
impl TeraRenderer for TeraEngine {
    #[instrument(skip_all, fields(
        operation = "render_template_release",
        template = %template_name
    ))]
    fn render_template(&self, template_name: &str, context: &Context) -> AppResult<Html<String>> {
        let html_string = self
            .render(template_name, context)
            .map_err(|e| AppError::configuration_from_error(e, "template rendering"))?;

        Ok(Html(html_string))
    }
}

/// Helper function to create TeraEngine from Tera instance
#[instrument(skip_all, fields(operation = "create_tera_engine"))]
pub fn create_tera_engine(tera: Tera) -> TeraEngine {
    #[cfg(debug_assertions)]
    {
        tracing::warn!("Creating TeraEngine in debug mode with Arc<Mutex<Tera>>");
        Arc::new(Mutex::new(tera))
    }

    #[cfg(not(debug_assertions))]
    {
        tracing::warn!("Creating TeraEngine in release mode with direct Tera");
        tera
    }
}
