//! Resources infrastructure implementation
//!
//! This module provides the concrete implementation of resource repository
//! for managing RUSTWS resources across different service types.

// Sub-modules
mod storage;

// Re-export public types and implementations
pub use storage::ResourceStorage;
