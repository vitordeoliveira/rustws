//! Judge Lambda - Binary decision maker for prompt evaluation
//!
//! This lambda provides simple APPROVE/DENY judgments for various prompt evaluation scenarios.
//! It can judge SQL statements, content safety, policy compliance, or any binary decision task.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== INPUT/OUTPUT TYPES =====

/// Input for the judge lambda
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeRequest {
    /// The prompt/content to be judged
    pub prompt: String,
    /// What aspect to judge (e.g., "sql_delete_operations", "content_safety", "policy_compliance")
    pub judgment_criteria: String,
    /// Optional context or additional rules for judgment
    pub context: Option<String>,
    /// Whether to use AI assistance for complex judgments (requires API key)
    pub use_ai_assistance: Option<bool>,
}

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
    /// The prompt/content to be judged
    pub prompt: String,
}

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features = [env, http])]
fn judge_lambda(input: JudgeRequest) -> JudgeResponse {
    // Validate input
    if input.prompt.trim().is_empty() {
        return JudgeResponse {
            decision: JudgmentDecision::DENY,
            reason: "Empty prompt provided".to_string(),
            confidence: 1.0,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        };
    }

    // Route to appropriate judgment function based on criteria
    match input.judgment_criteria.to_lowercase().as_str() {
        "sql_delete_operations" => judge_sql_delete_operations(&input),
        "sql_safety" => judge_sql_safety(&input),
        "content_safety" => judge_content_safety(&input),
        "policy_compliance" => judge_policy_compliance(&input),
        "code_safety" => judge_code_safety(&input),
        _ => {
            // Use AI assistance for unknown criteria if enabled
            if input.use_ai_assistance.unwrap_or(false) {
                judge_with_ai_assistance(&input)
            } else {
                JudgeResponse {
                    decision: JudgmentDecision::DENY,
                    reason: format!("Unknown judgment criteria: {}", input.judgment_criteria),
                    confidence: 1.0,
                    criteria_used: input.judgment_criteria.clone(),
                    prompt: input.prompt.clone(),
                }
            }
        }
    }
}

// ===== JUDGMENT FUNCTIONS =====

/// Judge SQL statements for DELETE operations
fn judge_sql_delete_operations(input: &JudgeRequest) -> JudgeResponse {
    let prompt_lower = input.prompt.to_lowercase();

    // Check for DELETE keywords
    let delete_patterns = [
        "delete from",
        "delete ",
        " delete ",
        "truncate table",
        "truncate ",
        " truncate ",
        "drop table",
        "drop database",
        "drop schema",
        " drop ",
    ];

    let has_delete_operation = delete_patterns
        .iter()
        .any(|pattern| prompt_lower.contains(pattern));

    if has_delete_operation {
        JudgeResponse {
            decision: JudgmentDecision::DENY,
            reason: "SQL statement contains DELETE or destructive operations".to_string(),
            confidence: 0.95,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        }
    } else {
        JudgeResponse {
            decision: JudgmentDecision::APPROVE,
            reason: "SQL statement does not contain DELETE operations".to_string(),
            confidence: 0.9,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        }
    }
}

/// Judge SQL statements for general safety
fn judge_sql_safety(input: &JudgeRequest) -> JudgeResponse {
    let prompt_lower = input.prompt.to_lowercase();

    // Check for potentially dangerous SQL operations
    let dangerous_patterns = [
        "delete from",
        "truncate",
        "drop table",
        "drop database",
        "drop schema",
        "alter table",
        "create table",
        "create database",
        "create schema",
        "grant all",
        "revoke",
        "update ",
        "insert into",
        "exec ",
        "execute ",
        "sp_",
        "xp_",
        "cmdshell",
        "--",
        "/*",
        "*/",
        "union select",
        "or 1=1",
        "' or '",
    ];

    let has_dangerous_operation = dangerous_patterns
        .iter()
        .any(|pattern| prompt_lower.contains(pattern));

    if has_dangerous_operation {
        JudgeResponse {
            decision: JudgmentDecision::DENY,
            reason: "SQL statement contains potentially dangerous operations".to_string(),
            confidence: 0.9,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        }
    } else {
        JudgeResponse {
            decision: JudgmentDecision::APPROVE,
            reason: "SQL statement appears safe".to_string(),
            confidence: 0.85,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        }
    }
}

/// Judge content for safety issues
fn judge_content_safety(input: &JudgeRequest) -> JudgeResponse {
    let prompt_lower = input.prompt.to_lowercase();

    // Check for unsafe content patterns
    let unsafe_patterns = [
        "violence",
        "kill",
        "murder",
        "bomb",
        "weapon",
        "terrorist",
        "hack",
        "illegal",
        "drugs",
        "poison",
        "suicide",
        "hate",
        "discrimination",
        "racist",
        "sexist",
        "adult content",
        "nsfw",
        "explicit",
    ];

    let has_unsafe_content = unsafe_patterns
        .iter()
        .any(|pattern| prompt_lower.contains(pattern));

    if has_unsafe_content {
        JudgeResponse {
            decision: JudgmentDecision::DENY,
            reason: "Content contains potentially unsafe or inappropriate material".to_string(),
            confidence: 0.8,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        }
    } else {
        JudgeResponse {
            decision: JudgmentDecision::APPROVE,
            reason: "Content appears safe and appropriate".to_string(),
            confidence: 0.8,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        }
    }
}

