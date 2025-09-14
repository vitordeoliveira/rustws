use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;

/// Main workflow definition structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Optional comment describing the workflow
    #[serde(rename = "Comment", skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// The name of the state to start the workflow execution
    #[serde(rename = "StartAt")]
    pub start_at: String,

    /// Map of state names to their definitions
    #[serde(rename = "States")]
    pub states: HashMap<String, State>,
}

/// Represents a single state in the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Type")]
pub enum State {
    /// Task state - executes a lambda function or service
    #[serde(rename = "Task")]
    Task(TaskState),

    /// Choice state - conditional branching logic
    #[serde(rename = "Choice")]
    Choice(ChoiceState),

    /// Wait state - pause execution for a specified time
    #[serde(rename = "Wait")]
    Wait(WaitState),

    /// Succeed state - successful termination
    #[serde(rename = "Succeed")]
    Succeed(SucceedState),

    /// Fail state - failed termination
    #[serde(rename = "Fail")]
    Fail(FailState),
}

/// Task state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    /// Resource to execute (e.g., "rustws:lambda:hello_world")
    #[serde(rename = "Resource")]
    pub resource: String,

    /// Next state to transition to
    #[serde(rename = "Next", skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// Whether this state ends the workflow
    #[serde(rename = "End", skip_serializing_if = "Option::is_none")]
    pub end: Option<bool>,

    /// Retry configuration for error handling
    #[serde(rename = "Retry", skip_serializing_if = "Option::is_none")]
    pub retry: Option<Vec<RetryConfig>>,

    /// Catch configuration for error handling
    #[serde(rename = "Catch", skip_serializing_if = "Option::is_none")]
    pub catch: Option<Vec<CatchConfig>>,
}

/// Choice state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceState {
    /// List of choice rules
    #[serde(rename = "Choices")]
    pub choices: Vec<ChoiceRule>,

    /// Default state if no choices match
    #[serde(rename = "Default", skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Individual choice rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceRule {
    /// JSON path to the variable to test
    #[serde(rename = "Variable")]
    pub variable: String,

    /// Next state if condition matches
    #[serde(rename = "Next")]
    pub next: String,

    /// String equality condition
    #[serde(rename = "StringEquals", skip_serializing_if = "Option::is_none")]
    pub string_equals: Option<String>,

    /// Boolean equality condition
    #[serde(rename = "BooleanEquals", skip_serializing_if = "Option::is_none")]
    pub boolean_equals: Option<bool>,

    /// Numeric equality condition
    #[serde(rename = "NumericEquals", skip_serializing_if = "Option::is_none")]
    pub numeric_equals: Option<f64>,

    /// Numeric greater than condition
    #[serde(rename = "NumericGreaterThan", skip_serializing_if = "Option::is_none")]
    pub numeric_greater_than: Option<f64>,

    /// Numeric less than condition
    #[serde(rename = "NumericLessThan", skip_serializing_if = "Option::is_none")]
    pub numeric_less_than: Option<f64>,
}

/// Wait state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitState {
    /// Next state after waiting
    #[serde(rename = "Next", skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// Whether this state ends the workflow
    #[serde(rename = "End", skip_serializing_if = "Option::is_none")]
    pub end: Option<bool>,

    /// Number of seconds to wait
    #[serde(rename = "Seconds", skip_serializing_if = "Option::is_none")]
    pub seconds: Option<u32>,

    /// Timestamp to wait until
    #[serde(rename = "Timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// JSON path to seconds value
    #[serde(rename = "SecondsPath", skip_serializing_if = "Option::is_none")]
    pub seconds_path: Option<String>,

    /// JSON path to timestamp value
    #[serde(rename = "TimestampPath", skip_serializing_if = "Option::is_none")]
    pub timestamp_path: Option<String>,
}

/// Succeed state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SucceedState {
    /// Optional comment for the success
    #[serde(rename = "Comment", skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Fail state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailState {
    /// Error name/type
    #[serde(rename = "Error", skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Error cause/description
    #[serde(rename = "Cause", skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
}

/// Retry configuration for error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Types of errors to retry on
    #[serde(rename = "ErrorEquals")]
    pub error_equals: Vec<String>,

    /// Interval between retries in seconds
    #[serde(rename = "IntervalSeconds", skip_serializing_if = "Option::is_none")]
    pub interval_seconds: Option<u32>,

    /// Maximum number of retry attempts
    #[serde(rename = "MaxAttempts", skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,

    /// Backoff rate multiplier
    #[serde(rename = "BackoffRate", skip_serializing_if = "Option::is_none")]
    pub backoff_rate: Option<f64>,
}

