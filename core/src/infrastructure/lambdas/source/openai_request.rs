//! OpenAI Request Lambda Function - Using #[lambda_fn] attribute macro with environment variables and HTTP
//!
//! This demonstrates:
//! - Using the #[lambda_fn] attribute macro with features = [env, http]
//! - Retrieving OpenAI API key from environment variables
//! - Making HTTP requests to OpenAI API
//! - Automatic JSON schema generation for workflow validation
//! - Pure business logic without any WASM boilerplate

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== LAMBDA BUSINESS LOGIC =====

/// Input structure for the OpenAI request lambda
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenAIRequestInput {
    /// The user prompt/message to send to OpenAI
    pub prompt: String,
}

/// Output structure containing the OpenAI response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenAIRequestOutput {
    /// The response text from OpenAI
    pub response: String,
    /// Whether the request was successful
    pub success: bool,
    /// Error message if request failed
    pub error: Option<String>,
}

// ===== LAMBDA IMPLEMENTATION USING ATTRIBUTE MACRO =====

#[lambda_fn(features = [env, http])]
fn openai_request_lambda(input: OpenAIRequestInput) -> OpenAIRequestOutput {
    // 🎉 FULL IDE AUTOCOMPLETE WORKS HERE! 🎉

    // Get OpenAI API key from environment variables
    let api_key = match get_env("OPENAPI_KEY") {
        Ok(Some(key)) => key,
        Ok(None) => {
            return OpenAIRequestOutput {
                response: String::new(),
                success: false,
                error: Some("OPENAPI_KEY environment variable not found".to_string()),
            };
        }
        Err(e) => {
            return OpenAIRequestOutput {
                response: String::new(),
                success: false,
                error: Some(format!("Error retrieving OPENAPI_KEY: {}", e)),
            };
        }
    };

    // Prepare the OpenAI API request
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));

    // Create the request body for OpenAI Chat Completions API
    let request_body = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": input.prompt
            }
        ],
        "max_tokens": 1000,
        "temperature": 0.7
    });

    let request = Request {
        method: "POST".to_string(),
        url: "https://api.openai.com/v1/chat/completions".to_string(),
        headers,
        body: Some(request_body.to_string()),
    };

    // Make the HTTP request to OpenAI
    match http_request(&request) {
        Ok(response) if response.is_success() => {
            // Parse the OpenAI response
            match response.json::<serde_json::Value>() {
                Ok(json_response) => {
                    // Extract the content from OpenAI's response structure
                    if let Some(choices) = json_response.get("choices") {
                        if let Some(first_choice) = choices.get(0) {
                            if let Some(message) = first_choice.get("message") {
                                if let Some(content) = message.get("content") {
                                    if let Some(content_str) = content.as_str() {
                                        return OpenAIRequestOutput {
                                            response: content_str.to_string(),
                                            success: true,
                                            error: None,
                                        };
                                    }
                                }
                            }
                        }
                    }
                    
                    // If we can't extract the content, return the raw response
                    OpenAIRequestOutput {
                        response: response.text().to_string(),
                        success: true,
                        error: None,
                    }
                }
                Err(e) => OpenAIRequestOutput {
                    response: String::new(),
                    success: false,
                    error: Some(format!("Failed to parse OpenAI response: {}", e)),
                },
            }
        }
        Ok(response) => OpenAIRequestOutput {
            response: String::new(),
            success: false,
            error: Some(format!("OpenAI API request failed with status: {}", response.status())),
        },
        Err(e) => OpenAIRequestOutput {
            response: String::new(),
            success: false,
            error: Some(format!("HTTP request error: {}", e)),
        },
    }
}

// ===== ATTRIBUTE MACRO USAGE NOTES =====
//
// This lambda uses the #[lambda_fn(features = [env, http])] attribute macro.
// The macro automatically generates:
// - get_env() function for easy environment variable access
// - Request/Response structs with JsonSchema derives
// - http_request() function for easy HTTP calls
// - All WASM memory management and host function bindings
// - JSON schema export functions for workflow validation
//
// 🎉 BENEFITS of the attribute macro:
// ✅ FULL IDE AUTOCOMPLETE: Your function body has complete IDE support!
// ✅ No boilerplate: Just business logic
// ✅ Type safety: Input/Output are strongly typed
// ✅ Schema generation: Automatic workflow validation support
// ✅ Error handling: Built-in environment variable and HTTP access
// ✅ Environment access: Full environment variable functionality
// ✅ HTTP support: Full HTTP client functionality
// ✅ Testability: You can call openai_request_lambda() directly for testing!
// ✅ Clean syntax: Looks like a normal Rust function
//
// Usage examples:
//
// - Test the lambda directly:
//   let test_input = OpenAIRequestInput { 
//       prompt: "Hello, how are you?".to_string() 
//   };
//   let result = openai_request_lambda(test_input);
//   println!("Response: {}", result.response);
//
// - Environment variable access:
//   match get_env("OPENAI_API_KEY") {
//       Ok(Some(key)) => println!("Found API key"),
//       Ok(None) => println!("API key not found"),
//       Err(e) => println!("Error: {}", e),
//   }
//
// - HTTP request with headers:
//   let mut headers = HashMap::new();
//   headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
//   let request = Request {
//       method: "POST".to_string(),
//       url: "https://api.openai.com/v1/chat/completions".to_string(),
//       headers,
//       body: Some(json_body),
//   };
//   let response = http_request(&request)?;
