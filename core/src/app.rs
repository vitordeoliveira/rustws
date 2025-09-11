use std::net::SocketAddr;

use tokio::{signal, task::AbortHandle};
use tracing_subscriber::EnvFilter;

#[cfg(debug_assertions)]
use crate::development::livereload::enable_livereload;
use crate::{
    auth, configuration,
    error_handling::{self, AppResult},
    routing,
    state::{self, AppState},
};

pub struct App {
    config: configuration::AppConfig,
    app_state: AppState,
}

impl App {
    pub async fn new() -> Self {
        // Load configuration FIRST (panics if required values missing)
        let config = configuration::load_config();
        let app_state = state::build_app_state(config.clone()).await.unwrap();
        Self { config, app_state }
    }

    pub async fn serve(&self) -> AppResult<()> {
        // Initialize tracing with the configured log level
        let log_level = &self.config.logging.level;
        tracing_subscriber::fmt()
            .pretty()
            .with_env_filter(EnvFilter::new(log_level))
            .init();
        tracing::info!(level = %log_level, "Tracing initialized with configured log level");

        let (session_manager_layer, deletion_task) =
            auth::create_session_manager(self.app_state.pg_pool.clone()).await?;
        let router = routing::create_router(self.app_state.clone(), session_manager_layer).await?;

        #[cfg(debug_assertions)]
        let (_watcher, router) = enable_livereload(router, self.app_state.tera.clone())?;

        // Run the server
        let addr: SocketAddr = format!("{}:{}", self.config.server.host, self.config.server.port)
            .parse()
            .map_err(|e| {
                error_handling::types::AppError::internal(&format!(
                    "Invalid host:port combination {}:{} - {}",
                    self.config.server.host, self.config.server.port, e
                ))
            })?;

        tracing::info!(
            "🏔️ {} v{} server starting on http://{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            addr
        );

        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            error_handling::types::AppError::internal(&format!(
                "Failed to bind to address {}: {}",
                addr, e
            ))
        })?;
        axum::serve(listener, router)
            .with_graceful_shutdown(App::shutdown_signal(deletion_task.abort_handle()))
            .await
            .map_err(|e| {
                error_handling::types::AppError::internal(&format!("Server error: {}", e))
            })?;

        // Handle cleanup task completion/cancellation gracefully
        match deletion_task.await {
            Ok(Ok(())) => {
                tracing::info!("Cleanup task completed successfully");
            }
            Ok(Err(e)) => {
                tracing::warn!("Cleanup task failed: {}", e);
            }
            Err(e) if e.is_cancelled() => {
                tracing::info!("Cleanup task was cancelled during shutdown - this is expected");
            }
            Err(e) => {
                tracing::error!("Unexpected cleanup task error: {}", e);
                return Err(error_handling::types::AppError::internal(&format!(
                    "cleanup task error: {}",
                    e
                )));
            }
        }

        Ok(())
    }

    async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            use tokio::signal;

            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => { deletion_task_abort_handle.abort() },
            _ = terminate => { deletion_task_abort_handle.abort() },
        }
    }
}
