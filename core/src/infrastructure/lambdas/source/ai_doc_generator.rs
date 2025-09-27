//! AI Documentation Generator Lambda - Generates documentation from code/API inputs
//!
//! This lambda takes code or API specifications and generates comprehensive documentation
//! using AI assistance with configurable templates and styles.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== INPUT/OUTPUT TYPES =====

/// Input for AI documentation generation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AIDocGeneratorRequest {
    /// The code or API specification to document
    pub source_content: String,
    /// Type of content being documented (e.g., "rust_function", "api_endpoint", "module")
    pub content_type: String,
    /// Documentation template to use (e.g., "api_reference", "user_guide", "technical_spec")
    pub template_type: String,
    /// Target audience (e.g., "developers", "end_users", "administrators")
    pub target_audience: Option<String>,
    /// Additional context or requirements
    pub context: Option<String>,
    /// Whether to include code examples
    pub include_examples: Option<bool>,
}

/// Generated documentation output
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

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features = [env, http])]
fn ai_doc_generator_lambda(input: AIDocGeneratorRequest) -> AIDocGeneratorResponse {
    // Starting documentation generation

    // Validate input
    if input.source_content.trim().is_empty() {
        return create_error_response(&input, "Empty source content provided");
    }

    // Determine generation approach based on content type
    match input.content_type.to_lowercase().as_str() {
        "rust_function" => generate_rust_function_docs(&input),
        "api_endpoint" => generate_api_endpoint_docs(&input),
        "module" => generate_module_docs(&input),
        "library" => generate_library_docs(&input),
        _ => {
            // Try AI-assisted generation for unknown content types
            generate_with_ai_assistance(&input)
        }
    }
}

// ===== DOCUMENTATION GENERATION FUNCTIONS =====

/// Generate documentation for Rust functions
fn generate_rust_function_docs(input: &AIDocGeneratorRequest) -> AIDocGeneratorResponse {
    // Generating Rust function documentation

    let mut sections = Vec::new();
    let mut documentation = String::new();

    // Function signature and overview
    documentation.push_str("# Function Documentation\n\n");
    sections.push("overview".to_string());

    // Extract function information (simplified parsing)
    if let Some(function_info) = extract_rust_function_info(&input.source_content) {
        documentation.push_str(&format!("## {}\n\n", function_info.name));
        documentation.push_str(&format!("```rust\n{}\n```\n\n", function_info.signature));
        sections.push("signature".to_string());

        // Description
        documentation.push_str("## Description\n\n");
        documentation.push_str(&generate_function_description(&function_info));
        documentation.push_str("\n\n");
        sections.push("description".to_string());

        // Parameters
        if !function_info.parameters.is_empty() {
            documentation.push_str("## Parameters\n\n");
            for param in &function_info.parameters {
                documentation.push_str(&format!("- `{}`: {}\n", param.name, param.description));
            }
            documentation.push_str("\n");
            sections.push("parameters".to_string());
        }

        // Return value
        if !function_info.return_type.is_empty() {
            documentation.push_str("## Returns\n\n");
            documentation.push_str(&format!(
                "Returns `{}`: {}\n\n",
                function_info.return_type, function_info.return_description
            ));
            sections.push("returns".to_string());
        }

        // Examples
        if input.include_examples.unwrap_or(true) {
            documentation.push_str("## Examples\n\n");
            documentation.push_str(&generate_usage_examples(&function_info));
            sections.push("examples".to_string());
        }
    } else {
        documentation.push_str("Unable to parse function information from provided source.\n");
    }

    AIDocGeneratorResponse {
        documentation: documentation.clone(),
        content_type: input.content_type.clone(),
        template_used: input.template_type.clone(),
        includes_examples: input.include_examples.unwrap_or(true),
        generation_info: GenerationInfo {
            generation_method: "template_based".to_string(),
            confidence: 0.85,
            word_count: estimate_word_count(&documentation),
            sections_included: sections,
        },
    }
}

