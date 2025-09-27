//! Extract Documentation Lambda - Returns only the documentation content from judge enforcer response
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

// ===== OUTPUT TYPES =====

/// Final documentation output - just the content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentationOutput {
    /// The final approved documentation content
    pub documentation: String,
    /// Status indicating the documentation passed all quality checks
    pub status: String,
    /// Final quality score based on all validation steps
    pub quality_score: f64,
}

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features)]
fn extract_documentation_lambda(input: JudgeEnforcerResponse) -> DocumentationOutput {
    // The judge stores the original documentation in its prompt field
    let documentation = input.judge_response.prompt;
    
    // Calculate a quality score based on the judge confidence
    let quality_score = input.judge_response.confidence;
    
    DocumentationOutput {
        documentation,
        status: "APPROVED".to_string(),
        quality_score,
    }
}
