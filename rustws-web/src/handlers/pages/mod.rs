//! Page handlers module - organized by functionality

mod auth;
mod main;

// Re-export all handlers for easy access
pub use auth::*;
pub use main::*;