/// Generate documentation for API endpoints
fn generate_api_endpoint_docs(input: &AIDocGeneratorRequest) -> AIDocGeneratorResponse {
    // Generating API endpoint documentation

    let mut sections = Vec::new();
    let mut documentation = String::new();

    documentation.push_str("# API Endpoint Documentation\n\n");
    sections.push("overview".to_string());

    // Try to extract endpoint information
    if let Some(endpoint_info) = extract_api_endpoint_info(&input.source_content) {
        documentation.push_str(&format!(
            "## {} {}\n\n",
            endpoint_info.method, endpoint_info.path
        ));
        sections.push("endpoint".to_string());

        documentation.push_str(&format!("{}\n\n", endpoint_info.description));
        sections.push("description".to_string());

        // Request format
        if !endpoint_info.request_body.is_empty() {
            documentation.push_str("## Request Body\n\n");
            documentation.push_str(&format!("```json\n{}\n```\n\n", endpoint_info.request_body));
            sections.push("request".to_string());
        }

        // Response format
        if !endpoint_info.response_body.is_empty() {
            documentation.push_str("## Response\n\n");
            documentation.push_str(&format!(
                "```json\n{}\n```\n\n",
                endpoint_info.response_body
            ));
            sections.push("response".to_string());
        }

        // Examples
        if input.include_examples.unwrap_or(true) {
            documentation.push_str("## Example Usage\n\n");
            documentation.push_str(&generate_api_examples(&endpoint_info));
            sections.push("examples".to_string());
        }
    } else {
        documentation.push_str("Unable to parse API endpoint information from provided source.\n");
    }

    AIDocGeneratorResponse {
        documentation: documentation.clone(),
        content_type: input.content_type.clone(),
        template_used: input.template_type.clone(),
        includes_examples: input.include_examples.unwrap_or(true),
        generation_info: GenerationInfo {
            generation_method: "template_based".to_string(),
            confidence: 0.80,
            word_count: estimate_word_count(&documentation),
            sections_included: sections,
        },
    }
}

/// Generate documentation for modules
fn generate_module_docs(input: &AIDocGeneratorRequest) -> AIDocGeneratorResponse {
    // Generating module documentation

    let mut sections = Vec::new();
    let mut documentation = String::new();

    documentation.push_str("# Module Documentation\n\n");
    sections.push("overview".to_string());

    // Basic module information
    documentation.push_str("## Overview\n\n");
    documentation
        .push_str("This module provides functionality for [describe the module's purpose].\n\n");
    sections.push("description".to_string());

    // Exports/Public API
    documentation.push_str("## Public API\n\n");
    documentation.push_str("### Functions\n\n");
    documentation.push_str("### Types\n\n");
    documentation.push_str("### Constants\n\n");
    sections.push("api".to_string());

    // Usage examples
    if input.include_examples.unwrap_or(true) {
        documentation.push_str("## Usage Examples\n\n");
        documentation.push_str("```rust\n// Example usage\n```\n\n");
        sections.push("examples".to_string());
    }

    AIDocGeneratorResponse {
        documentation: documentation.clone(),
        content_type: input.content_type.clone(),
        template_used: input.template_type.clone(),
        includes_examples: input.include_examples.unwrap_or(true),
        generation_info: GenerationInfo {
            generation_method: "template_based".to_string(),
            confidence: 0.75,
            word_count: estimate_word_count(&documentation),
            sections_included: sections,
        },
    }
}

/// Generate documentation for libraries
fn generate_library_docs(input: &AIDocGeneratorRequest) -> AIDocGeneratorResponse {
    // Generating library documentation

    let mut sections = Vec::new();
    let mut documentation = String::new();

    documentation.push_str("# Library Documentation\n\n");
    sections.push("overview".to_string());

    documentation.push_str("## Installation\n\n");
    documentation.push_str("## Quick Start\n\n");
    documentation.push_str("## API Reference\n\n");
    documentation.push_str("## Examples\n\n");

    sections.extend(
        ["installation", "quickstart", "api", "examples"]
            .iter()
            .map(|s| s.to_string()),
    );

    AIDocGeneratorResponse {
        documentation: documentation.clone(),
        content_type: input.content_type.clone(),
        template_used: input.template_type.clone(),
        includes_examples: input.include_examples.unwrap_or(true),
        generation_info: GenerationInfo {
            generation_method: "template_based".to_string(),
            confidence: 0.70,
            word_count: estimate_word_count(&documentation),
            sections_included: sections,
        },
    }
}

