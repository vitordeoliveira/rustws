//! Authentication page handlers

use axum::{
    extract::State,
    response::{Html, Redirect},
    Form,
};
use axum_login::AuthSession;
use serde::Deserialize;
use tower_sessions::{cookie::time::Duration, Expiry};
use tracing::instrument;

use crate::{
    auth::dto::{AuthBackend, Credentials},
    error_handling::types::AppResult,
    state::AppState,
    ui::{
        auth::LoginPageUi,
        Ui,
    },
};

#[derive(Deserialize)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
    pub remember_me: Option<String>,
}

/// Login page handler - delegates all UI concerns to UI layer
#[instrument(skip_all, fields(handler = "login", operation = "page_render"))]
pub async fn login_handler(State(state): State<AppState>) -> AppResult<Html<String>> {
    let login_ui = LoginPageUi::new();
    login_ui.render_html(&state.tera)
}

/// Login form submission handler - processes login form data
#[instrument(
    skip_all,
    fields(handler = "login_form", operation = "form_submission")
)]
pub async fn login_form_handler(
    mut auth_session: AuthSession<AuthBackend>,
    Form(form_data): Form<LoginFormData>,
) -> AppResult<Redirect> {
    tracing::info!(
        email = %form_data.email,
        "Processing login form submission"
    );

    let credentials = Credentials {
        email: form_data.email.clone(),
        password: form_data.password,
    };

    match auth_session.authenticate(credentials).await {
        Ok(Some(user)) => {
            tracing::info!(
                user_id = %user.user_id,
                email = %form_data.email,
                "User authenticated successfully"
            );

            if let Err(e) = auth_session.login(&user).await {
                tracing::error!(
                    user_id = %user.user_id,
                    error = %e,
                    "Failed to create session after authentication"
                );
                return Ok(Redirect::to("/login"));
            }

            if form_data.remember_me.is_some() {
                auth_session
                    .session
                    .set_expiry(Some(Expiry::OnInactivity(Duration::hours(48))));
                tracing::info!(
                    user_id = %user.user_id,
                    "Remember me enabled - session extended to 48 hours"
                );
            }

            tracing::info!(
                user_id = %user.user_id,
                "Session created successfully - redirecting to home"
            );

            Ok(Redirect::to("/"))
        }
        Ok(None) => {
            tracing::warn!(
                email = %form_data.email,
                "Authentication failed - invalid credentials"
            );
            Ok(Redirect::to("/login"))
        }
        Err(e) => {
            tracing::error!(
                email = %form_data.email,
                error = %e,
                "Authentication error"
            );
            Ok(Redirect::to("/login"))
        }
    }
}

/// Logout handler - processes user logout
#[instrument(skip_all, fields(handler = "logout", operation = "user_logout"))]
pub async fn logout_handler(mut auth_session: AuthSession<AuthBackend>) -> AppResult<Redirect> {
    if let Some(user) = &auth_session.user {
        tracing::info!(
            user_id = %user.user_id,
            "User logging out"
        );
    }

    match auth_session.logout().await {
        Ok(user) => {
            if let Some(user) = user {
                tracing::info!(
                    user_id = %user.user_id,
                    "User logged out successfully"
                );
            }
            Ok(Redirect::to("/login"))
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                "Failed to logout user"
            );
            Ok(Redirect::to("/login"))
        }
    }
}
