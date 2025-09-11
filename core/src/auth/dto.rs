use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum_login::{AuthUser, AuthnBackend};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error_handling::{AppError, types::AppResult};

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

impl AuthBackend {
    #[instrument(skip_all, fields(service = "auth_backend", operation = "create_user"))]
    pub async fn create_user(
        &self,
        client_id: uuid::Uuid,
        email: &str,
        password: &str,
        role: Option<&str>,
        is_owner: Option<bool>,
    ) -> AppResult<User> {
        tracing::info!(email = %email, client_id = %client_id, "Creating new user");

        let user_id = uuid::Uuid::new_v4();
        let user_role = role.unwrap_or("user");
        let is_owner_flag = is_owner.unwrap_or(false);

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::internal(&format!("Password hashing failed: {}", e)))?
            .to_string();

        tracing::debug!("Password hashed successfully");

        sqlx::query!(
            r#"
            INSERT INTO users.users (id, email, hashed_password, role, is_owner)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            user_id,
            email,
            password_hash,
            user_role,
            is_owner_flag
        )
        .execute(&self.db)
        .await?;

        tracing::info!(user_id = %user_id, email = %email, "User created successfully");

        Ok(User {
            user_id,
            role: user_role.to_string(),
            remember_me: false,
            password_hash: password_hash.clone(),
            is_owner: is_owner_flag,
        })
    }
}

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
