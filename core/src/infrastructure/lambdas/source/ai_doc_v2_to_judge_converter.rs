//! AI Doc V2 to Judge Converter Lambda - Transforms AI doc generator V2 output for judge evaluation
//!
//! This lambda converts AIDocGeneratorV2Response into JudgeRequest for content
//! safety evaluation using the judge lambda.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== INPUT/OUTPUT TYPES =====

/// AI Documentation Generator V2 Response (input to this converter)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AIDocGeneratorV2Response {
    /// The generated documentation content
    pub documentation: String,
    /// Type of content that was documented
    pub content_type: String,
    /// Target audience for the documentation
    pub target_audience: String,
    /// Generation metadata
    pub generation_info: GenerationInfo,
}

/// Information about the documentation generation process
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerationInfo {
    /// Method used for generation (always "ai_assisted" in V2)
    pub generation_method: String,
    /// Confidence in the generated content (0.0-1.0)
    pub confidence: f64,
    /// Estimated word count
    pub word_count: u32,
    /// Sections included in the documentation
    pub sections_included: Vec<String>,
}

/// Judge Request (output from this converter)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeRequest {
    /// The prompt/content to be judged
    pub prompt: String,
    /// What aspect to judge
    pub judgment_criteria: String,
    /// Optional context or additional rules for judgment
    pub context: Option<String>,
    /// Whether to use AI assistance for complex judgments
    pub use_ai_assistance: Option<bool>,
}

// ===== LAMBDA HANDLER =====

#[lambda_fn]
pub async fn handler_v2(input: AIDocGeneratorV2Response) -> JudgeRequest {
    // Create judgment criteria based on content type and target audience
    let judgment_criteria =
        determine_judgment_criteria(&input.content_type, &input.target_audience);

    // Create context for the judge with relevant information
    let context = create_judgment_context(&input);

    // Create and return the judge request directly
    JudgeRequest {
        prompt: input.documentation,
        judgment_criteria,
        context: Some(context),
        use_ai_assistance: Some(true), // Use AI for content safety evaluation
    }
}

// ===== HELPER FUNCTIONS =====

/// Determine appropriate judgment criteria based on content type and audience
fn determine_judgment_criteria(content_type: &str, target_audience: &str) -> String {
    match (content_type, target_audience) {
        // Children's content requires strict safety evaluation
        ("children_story", _) | (_, "children_ages_4_to_8") | (_, "children") => {
            "child_content_safety".to_string()
        }
        // Educational content for young audiences
        ("educational", audience) if audience.contains("children") || audience.contains("kids") => {
            "child_educational_safety".to_string()
        }
        // General content safety for other audiences
        _ => "content_safety".to_string(),
    }
}

/// Create judgment context with relevant information for the judge
fn create_judgment_context(input: &AIDocGeneratorV2Response) -> String {
    let mut context_parts = Vec::new();

    // Add content type information
    context_parts.push(format!("Content Type: {}", input.content_type));

    // Add target audience information
    context_parts.push(format!("Target Audience: {}", input.target_audience));

    // Add generation confidence
    context_parts.push(format!(
        "AI Confidence: {:.2}",
        input.generation_info.confidence
    ));

    // Add word count
    context_parts.push(format!("Word Count: {}", input.generation_info.word_count));

    // Add sections information if available
    if !input.generation_info.sections_included.is_empty() {
        context_parts.push(format!(
            "Sections: {}",
            input.generation_info.sections_included.join(", ")
        ));
    }

    // Special context for children's content
    if input.target_audience.contains("children") || input.content_type == "children_story" {
        context_parts.push("SPECIAL REQUIREMENT: This content is intended for children. Ensure it contains no inappropriate language, scary themes, violence, or adult concepts. Content should be positive, educational, and age-appropriate.".to_string());
    }

    context_parts.join("\n")
}