/// Use AI assistance for unknown content types (requires OpenAI API key)
fn generate_with_ai_assistance(input: &AIDocGeneratorRequest) -> AIDocGeneratorResponse {
    // Using AI assistance for documentation generation

    // Get API key from environment
    let api_key = match get_env("OPENAPI_KEY") {
        Ok(Some(key)) => key,
        Ok(None) => {
            return create_error_response(
                input,
                "AI assistance requires OPENAPI_KEY environment variable",
            );
        }
        Err(e) => {
            return create_error_response(input, &format!("Environment error: {}", e));
        }
    };

    // Prepare AI prompt for documentation generation
    let ai_prompt = format!(
        "You are a technical documentation writer. Generate comprehensive documentation for the following {} content:\n\nContent:\n{}\n\nTemplate type: {}\nTarget audience: {}\nContext: {}\n\nGenerate well-structured documentation with appropriate sections, clear explanations, and examples if requested.",
        input.content_type,
        input.source_content,
        input.template_type,
        input.target_audience.as_deref().unwrap_or("developers"),
        input.context.as_deref().unwrap_or("No additional context")
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
        "max_tokens": 2000,
        "temperature": 0.3
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
                    return AIDocGeneratorResponse {
                        documentation: content.to_string(),
                        content_type: input.content_type.clone(),
                        template_used: input.template_type.clone(),
                        includes_examples: input.include_examples.unwrap_or(true),
                        generation_info: GenerationInfo {
                            generation_method: "ai_assisted".to_string(),
                            confidence: 0.90,
                            word_count: estimate_word_count(content),
                            sections_included: vec!["ai_generated".to_string()],
                        },
                    };
                }
            }

            create_error_response(input, "Failed to parse AI documentation response")
        }
        _ => create_error_response(input, "AI documentation generation unavailable"),
    }
}

// ===== HELPER FUNCTIONS =====

/// Create an error response
fn create_error_response(input: &AIDocGeneratorRequest, error_msg: &str) -> AIDocGeneratorResponse {
    AIDocGeneratorResponse {
        documentation: format!("# Documentation Generation Error\n\n{}", error_msg),
        content_type: input.content_type.clone(),
        template_used: input.template_type.clone(),
        includes_examples: false,
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

// ===== PARSING HELPERS (Simplified implementations) =====

#[derive(Debug)]
struct FunctionInfo {
    name: String,
    signature: String,
    parameters: Vec<ParameterInfo>,
    return_type: String,
    return_description: String,
}

#[derive(Debug)]
struct ParameterInfo {
    name: String,
    description: String,
}

#[derive(Debug)]
struct EndpointInfo {
    method: String,
    path: String,
    description: String,
    request_body: String,
    response_body: String,
}

/// Extract basic function information (simplified parser)
fn extract_rust_function_info(source: &str) -> Option<FunctionInfo> {
    // Simplified function parsing - in real implementation would use a proper Rust parser
    if source.contains("fn ") {
        Some(FunctionInfo {
            name: "function_name".to_string(),
            signature: "fn example_function() -> Result<String, Error>".to_string(),
            parameters: vec![],
            return_type: "Result<String, Error>".to_string(),
            return_description: "Operation result".to_string(),
        })
    } else {
        None
    }
}

/// Extract API endpoint information (simplified parser)
fn extract_api_endpoint_info(_source: &str) -> Option<EndpointInfo> {
    // Simplified API parsing
    Some(EndpointInfo {
        method: "GET".to_string(),
        path: "/api/example".to_string(),
        description: "Example API endpoint".to_string(),
        request_body: "{}".to_string(),
        response_body: r#"{"status": "success"}"#.to_string(),
    })
}

/// Generate function description
fn generate_function_description(info: &FunctionInfo) -> String {
    format!("This function performs [describe what {} does].", info.name)
}

/// Generate usage examples for functions
fn generate_usage_examples(info: &FunctionInfo) -> String {
    format!(
        "```rust\n// Example usage of {}\nlet result = {}();\n```\n",
        info.name, info.name
    )
}

/// Generate API usage examples
fn generate_api_examples(info: &EndpointInfo) -> String {
    format!("```bash\ncurl -X {} {}\n```\n", info.method, info.path)
}
