//! AI Documentation Generator V2 - Simplified AI-only documentation generation
//!
//! This lambda generates documentation entirely using AI assistance, removing template-based
//! generation in favor of intelligent AI-driven content creation.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== INPUT/OUTPUT TYPES =====

/// Input for AI documentation generation V2
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AIDocGeneratorV2Request {
    /// The code or API specification to document
    pub source_content: String,
    /// Type of content being documented (e.g., "rust_function", "api_endpoint", "module", "library", "user_feature")
    pub content_type: String,
    /// Target audience (e.g., "developers", "end_users", "administrators", "technical_writers")
    pub target_audience: String,
    /// Additional context or requirements
    pub context: Option<String>,
}

/// Generated documentation output V2
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

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features = [env, http])]
fn ai_doc_generator_v2_lambda(input: AIDocGeneratorV2Request) -> AIDocGeneratorV2Response {
    // Validate input
    if input.source_content.trim().is_empty() {
        return create_error_response(&input, "Empty source content provided");
    }

    if input.target_audience.trim().is_empty() {
        return create_error_response(&input, "Target audience is required");
    }

    // Always use AI assistance for documentation generation
    generate_with_ai_assistance(&input)
}

// ===== AI DOCUMENTATION GENERATION =====

/// Generate documentation using AI assistance (requires OpenAI API key)
fn generate_with_ai_assistance(input: &AIDocGeneratorV2Request) -> AIDocGeneratorV2Response {
    // Get API key from environment
    let api_key = match get_env("OPENAPI_KEY") {
        Ok(Some(key)) => key,
        Ok(None) => {
            return create_error_response(
                input,
                "AI documentation generation requires OPENAPI_KEY environment variable",
            );
        }
        Err(e) => {
            return create_error_response(input, &format!("Environment error: {}", e));
        }
    };

    // Create specialized AI prompt based on content type and audience
    let ai_prompt = create_documentation_prompt(input);

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
        "max_tokens": 3000,
        "temperature": 0.2
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
                    // Extract confidence score and clean documentation
                    match extract_documentation_and_confidence(content) {
                        Ok((documentation, confidence)) => {
                            let sections = extract_sections_from_content(&documentation);
                            let word_count = estimate_word_count(&documentation);
                            
                            return AIDocGeneratorV2Response {
                                documentation,
                                content_type: input.content_type.clone(),
                                target_audience: input.target_audience.clone(),
                                generation_info: GenerationInfo {
                                    generation_method: "ai_assisted".to_string(),
                                    confidence,
                                    word_count,
                                    sections_included: sections,
                                },
                            };
                        }
                        Err(error) => {
                            return create_error_response(input, &format!("Failed to parse AI response: {}", error));
                        }
                    }
                }
            }

            create_error_response(input, "Failed to parse AI documentation response")
        }
        _ => create_error_response(input, "AI documentation generation unavailable"),
    }
}

// ===== HELPER FUNCTIONS =====

/// Create specialized AI prompts based on content type and target audience
fn create_documentation_prompt(input: &AIDocGeneratorV2Request) -> String {
    let context_str = input.context.as_deref().unwrap_or("No additional context provided");
    
    let audience_guidance = match input.target_audience.to_lowercase().as_str() {
        "developers" => "Use technical language, include code examples, focus on implementation details, and provide API references.",
        "end_users" => "Use simple, clear language, focus on how to use the feature, include step-by-step instructions, and avoid technical jargon.",
        "administrators" => "Focus on configuration, deployment, security considerations, and operational aspects.",
        "technical_writers" => "Provide comprehensive coverage, include all technical details, and structure for easy editing and expansion.",
        _ => "Adapt the language and detail level appropriately for the specified audience.",
    };

    let content_type_guidance = match input.content_type.to_lowercase().as_str() {
        "rust_function" => "Document the function signature, parameters, return values, usage examples, and any important implementation notes.",
        "api_endpoint" => "Document the HTTP method, URL, request/response formats, status codes, authentication requirements, and provide curl examples.",
        "module" => "Document the module's purpose, public API, key functions/types, usage patterns, and integration examples.",
        "library" => "Create comprehensive documentation including installation, quick start guide, API reference, examples, and best practices.",
        "user_feature" => "Focus on user benefits, how to access/use the feature, step-by-step instructions, and common use cases.",
        _ => "Analyze the content and create appropriate documentation structure and content.",
    };

    format!(
        "You are an expert technical documentation writer. Create comprehensive, well-structured documentation for the following content.\n\n\
        **Content Type**: {}\n\
        **Target Audience**: {}\n\
        **Content to Document**:\n{}\n\n\
        **Context**: {}\n\n\
        **Audience Guidelines**: {}\n\n\
        **Content Type Guidelines**: {}\n\n\
        **Requirements**:\n\
        - Use proper Markdown formatting\n\
        - Create clear, logical section headers\n\
        - Include examples where appropriate\n\
        - Ensure content is accurate and complete\n\
        - Match the tone and complexity to the target audience\n\
        - Structure the documentation for easy reading and navigation\n\n\
        Generate the complete documentation now.\n\n\
        After the documentation, add a confidence assessment:\n\
        Confidence: [0.0-1.0 score indicating how confident you are in the quality and completeness of this documentation]",
        input.content_type,
        input.target_audience,
        input.source_content,
        context_str,
        audience_guidance,
        content_type_guidance
    )
}

/// Extract documentation content and confidence score from AI response
fn extract_documentation_and_confidence(content: &str) -> Result<(String, f64), String> {
    // Look for confidence score at the end of the response
    if let Some(confidence_start) = content.rfind("Confidence:") {
        let documentation = content[..confidence_start].trim().to_string();
        let confidence_text = &content[confidence_start + 11..];
        
        // Extract the confidence score
        if let Some(confidence_str) = confidence_text.split_whitespace().next() {
            if let Ok(confidence) = confidence_str.parse::<f64>() {
                let clamped_confidence = confidence.max(0.0).min(1.0);
                return Ok((documentation, clamped_confidence));
            } else {
                return Err(format!("Invalid confidence score format: {}", confidence_str));
            }
        } else {
            return Err("Confidence score value not found after 'Confidence:' marker".to_string());
        }
    } else {
        return Err("AI did not provide required confidence score".to_string());
    }
}

/// Extract section names from generated content by looking for markdown headers
fn extract_sections_from_content(content: &str) -> Vec<String> {
    let mut sections = Vec::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            // Extract header text, removing # symbols and cleaning up
            let header_text = trimmed
                .trim_start_matches('#')
                .trim()
                .to_lowercase()
                .replace(' ', "_");
            
            if !header_text.is_empty() {
                sections.push(header_text);
            }
        }
    }
    
    // If no sections found, add a default
    if sections.is_empty() {
        sections.push("ai_generated".to_string());
    }
    
    sections
}

/// Create an error response
fn create_error_response(input: &AIDocGeneratorV2Request, error_msg: &str) -> AIDocGeneratorV2Response {
    AIDocGeneratorV2Response {
        documentation: format!("# Documentation Generation Error\n\n{}", error_msg),
        content_type: input.content_type.clone(),
        target_audience: input.target_audience.clone(),
        generation_info: GenerationInfo {
            generation_method: "error".to_string(),
            confidence: 0.0,
            word_count: estimate_word_count(error_msg),
            sections_included: vec!["error".to_string()],
        },
    }
}

/// Estimate word count in text
fn estimate_word_count(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}
