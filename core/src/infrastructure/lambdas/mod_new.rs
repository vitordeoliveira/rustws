//! Lambda infrastructure implementation
//!
//! This module provides the concrete implementation of lambda repository
//! organized into focused sub-modules for better maintainability.

// Sub-modules
mod metrics;
mod storage;
mod wasm_runtime;
mod repository;

// Re-export public types and implementations
pub use metrics::LambdasMetrics;
pub use storage::LambdaStorage;
