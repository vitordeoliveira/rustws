//! Converter Lambda - Judge Enforcer Response to Format Checker Request
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

// ===== FORMAT CHECKER TYPES (redeclared for compilation) =====
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormatCheckerRequest {
    /// The documentation content to check
    pub documentation: String,
    /// Type of content being checked (e.g., "markdown", "api_reference", "user_guide")
    pub content_type: String,
    /// Format requirements to apply
    pub format_requirements: FormatRequirements,
    /// Additional context for validation
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormatRequirements {
    /// Required sections that must be present
    pub required_sections: Vec<String>,
    /// Maximum line length allowed
    pub max_line_length: Option<u32>,
    /// Whether code blocks must have language specified
    pub require_code_block_language: Option<bool>,
    /// Whether headers must follow hierarchy (h1 -> h2 -> h3)
    pub enforce_header_hierarchy: Option<bool>,
    /// Whether links must be valid format
    pub validate_links: Option<bool>,
}

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features)]
fn enforcer_to_format_converter_lambda(input: JudgeEnforcerResponse) -> FormatCheckerRequest {
    // Extract the original documentation from the judge response
    // The judge stores the original content that was judged in its prompt field
    let documentation = input.judge_response.prompt.clone();

    // Create format requirements for documentation
    let format_requirements = FormatRequirements {
        required_sections: vec!["Description".to_string(), "Examples".to_string()],
        max_line_length: Some(100),
        require_code_block_language: Some(true),
        enforce_header_hierarchy: Some(true),
        validate_links: Some(true),
    };

    FormatCheckerRequest {
        documentation,
        content_type: "markdown".to_string(),
        format_requirements,
        context: Some("AI-generated function documentation".to_string()),
    }
}