/// Judge for policy compliance
fn judge_policy_compliance(input: &JudgeRequest) -> JudgeResponse {
    let prompt_lower = input.prompt.to_lowercase();

    // Check against policy violations (customizable based on context)
    let policy_violations = [
        "confidential",
        "secret",
        "private",
        "internal only",
        "do not share",
        "proprietary",
        "classified",
        "personal information",
        "pii",
        "social security",
        "credit card",
        "password",
        "api key",
    ];

    let has_policy_violation = policy_violations
        .iter()
        .any(|pattern| prompt_lower.contains(pattern));

    if has_policy_violation {
        JudgeResponse {
            decision: JudgmentDecision::DENY,
            reason: "Content may violate information sharing policies".to_string(),
            confidence: 0.75,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        }
    } else {
        JudgeResponse {
            decision: JudgmentDecision::APPROVE,
            reason: "Content complies with information sharing policies".to_string(),
            confidence: 0.8,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        }
    }
}

/// Judge code for safety issues
fn judge_code_safety(input: &JudgeRequest) -> JudgeResponse {
    let prompt_lower = input.prompt.to_lowercase();

    // Check for potentially dangerous code patterns
    let dangerous_code_patterns = [
        "rm -rf",
        "del /f",
        "format c:",
        "dd if=",
        "eval(",
        "exec(",
        "system(",
        "shell_exec(",
        "file_get_contents",
        "fopen(",
        "fwrite(",
        "require(",
        "include(",
        "import os",
        "import subprocess",
        "process.exec",
        "child_process",
        "fs.unlink",
        "os.remove",
        "os.system",
        "subprocess.call",
    ];

    let has_dangerous_code = dangerous_code_patterns
        .iter()
        .any(|pattern| prompt_lower.contains(pattern));

    if has_dangerous_code {
        JudgeResponse {
            decision: JudgmentDecision::DENY,
            reason: "Code contains potentially dangerous operations".to_string(),
            confidence: 0.9,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        }
    } else {
        JudgeResponse {
            decision: JudgmentDecision::APPROVE,
            reason: "Code appears safe".to_string(),
            confidence: 0.85,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        }
    }
}

/// Use AI assistance for complex judgments (requires OpenAI API key)
fn judge_with_ai_assistance(input: &JudgeRequest) -> JudgeResponse {
    // Get API key from environment
    let api_key = match get_env("OPENAPI_KEY") {
        Ok(Some(key)) => key,
        Ok(None) => {
            return JudgeResponse {
                decision: JudgmentDecision::DENY,
                reason: "AI assistance requires OPENAPI_KEY environment variable".to_string(),
                confidence: 1.0,
                criteria_used: input.judgment_criteria.clone(),
                prompt: input.prompt.clone(),
            };
        }
        Err(e) => {
            return JudgeResponse {
                decision: JudgmentDecision::DENY,
                reason: format!("Environment error: {}", e),
                confidence: 1.0,
                criteria_used: input.judgment_criteria.clone(),
                prompt: input.prompt.clone(),
            };
        }
    };

    // Prepare AI prompt for judgment
    let ai_prompt = format!(
        "You are a content judge. Evaluate the following prompt based on the criteria: '{}'\n\nPrompt to judge: \"{}\"\n\nContext: {}\n\nRespond with ONLY 'APPROVE' or 'DENY' followed by a brief reason.\nFormat: DECISION: [APPROVE/DENY]\nReason: [brief explanation]",
        input.judgment_criteria,
        input.prompt,
        input
            .context
            .as_deref()
            .unwrap_or("No additional context provided")
    );

    // Prepare OpenAI request
    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));

    let request_body = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": ai_prompt
            }
        ],
        "max_tokens": 100,
        "temperature": 0.1
    })
    .to_string();

    let request = Request {
        method: "POST".to_string(),
        url: "https://api.openai.com/v1/chat/completions".to_string(),
        headers,
        body: Some(request_body),
    };

    match http_request(&request) {
        Ok(response) if response.is_success() => {
            // Parse AI response
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(response.text()) {
                if let Some(content) = json
                    .get("choices")
                    .and_then(|choices| choices.get(0))
                    .and_then(|choice| choice.get("message"))
                    .and_then(|message| message.get("content"))
                    .and_then(|content| content.as_str())
                {
                    return parse_ai_judgment_response(
                        content,
                        &input.judgment_criteria,
                        &input.prompt,
                    );
                }
            }

            // Fallback if parsing fails
            JudgeResponse {
                decision: JudgmentDecision::DENY,
                reason: "Failed to parse AI judgment response".to_string(),
                confidence: 0.5,
                criteria_used: input.judgment_criteria.clone(),
                prompt: input.prompt.clone(),
            }
        }
        _ => {
            // AI assistance failed, deny by default for safety
            JudgeResponse {
                decision: JudgmentDecision::DENY,
                reason: "AI assistance unavailable, defaulting to DENY for safety".to_string(),
                confidence: 0.6,
                criteria_used: input.judgment_criteria.clone(),
                prompt: input.prompt.clone(),
            }
        }
    }
}

/// Parse AI judgment response
fn parse_ai_judgment_response(
    ai_response: &str,
    criteria: &str,
    original_prompt: &str,
) -> JudgeResponse {
    let response_lower = ai_response.to_lowercase();

    let decision = if response_lower.contains("approve") {
        JudgmentDecision::APPROVE
    } else {
        JudgmentDecision::DENY // Default to DENY for safety
    };

    // Extract reason if available
    let reason = if let Some(reason_start) = ai_response.find("Reason:") {
        ai_response[reason_start + 7..].trim().to_string()
    } else {
        format!("AI judgment: {}", decision_to_string(&decision))
    };

    JudgeResponse {
        decision,
        reason,
        confidence: 0.8, // AI assistance confidence
        criteria_used: criteria.to_string(),
        prompt: original_prompt.to_string(),
    }
}

/// Helper function to convert decision to string
fn decision_to_string(decision: &JudgmentDecision) -> &'static str {
    match decision {
        JudgmentDecision::APPROVE => "APPROVE",
        JudgmentDecision::DENY => "DENY",
    }
}
