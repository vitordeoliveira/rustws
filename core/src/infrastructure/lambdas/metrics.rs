//! Lambda execution metrics tracking
//!
//! This module provides metrics collection and persistence for lambda executions.

/// Metrics tracking for lambda executions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LambdasMetrics {
    /// Total number of lambda executions
    pub total_executions: u64,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Most recently executed lambda name
    pub last_executed_lambda: Option<String>,
    /// Timestamp of last execution
    pub last_execution_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for LambdasMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            total_execution_time_ms: 0,
            successful_executions: 0,
            failed_executions: 0,
            last_executed_lambda: None,
            last_execution_time: None,
        }
    }
}
