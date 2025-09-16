//! Lambda execution metrics tracking
//!
//! This module provides metrics collection and persistence for lambda executions
//! using a ledger pattern where individual executions are recorded and totals
//! are calculated on-demand.

/// Individual lambda execution record (ledger entry)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LambdaExecutionEntry {
    /// Name of the executed lambda
    pub lambda_name: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether the execution succeeded or failed
    pub status: ExecutionStatus,
    /// When the execution occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Execution status for ledger entries
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failed,
}

/// Ledger of all lambda executions (event sourcing pattern)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LambdaMetricsLedger {
    /// All execution records
    pub entries: Vec<LambdaExecutionEntry>,
}

/// Aggregated metrics calculated from ledger entries
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

impl LambdaExecutionEntry {
    /// Create a new execution entry
    pub fn new(lambda_name: String, execution_time_ms: u64, status: ExecutionStatus) -> Self {
        Self {
            lambda_name,
            execution_time_ms,
            status,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check if this execution was successful
    pub fn is_success(&self) -> bool {
        matches!(self.status, ExecutionStatus::Success)
    }
}

impl LambdaMetricsLedger {
    /// Create a new empty ledger
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a new execution entry to the ledger
    pub fn add_execution(&mut self, entry: LambdaExecutionEntry) {
        self.entries.push(entry);
    }

    /// Record a successful execution
    pub fn record_success(&mut self, lambda_name: String, execution_time_ms: u64) {
        self.add_execution(LambdaExecutionEntry::new(
            lambda_name,
            execution_time_ms,
            ExecutionStatus::Success,
        ));
    }

    /// Record a failed execution
    pub fn record_failure(&mut self, lambda_name: String, execution_time_ms: u64) {
        self.add_execution(LambdaExecutionEntry::new(
            lambda_name,
            execution_time_ms,
            ExecutionStatus::Failed,
        ));
    }

    /// Calculate aggregated metrics from all entries
    pub fn calculate_totals(&self) -> LambdasMetrics {
        if self.entries.is_empty() {
            return LambdasMetrics::default();
        }

        let total_executions = self.entries.len() as u64;
        let total_execution_time_ms = self.entries.iter().map(|e| e.execution_time_ms).sum();

        let successful_executions = self.entries.iter().filter(|e| e.is_success()).count() as u64;

        let failed_executions = total_executions - successful_executions;

        // Find most recent execution
        let most_recent = self.entries.iter().max_by_key(|e| e.timestamp);

        let (last_executed_lambda, last_execution_time) = most_recent
            .map(|e| (Some(e.lambda_name.clone()), Some(e.timestamp)))
            .unwrap_or((None, None));

        LambdasMetrics {
            total_executions,
            total_execution_time_ms,
            successful_executions,
            failed_executions,
            last_executed_lambda,
            last_execution_time,
        }
    }
}

impl Default for LambdaMetricsLedger {
    fn default() -> Self {
        Self::new()
    }
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
