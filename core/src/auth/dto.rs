use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use axum_login::{AuthUser, AuthnBackend};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error_handling::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: uuid::Uuid,
    pub role: String,
    pub remember_me: bool,
    pub password_hash: String,
    pub is_owner: bool,
}

impl AuthUser for User {
    type Id = uuid::Uuid;

    fn id(&self) -> Self::Id {
        self.user_id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[derive(Clone)]
pub struct AuthBackend {
    pub db: sqlx::PgPool,
}

impl AuthBackend {}

pub struct Credentials {
    pub email: String,
    pub password: String,
}

impl AuthnBackend for AuthBackend {
    type User = User;

    type Credentials = Credentials;

    type Error = AppError;

    #[instrument(skip_all, fields(
        service = "auth_backend",
        operation = "authenticate",
        email = %creds.email
    ))]
    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        tracing::info!(email = %creds.email, "Authenticating user");

        let user_record = sqlx::query!(
            r#"
            SELECT id, email, hashed_password, role, is_owner
            FROM users.users 
            WHERE email = $1
            "#,
            creds.email
        )
        .fetch_optional(&self.db)
        .await?;

        let Some(user_record) = user_record else {
            tracing::warn!(email = %creds.email, "User not found during authentication");
            return Ok(None);
        };

        let parsed_hash = PasswordHash::new(&user_record.hashed_password)
            .map_err(|e| AppError::internal(&format!("Invalid password hash format: {}", e)))?;

        let argon2 = Argon2::default();
        match argon2.verify_password(creds.password.as_bytes(), &parsed_hash) {
            Ok(()) => {
                tracing::info!(
                    user_id = %user_record.id,
                    email = %creds.email,
                    "User authenticated successfully"
                );

                Ok(Some(User {
                    user_id: user_record.id,
                    role: user_record.role.unwrap_or_else(|| "user".to_string()),
                    remember_me: false,
                    password_hash: user_record.hashed_password,
                    is_owner: user_record.is_owner.unwrap_or(false),
                }))
            }
            Err(_) => {
                tracing::warn!(email = %creds.email, "Invalid password during authentication");
                Ok(None)
            }
        }
    }

    #[instrument(skip_all, fields(
        service = "auth_backend",
        operation = "get_user",
        user_id = %user_id
    ))]
    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        tracing::debug!(user_id = %user_id, "Fetching user by ID");

        let user_record = sqlx::query!(
            r#"
            SELECT id, email, role, is_owner, hashed_password
            FROM users.users 
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?;

        match user_record {
            Some(user_record) => {
                tracing::debug!(
                    user_id = %user_record.id,
                    email = %user_record.email,
                    "User found by ID"
                );

                Ok(Some(User {
                    user_id: user_record.id,
                    role: user_record.role.unwrap_or_else(|| "user".to_string()),
                    remember_me: false,
                    password_hash: user_record.hashed_password,
                    is_owner: user_record.is_owner.unwrap_or(false),
                }))
            }
            None => {
                tracing::warn!(user_id = %user_id, "User not found by ID");
                Ok(None)
            }
        }
    }
}
