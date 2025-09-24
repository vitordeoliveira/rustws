//! Live reload functionality for development mode
//! Watches template and static files for changes and triggers browser refresh

use crate::error_handling::types::AppResult;
use crate::ui::{TeraEngine, TeraRenderer};
use axum::Router;
use notify::{Event, RecommendedWatcher, Watcher};
use std::path::Path;
use std::process::Command;
use tower_livereload::LiveReloadLayer;

/// Compile Tailwind CSS using npx
fn compile_tailwind() -> Result<(), String> {
    let project_path =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let input_path = project_path.join("assets/src/main.css");
    let output_path = project_path.join("assets/public/output.css");

    // Check if input CSS file exists
    if !input_path.exists() {
        return Err(format!(
            "Input CSS file not found: {}",
            input_path.display()
        ));
    }

    // Ensure output directory exists
    if let Some(output_dir) = output_path.parent() {
        std::fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    tracing::debug!(
        "🎨 Compiling Tailwind CSS: {} -> {}",
        input_path.display(),
        output_path.display()
    );

    let output = Command::new("npx")
        .args([
            "@tailwindcss/cli",
            "-i",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
            "--minify", // Minify CSS in production
        ])
        .output()
        .map_err(|e| format!("Failed to execute npx command (is npx installed?): {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Tailwind CSS compilation failed:\nSTDERR: {}\nSTDOUT: {}",
            stderr, stdout
        ));
    }

    tracing::debug!("✅ Tailwind CSS compiled successfully");
    Ok(())
}

/// Enable live reload for development
/// Watches templates and static files for changes
pub fn enable_livereload(app: Router, tera: TeraEngine) -> AppResult<(RecommendedWatcher, Router)> {
    let livereload = LiveReloadLayer::new();
    let reloader = livereload.reloader();

    let watcher = {
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    // Filter out node_modules files to avoid unnecessary reloads
                    let relevant_files: Vec<_> = event
                        .paths
                        .iter()
                        .filter(|path| !path.to_string_lossy().contains("node_modules"))
                        .map(|p| p.display().to_string())
                        .collect();

                    // Only proceed if there are relevant file changes
                    if !relevant_files.is_empty() {
                        tracing::warn!("🔄 File change detected: {:?}", relevant_files);

                        // Compile Tailwind CSS on file changes
                        if let Err(e) = compile_tailwind() {
                            tracing::error!("❌ Tailwind CSS compilation failed: {}", e);
                        }

                        // Reload templates in debug mode
                        #[cfg(debug_assertions)]
                        if let Err(e) = tera.reload_templates() {
                            tracing::error!("❌ Failed to reload templates: {}", e);
                        }

                        // Trigger browser reload
                        tracing::debug!("🌐 Triggering browser reload");
                        reloader.reload()
                    } else {
                        tracing::debug!("🔄 Ignoring node_modules file changes");
                    }
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

    // Initial Tailwind CSS compilation on startup
    tracing::warn!("🚀 Running initial Tailwind CSS compilation...");
    match compile_tailwind() {
        Ok(_) => {
            tracing::warn!("✅ Initial Tailwind CSS compilation completed successfully");
        }
        Err(e) => {
            tracing::warn!("⚠️ Initial Tailwind CSS compilation failed: {}", e);
            tracing::warn!("💡 To fix this, ensure:");
            tracing::warn!("   1. Node.js and npm are installed");
            tracing::warn!("   2. Run: npm install -D @tailwindcss/cli");
            tracing::warn!("   3. Verify assets/src/main.css exists");
        }
    }

    let app_with_reloader = app.layer(livereload);

    Ok((watcher, app_with_reloader))
}
