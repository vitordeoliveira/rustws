//! Converter Lambda - Judge Enforcer Response to Example Validator Request
use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== JUDGE ENFORCER TYPES (redeclared for compilation) =====
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum JudgmentDecision {
    APPROVE,
    DENY,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeResponse {
    pub decision: JudgmentDecision,
    pub reason: String,
    pub confidence: f64,
    pub criteria_used: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeEnforcerResponse {
    pub status: String,
    pub judge_response: JudgeResponse,
    pub prompt: String,
}

// ===== EXAMPLE VALIDATOR TYPES (redeclared for compilation) =====
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExampleValidatorRequest {
    /// The documentation content containing examples
    pub documentation: String,
    /// Type of examples to validate (e.g., "rust", "javascript", "shell", "json")
    pub example_types: Vec<String>,
    /// Validation configuration
    pub validation_config: ValidationConfig,
    /// Additional context for validation
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationConfig {
    /// Whether to check syntax for code examples
    pub check_syntax: Option<bool>,
    /// Whether examples should be executable/runnable
    pub require_executable: Option<bool>,
    /// Whether to validate that examples compile (for compiled languages)
    pub check_compilation: Option<bool>,
    /// Maximum number of examples to validate (for performance)
    pub max_examples: Option<u32>,
    /// Whether to check for best practices
    pub check_best_practices: Option<bool>,
}

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features)]
fn enforcer_to_example_converter_lambda(input: JudgeEnforcerResponse) -> ExampleValidatorRequest {
    // Extract the original documentation from the judge response
    // The judge stores the original content that was judged in its prompt field
    let documentation = input.judge_response.prompt.clone();

    // Create validation configuration for code examples
    let validation_config = ValidationConfig {
        check_syntax: Some(true),
        require_executable: Some(false),
        check_compilation: Some(true),
        max_examples: Some(10),
        check_best_practices: Some(true),
    };

    ExampleValidatorRequest {
        documentation,
        example_types: vec![
            "rust".to_string(),
            "python".to_string(),
            "javascript".to_string(),
            "typescript".to_string(),
            "shell".to_string(),
            "json".to_string(),
        ],
        validation_config,
        context: Some("AI-generated function documentation examples".to_string()),
    }
}
