//! AI Doc to Judge Converter Lambda - Transforms AI doc generator output for judge evaluation
//!
//! This lambda converts AIDocGeneratorResponse into JudgeRequest for documentation
//! completeness evaluation using the judge lambda.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== INPUT/OUTPUT TYPES =====

/// AI Documentation Generator Response (input to this converter)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AIDocGeneratorResponse {
    /// The generated documentation content
    pub documentation: String,
    /// Type of content that was documented
    pub content_type: String,
    /// Template used for generation
    pub template_used: String,
    /// Whether examples were included
    pub includes_examples: bool,
    /// Generation metadata
    pub generation_info: GenerationInfo,
}

/// Information about the documentation generation process
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerationInfo {
    /// Method used for generation (e.g., "ai_assisted", "template_based")
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

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features = [env])]
fn ai_doc_to_judge_converter_lambda(input: AIDocGeneratorResponse) -> JudgeRequest {
    // Build context for the judge with relevant metadata
    let context = build_judge_context(&input);

    // Create judge request for documentation completeness evaluation
    JudgeRequest {
        prompt: input.documentation,
        judgment_criteria: "content_safety".to_string(),
        context: Some(context),
        use_ai_assistance: Some(false), // Use rule-based evaluation for speed
    }
}

// ===== HELPER FUNCTIONS =====

/// Build context information for the judge
fn build_judge_context(input: &AIDocGeneratorResponse) -> String {
    let mut context_parts = Vec::new();

    // Content type information
    context_parts.push(format!("Content Type: {}", input.content_type));
    context_parts.push(format!("Template Used: {}", input.template_used));
    context_parts.push(format!(
        "Generation Method: {}",
        input.generation_info.generation_method
    ));

    // Quality indicators
    context_parts.push(format!(
        "Generation Confidence: {:.2}",
        input.generation_info.confidence
    ));
    context_parts.push(format!("Word Count: {}", input.generation_info.word_count));
    context_parts.push(format!("Examples Included: {}", input.includes_examples));

    // Sections information
    if !input.generation_info.sections_included.is_empty() {
        context_parts.push(format!(
            "Sections Generated: {}",
            input.generation_info.sections_included.join(", ")
        ));
    }

    // Completeness requirements based on content type
    let completeness_requirements = get_completeness_requirements(&input.content_type);
    if !completeness_requirements.is_empty() {
        context_parts.push(format!(
            "Required Sections: {}",
            completeness_requirements.join(", ")
        ));
    }

    // Quality thresholds
    context_parts.push("Minimum Word Count: 50".to_string());
    if input.includes_examples {
        context_parts.push("Examples should be present and properly formatted".to_string());
    }

    context_parts.join("\n")
}

/// Get completeness requirements based on content type
fn get_completeness_requirements(content_type: &str) -> Vec<String> {
    match content_type.to_lowercase().as_str() {
        "rust_function" => vec![
            "overview".to_string(),
            "signature".to_string(),
            "description".to_string(),
            "parameters".to_string(),
            "returns".to_string(),
            "examples".to_string(),
        ],
        "api_endpoint" => vec![
            "overview".to_string(),
            "endpoint".to_string(),
            "description".to_string(),
            "request".to_string(),
            "response".to_string(),
            "examples".to_string(),
        ],
        "module" => vec![
            "overview".to_string(),
            "description".to_string(),
            "api".to_string(),
            "examples".to_string(),
        ],
        "library" => vec![
            "overview".to_string(),
            "installation".to_string(),
            "quickstart".to_string(),
            "api".to_string(),
            "examples".to_string(),
        ],
        _ => vec![
            "overview".to_string(),
            "description".to_string(),
            "examples".to_string(),
        ],
    }
}
