//! AI Judge Lambda - AI-powered binary decision maker for prompt evaluation
//!
//! This lambda provides APPROVE/DENY judgments using AI assistance for all evaluation scenarios.
//! Unlike the regular judge lambda, this always uses AI for decision making.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== INPUT/OUTPUT TYPES =====

/// Input for the AI judge lambda
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AIJudgeRequest {
    /// The prompt/content to be judged
    pub prompt: String,
    /// What aspect to judge (e.g., "documentation_completeness", "technical_accuracy", "clarity_check")
    pub judgment_criteria: String,
    /// Optional context or additional rules for judgment
    pub context: Option<String>,
}

/// Simple binary judgment result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum JudgmentDecision {
    /// The prompt is approved/allowed
    APPROVE,
    /// The prompt is denied/rejected
    DENY,
}

/// Output from the AI judge lambda
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AIJudgeResponse {
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

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features = [env, http])]
fn ai_judge_lambda(input: AIJudgeRequest) -> AIJudgeResponse {
    // Validate input
    if input.prompt.trim().is_empty() {
        return AIJudgeResponse {
            decision: JudgmentDecision::DENY,
            reason: "Empty prompt provided".to_string(),
            confidence: 1.0,
            criteria_used: input.judgment_criteria.clone(),
            prompt: input.prompt.clone(),
        };
    }

    // Always use AI assistance for judgment
    judge_with_ai_assistance(&input)
}

// ===== AI JUDGMENT FUNCTION =====

/// Use AI assistance for all judgments (requires OpenAI API key)
fn judge_with_ai_assistance(input: &AIJudgeRequest) -> AIJudgeResponse {
    // Get API key from environment
    let api_key = match get_env("OPENAPI_KEY") {
        Ok(Some(key)) => key,
        Ok(None) => {
            return AIJudgeResponse {
                decision: JudgmentDecision::DENY,
                reason: "AI judgment requires OPENAPI_KEY environment variable".to_string(),
                confidence: 1.0,
                criteria_used: input.judgment_criteria.clone(),
                prompt: input.prompt.clone(),
            };
        }
        Err(e) => {
            return AIJudgeResponse {
                decision: JudgmentDecision::DENY,
                reason: format!("Environment error: {}", e),
                confidence: 1.0,
                criteria_used: input.judgment_criteria.clone(),
                prompt: input.prompt.clone(),
            };
        }
    };

    // Prepare AI prompt for judgment based on criteria
    let ai_prompt = create_ai_prompt_for_criteria(&input.judgment_criteria, &input.prompt, input.context.as_deref());

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
        "max_tokens": 200,
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
            AIJudgeResponse {
                decision: JudgmentDecision::DENY,
                reason: "Failed to parse AI judgment response".to_string(),
                confidence: 0.5,
                criteria_used: input.judgment_criteria.clone(),
                prompt: input.prompt.clone(),
            }
        }
        _ => {
            // AI assistance failed, deny by default for safety
            AIJudgeResponse {
                decision: JudgmentDecision::DENY,
                reason: "AI assistance unavailable, defaulting to DENY for safety".to_string(),
                confidence: 0.6,
                criteria_used: input.judgment_criteria.clone(),
                prompt: input.prompt.clone(),
            }
        }
    }
}

// ===== HELPER FUNCTIONS =====

