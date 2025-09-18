//! Lambda infrastructure implementation
//!
//! This module provides the concrete implementation of lambda repository
//! organized into focused sub-modules for better maintainability.

// Sub-modules
mod metrics;
mod repository;
mod storage;
mod wasm_runtime;

// Re-export public types and implementations
pub use metrics::{LambdasMetrics, LambdaExecutionEntry};
pub use storage::LambdaStorage;
