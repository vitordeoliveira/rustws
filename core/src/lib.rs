//! Matrix application library

// Public API - external crates can access these
pub mod app;
pub mod error_handling; // Error types used in public interfaces

// Internal modules - only accessible within this crate
pub(crate) mod auth;
pub(crate) mod business_logic; // Domain-based business logic
pub(crate) mod configuration; // App configuration setup
pub(crate) mod development; // Development tools (needed by main.rs for live reload)
pub(crate) mod handlers; // HTTP request handlers (only used by routing)
pub(crate) mod infrastructure; // Database/external services
pub(crate) mod routing; // Route definitions (needed to create app router)
pub(crate) mod state; // Application state (used by main.rs for setup)
pub(crate) mod ui; // UI templates and rendering (only used by handlers)
