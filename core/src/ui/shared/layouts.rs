//! Shared layout components

use serde::Serialize;
use tera::Context;

use crate::{auth::dto::User, error_handling::types::AppResult};

#[derive(Debug, Serialize)]
pub struct BaseLayoutProps {
    pub title: String,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub user: Option<User>,
}

impl Default for BaseLayoutProps {
    fn default() -> Self {
        Self {
            title: "Matrix".to_string(),
            description: None,
            keywords: None,
            user: None,
        }
    }
}

impl BaseLayoutProps {
    /// Create a new BaseLayoutProps with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page title (builder pattern)
    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = title.into();
        self
    }

    /// Set the page description (builder pattern)
    pub fn description<T: Into<String>>(mut self, description: T) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the page keywords (builder pattern)
    pub fn keywords<T: Into<String>>(mut self, keywords: T) -> Self {
        self.keywords = Some(keywords.into());
        self
    }

    /// Set the user (builder pattern)
    pub fn user(mut self, user: Option<User>) -> Self {
        self.user = user;
        self
    }

    /// Create a new Tera context with layout properties inserted
    pub fn to_context(self) -> AppResult<Context> {
        Context::from_serialize(self).map_err(|e| {
            crate::error_handling::types::AppError::configuration_from_error(
                e,
                "context serialization for layout properties",
            )
        })
    }
}
