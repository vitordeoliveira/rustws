use axum_login::{AuthManagerLayer, AuthManagerLayerBuilder};
use sqlx::PgPool;
use tokio::task::JoinHandle;
use tower_sessions::{
    cookie::{time::Duration, Key},
    session_store, ExpiredDeletion, Expiry, SessionManagerLayer,
};
use tower_sessions_sqlx_store::PostgresStore;
use tracing::instrument;

use crate::{auth::dto::AuthBackend, error_handling::AppResult};

pub mod dto;

pub type SessionManager =
    AuthManagerLayer<AuthBackend, PostgresStore, tower_sessions::service::PrivateCookie>;

#[instrument(
    skip_all,
    fields(service = "auth_service", operation = "create_session_managers")
)]
pub async fn create_session_manager(
    pg_pool: PgPool,
) -> AppResult<(SessionManager, JoinHandle<Result<(), session_store::Error>>)> {
    tracing::info!("Creating PostgreSQL session store");
    let session_store = PostgresStore::new(pg_pool.clone());

    tracing::info!("Running session store migrations");
    session_store.migrate().await?;

    tracing::info!("Starting session cleanup task");
    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(1)),
    );

    tracing::info!(
        "Creating session manager with private cookies and 60 minute inactivity timeout"
    );
    let key = Key::from(b"super_secret_key_for_session_encryption_must_be_64_bytes_long_123");
    // let key = Key::generate();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_private(key)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(10)));

    let backend = AuthBackend {
        db: pg_pool.clone(),
    };

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer.clone()).build();

    tracing::info!("Session manager created successfully");
    Ok((auth_layer, deletion_task))
}