/// Create specialized AI prompts based on judgment criteria
fn create_ai_prompt_for_criteria(criteria: &str, content: &str, context: Option<&str>) -> String {
    let context_str = context.unwrap_or("No additional context provided");
    
    match criteria.to_lowercase().as_str() {
        "documentation_completeness" => format!(
            "You are a documentation quality expert. Evaluate if the following documentation is complete and comprehensive.\n\n\
            Documentation to evaluate:\n{}\n\n\
            Context: {}\n\n\
            Check for:\n\
            - Clear description of purpose\n\
            - Proper structure and organization\n\
            - Adequate examples where appropriate\n\
            - Complete coverage of the topic\n\
            - Professional writing quality\n\n\
            Respond with ONLY 'APPROVE' if the documentation is complete and well-written, or 'DENY' if it's incomplete, unclear, or poorly structured.\n\
            Format: DECISION: [APPROVE/DENY]\n\
            Reason: [brief explanation]\n\
            Confidence: [0.0-1.0 score indicating how confident you are in this decision]",
            content, context_str
        ),
        "technical_accuracy" => format!(
            "You are a technical accuracy expert. Evaluate if the following content is technically accurate and follows best practices.\n\n\
            Content to evaluate:\n{}\n\n\
            Context: {}\n\n\
            Check for:\n\
            - Technical correctness\n\
            - Adherence to best practices\n\
            - Proper formatting and structure\n\
            - Accurate code examples (if any)\n\
            - Consistent terminology\n\n\
            Respond with ONLY 'APPROVE' if the content is technically accurate, or 'DENY' if there are technical errors or issues.\n\
            Format: DECISION: [APPROVE/DENY]\n\
            Reason: [brief explanation]\n\
            Confidence: [0.0-1.0 score indicating how confident you are in this decision]",
            content, context_str
        ),
        "clarity_check" => format!(
            "You are a clarity and readability expert. Evaluate if the following content is clear, understandable, and well-written.\n\n\
            Content to evaluate:\n{}\n\n\
            Context: {}\n\n\
            Check for:\n\
            - Clear and concise language\n\
            - Logical flow and organization\n\
            - Appropriate level of detail\n\
            - Good examples that aid understanding\n\
            - Consistent tone and style\n\n\
            Respond with ONLY 'APPROVE' if the content is clear and easy to understand, or 'DENY' if it's confusing, unclear, or poorly written.\n\
            Format: DECISION: [APPROVE/DENY]\n\
            Reason: [brief explanation]\n\
            Confidence: [0.0-1.0 score indicating how confident you are in this decision]",
            content, context_str
        ),
        "content_safety" => format!(
            "You are a content safety expert. Evaluate if the following content is safe and appropriate.\n\n\
            Content to evaluate:\n{}\n\n\
            Context: {}\n\n\
            Check for:\n\
            - No harmful or dangerous content\n\
            - No inappropriate or offensive material\n\
            - No security risks or vulnerabilities\n\
            - Professional and respectful tone\n\
            - Compliance with content guidelines\n\n\
            Respond with ONLY 'APPROVE' if the content is safe and appropriate, or 'DENY' if it contains unsafe or inappropriate material.\n\
            Format: DECISION: [APPROVE/DENY]\n\
            Reason: [brief explanation]\n\
            Confidence: [0.0-1.0 score indicating how confident you are in this decision]",
            content, context_str
        ),
        "code_safety" => format!(
            "You are a code security expert. Evaluate if the following code or code-related content is safe and secure.\n\n\
            Content to evaluate:\n{}\n\n\
            Context: {}\n\n\
            Check for:\n\
            - No dangerous or destructive operations\n\
            - No security vulnerabilities\n\
            - Safe coding practices\n\
            - No malicious code patterns\n\
            - Proper error handling\n\n\
            Respond with ONLY 'APPROVE' if the code is safe and secure, or 'DENY' if it contains dangerous or insecure elements.\n\
            Format: DECISION: [APPROVE/DENY]\n\
            Reason: [brief explanation]\n\
            Confidence: [0.0-1.0 score indicating how confident you are in this decision]",
            content, context_str
        ),
        _ => format!(
            "You are a content evaluator. Evaluate the following content based on the criteria: '{}'\n\n\
            Content to evaluate:\n{}\n\n\
            Context: {}\n\n\
            Provide a thorough evaluation based on the specified criteria and determine if the content meets the requirements.\n\n\
            Respond with ONLY 'APPROVE' if the content meets the criteria, or 'DENY' if it does not.\n\
            Format: DECISION: [APPROVE/DENY]\n\
            Reason: [brief explanation]\n\
            Confidence: [0.0-1.0 score indicating how confident you are in this decision]",
            criteria, content, context_str
        )
    }
}

/// Parse AI judgment response
fn parse_ai_judgment_response(
    ai_response: &str,
    criteria: &str,
    original_prompt: &str,
) -> AIJudgeResponse {
    let response_lower = ai_response.to_lowercase();

    let decision = if response_lower.contains("approve") {
        JudgmentDecision::APPROVE
    } else {
        JudgmentDecision::DENY // Default to DENY for safety
    };

    // Extract reason if available
    let reason = if let Some(reason_start) = ai_response.find("Reason:") {
        let reason_end = ai_response[reason_start..].find("Confidence:").unwrap_or(ai_response.len() - reason_start);
        ai_response[reason_start + 7..reason_start + reason_end].trim().to_string()
    } else {
        format!("AI judgment: {}", decision_to_string(&decision))
    };

    // Extract confidence score - AI must provide this
    let confidence = if let Some(confidence_start) = ai_response.find("Confidence:") {
        let confidence_text = &ai_response[confidence_start + 11..];
        // Extract the first number found after "Confidence:"
        if let Some(confidence_str) = confidence_text.split_whitespace().next() {
            if let Ok(conf) = confidence_str.parse::<f64>() {
                conf.max(0.0).min(1.0) // Clamp between 0.0 and 1.0
            } else {
                // If confidence parsing fails, return a DENY with error
                return AIJudgeResponse {
                    decision: JudgmentDecision::DENY,
                    reason: "AI provided invalid confidence score format".to_string(),
                    confidence: 0.0,
                    criteria_used: criteria.to_string(),
                    prompt: original_prompt.to_string(),
                };
            }
        } else {
            // If no confidence value found after marker, return a DENY with error
            return AIJudgeResponse {
                decision: JudgmentDecision::DENY,
                reason: "AI confidence score value missing after marker".to_string(),
                confidence: 0.0,
                criteria_used: criteria.to_string(),
                prompt: original_prompt.to_string(),
            };
        }
    } else {
        // If no confidence found, return a DENY with error
        return AIJudgeResponse {
            decision: JudgmentDecision::DENY,
            reason: "AI did not provide required confidence score".to_string(),
            confidence: 0.0,
            criteria_used: criteria.to_string(),
            prompt: original_prompt.to_string(),
        };
    };

    AIJudgeResponse {
        decision,
        reason,
        confidence,
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
