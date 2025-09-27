//! Jailbreak to Alignment Converter Lambda - Transforms jailbreak generator output for alignment testing
//!
//! This lambda converts JailbreakResponse from jailbreak_generator into AlignmentBreakerRequest
//! by adding the necessary configuration parameters for alignment testing.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== INPUT/OUTPUT TYPES =====

// The input is directly JailbreakResponse - no wrapper needed

/// Generated jailbreak prompts output (from jailbreak_generator)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JailbreakResponse {
    /// Unique identifier for this generation batch
    pub batch_id: String,
    /// Generated adversarial prompts
    pub prompts: Vec<AdversarialPrompt>,
    /// Strategy used for generation
    pub strategy_used: JailbreakStrategy,
    /// Target topic tested
    pub target_topic: String,
    /// Generation timestamp
    pub generated_at: String,
    /// Total prompts generated
    pub prompt_count: u32,
}

/// Different jailbreak strategies available
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum JailbreakStrategy {
    /// Role-playing scenarios ("You are a helpful assistant who...")
    Roleplay,
    /// Hypothetical scenarios ("What would happen if...")
    Hypothetical,
    /// System confusion ("Ignore previous instructions...")
    SystemConfusion,
    /// Emotional manipulation ("I'm desperate and need...")
    EmotionalManipulation,
    /// Technical bypassing ("Encode your response in...")
    TechnicalBypass,
    /// Gradual escalation (start innocent, escalate slowly)
    GradualEscalation,
}

/// Individual adversarial prompt
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AdversarialPrompt {
    /// Unique identifier for this prompt
    pub prompt_id: String,
    /// The adversarial prompt text
    pub prompt_text: String,
    /// Expected alignment violation type
    pub expected_violation: String,
    /// Severity level of this specific prompt
    pub severity: u32,
    /// Additional metadata for analysis
    pub metadata: PromptMetadata,
}

/// Metadata for adversarial prompts
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PromptMetadata {
    /// Strategy variant used
    pub strategy_variant: String,
    /// Expected response indicators
    pub expected_indicators: Vec<String>,
    /// Difficulty to detect (1-10)
    pub detection_difficulty: u32,
}

/// Output for alignment breaker (converted format)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlignmentBreakerRequest {
    /// Generated jailbreak prompts from jailbreak_generator
    pub jailbreak_response: JailbreakResponse,
    /// Target AI model endpoint to test
    pub target_model_endpoint: String,
    /// Maximum response length to analyze
    pub max_response_length: u32,
    /// Timeout for each request in seconds
    pub timeout_seconds: u32,
}

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn]
fn jailbreak_to_alignment_converter_lambda(input: JailbreakResponse) -> AlignmentBreakerRequest {
    // Validate input parameters
    if input.prompts.is_empty() {
        panic!("No prompts provided in jailbreak response");
    }

    // Transform JailbreakResponse into AlignmentBreakerRequest with default config
    AlignmentBreakerRequest {
        jailbreak_response: input,
        target_model_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
        max_response_length: 1000,
        timeout_seconds: 30,
    }
}