/// Catch configuration for error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchConfig {
    /// Types of errors to catch
    #[serde(rename = "ErrorEquals")]
    pub error_equals: Vec<String>,

    /// Next state when error is caught
    #[serde(rename = "Next")]
    pub next: String,

    /// Where to store error information
    #[serde(rename = "ResultPath", skip_serializing_if = "Option::is_none")]
    pub result_path: Option<String>,
}

impl Workflow {
    /// Create a new empty workflow
    pub fn new(start_at: String) -> Self {
        Self {
            comment: None,
            start_at,
            states: HashMap::new(),
        }
    }

    /// Add a comment to the workflow
    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    /// Add a state to the workflow
    pub fn add_state(mut self, name: String, state: State) -> Self {
        self.states.insert(name, state);
        self
    }

    /// Validate the workflow structure
    #[instrument(skip_all, fields(operation = "validate_workflow"))]
    pub fn validate(&self) -> Result<(), String> {
        // Check if start state exists
        if !self.states.contains_key(&self.start_at) {
            return Err(format!(
                "StartAt state '{}' not found in states",
                self.start_at
            ));
        }

        // Validate each state
        for (name, state) in &self.states {
            self.validate_state(name, state)?;
        }

        Ok(())
    }

    #[instrument(skip_all, fields(operation = "validate_state", state_name = %name))]
    fn validate_state(&self, name: &str, state: &State) -> Result<(), String> {
        match state {
            State::Task(task) => {
                // Check Next or End is specified
                if task.next.is_none() && task.end != Some(true) {
                    return Err(format!(
                        "Task state '{}' must have either Next or End",
                        name
                    ));
                }

                // Check Next state exists if specified
                if let Some(next_state) = &task.next {
                    if !self.states.contains_key(next_state) {
                        return Err(format!(
                            "Task state '{}' references non-existent Next state '{}'",
                            name, next_state
                        ));
                    }
                }
            }
            State::Choice(choice) => {
                // Validate all choice rules
                for (i, rule) in choice.choices.iter().enumerate() {
                    if !self.states.contains_key(&rule.next) {
                        return Err(format!(
                            "Choice state '{}' rule {} references non-existent Next state '{}'",
                            name, i, rule.next
                        ));
                    }
                }

                // Validate default state if specified
                if let Some(default_state) = &choice.default {
                    if !self.states.contains_key(default_state) {
                        return Err(format!(
                            "Choice state '{}' references non-existent Default state '{}'",
                            name, default_state
                        ));
                    }
                }
            }
            State::Wait(wait) => {
                // Similar validation for Wait states
                if wait.next.is_none() && wait.end != Some(true) {
                    return Err(format!(
                        "Wait state '{}' must have either Next or End",
                        name
                    ));
                }
            }
            State::Succeed(_) | State::Fail(_) => {
                // Terminal states don't need Next validation
            }
        }

        Ok(())
    }
}

// Helper functions for common state creation - RUSTWS Clean Approach
impl TaskState {
    /// Create a simple task state with resource and next
    /// Clean RUSTWS approach - just connect lambdas, no transformation fields
    pub fn new(resource: String, next: String) -> Self {
        Self {
            resource,
            next: Some(next),
            end: None,
            retry: None,
            catch: None,
        }
    }

    /// Create a terminal task state (with End: true)
    /// Clean RUSTWS approach - minimal fields for terminal states
    pub fn terminal(resource: String) -> Self {
        Self {
            resource,
            next: None,
            end: Some(true),
            retry: None,
            catch: None,
        }
    }

    /// Add retry configuration to task state
    pub fn with_retry(mut self, retry: Vec<RetryConfig>) -> Self {
        self.retry = Some(retry);
        self
    }

    /// Add catch configuration to task state  
    pub fn with_catch(mut self, catch: Vec<CatchConfig>) -> Self {
        self.catch = Some(catch);
        self
    }
}

impl ChoiceRule {
    /// Create a string equality choice rule
    pub fn string_equals(variable: String, value: String, next: String) -> Self {
        Self {
            variable,
            next,
            string_equals: Some(value),
            boolean_equals: None,
            numeric_equals: None,
            numeric_greater_than: None,
            numeric_less_than: None,
        }
    }

    /// Create a boolean equality choice rule
    pub fn boolean_equals(variable: String, value: bool, next: String) -> Self {
        Self {
            variable,
            next,
            string_equals: None,
            boolean_equals: Some(value),
            numeric_equals: None,
            numeric_greater_than: None,
            numeric_less_than: None,
        }
    }
}
