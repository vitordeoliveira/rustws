//! PostgreSQL connection pool setup

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tracing::{info, instrument};

use crate::error_handling::types::AppResult;

/// Create a PostgreSQL connection pool with optimized settings
#[instrument(skip_all, fields(
    operation = "create_postgres_pool",
    database_url = %mask_database_url(database_url)
))]
pub async fn create_postgres_pool(database_url: &str) -> AppResult<PgPool> {
    info!("Creating PostgreSQL connection pool");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;

    info!(
        max_connections = 10,
        min_connections = 2,
        "PostgreSQL connection pool created successfully"
    );

    Ok(pool)
}

/// Mask sensitive parts of database URL for logging
fn mask_database_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            if let Some(scheme_end) = url.find("://") {
                let scheme = &url[..scheme_end + 3];
                let user = &url[scheme_end + 3..colon_pos];
                let after_at = &url[at_pos..];
                return format!("{}{}:***{}", scheme, user, after_at);
            }
        }
    }
    "***".to_string()
}
