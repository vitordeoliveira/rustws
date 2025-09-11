use sqlx::PgPool;
use tracing::instrument;

use crate::infrastructure::storage::database::create_postgres_pool;
use crate::ui::{engine::create_tera_engine, TeraEngine};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub config: crate::configuration::AppConfig,
    pub pg_pool: PgPool,
    pub http_client: reqwest::Client,
    pub tera: TeraEngine,
    // Add other shared state here as needed:
    // pub cache: Arc<dyn Cache>,
}

/// Build the application state
#[instrument(skip_all, fields(operation = "build_app_state"))]
pub async fn build_app_state(
    config: crate::configuration::AppConfig,
) -> crate::error_handling::types::AppResult<AppState> {
    // Initialize database connection pool
    let pg_pool = create_postgres_pool(&config.database.url).await?;

    // Initialize HTTP client
    let http_client = reqwest::Client::new();

    // Initialize UI template engine
    let tera = crate::ui::init_ui(config.ui.clone())?;
    let tera_engine = create_tera_engine(tera);

    Ok(AppState {
        config,
        pg_pool,
        http_client,
        tera: tera_engine,
    })
}
