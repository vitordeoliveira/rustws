//! Judge Enforcer Lambda - Fast failure on DENY decisions
//!
//! This lambda receives a JudgeResponse and immediately panics if the decision is DENY.
//! Designed for fast failure in step functions where AI correctness is critical.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== JUDGE TYPES (redeclared for compilation) =====

/// Simple binary judgment result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum JudgmentDecision {
    /// The prompt is approved/allowed
    APPROVE,
    /// The prompt is denied/rejected
    DENY,
}

/// Output from the judge lambda
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeResponse {
    /// The binary decision
    pub decision: JudgmentDecision,
    /// Brief explanation of the decision
    pub reason: String,
    /// Confidence level in the decision (0.0-1.0)
    pub confidence: f64,
    /// The criteria used for judgment
    pub criteria_used: String,
    /// The prompt/content that was judged
    pub prompt: String,
}

// ===== INPUT/OUTPUT TYPES =====

/// Output from the judge enforcer lambda (only returned on APPROVE)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeEnforcerResponse {
    /// Confirmation that judgment was approved
    pub status: String,
    /// The original judge response
    pub judge_response: JudgeResponse,
    /// The prompt/content that was judged
    pub prompt: String,
}

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features)]
fn judge_enforcer_lambda(input: JudgeResponse) -> JudgeEnforcerResponse {
    match input.decision {
        JudgmentDecision::DENY => {
            // Panic with detailed information for fast failure
            panic!(
                "Judge DENIED: {} (Criteria: {}, Confidence: {:.2})",
                input.reason, input.criteria_used, input.confidence
            );
        }
        JudgmentDecision::APPROVE => {
            // Return success response
            JudgeEnforcerResponse {
                status: "APPROVED".to_string(),
                judge_response: input.clone(),
                prompt: input.prompt,
            }
        }
    }
}
