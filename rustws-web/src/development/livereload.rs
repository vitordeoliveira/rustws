//! Live reload functionality for development mode
//! Watches template and static files for changes and triggers browser refresh

use crate::error_handling::types::AppResult;
use crate::ui::{TeraEngine, TeraRenderer};
use axum::Router;
use notify::{Event, RecommendedWatcher, Watcher};
use std::path::Path;
use tower_livereload::LiveReloadLayer;

/// Enable live reload for development
/// Watches templates and static files for changes
pub fn enable_livereload(app: Router, tera: TeraEngine) -> AppResult<(RecommendedWatcher, Router)> {
    let livereload = LiveReloadLayer::new();
    let reloader = livereload.reloader();

    let watcher = {
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    tracing::warn!("🔄 File change detected, triggering reload");
                    #[cfg(debug_assertions)]
                    if let Err(e) = tera.reload_templates() {
                        tracing::error!("Failed to reload templates: {}", e);
                    }
                    reloader.reload()
                }
            }
        })
        .map_err(|e| {
            crate::error_handling::types::AppError::configuration_from_error(
                e,
                "Failed to create file watcher for livereload",
            )
        })?;

        // Watch assets directory
        if Path::new("assets").exists() {
            watcher
                .watch(Path::new("assets"), notify::RecursiveMode::Recursive)
                .map_err(|e| {
                    crate::error_handling::types::AppError::configuration_with_path(
                        &format!("Failed to watch assets directory: {}", e),
                        Path::new("assets"),
                        "Setting up livereload assets watcher",
                    )
                })?;
            tracing::warn!("📁 Watching assets/ for changes");
        }

        let ui_path = Path::new("src/ui");
        if ui_path.exists() {
            watcher
                .watch(ui_path, notify::RecursiveMode::Recursive)
                .map_err(|e| {
                    crate::error_handling::types::AppError::configuration_with_path(
                        &format!("Failed to watch UI directory: {}", e),
                        ui_path,
                        "Setting up livereload UI watcher",
                    )
                })?;
            tracing::warn!("📁 Watching src/ui/ for changes");
        } else {
            tracing::warn!("📁 UI directory src/ui/ not found, skipping UI file watching");
        }
        watcher
    };

    let app_with_reloader = app.layer(livereload);

    Ok((watcher, app_with_reloader))
}
