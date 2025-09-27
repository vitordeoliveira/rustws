//! Get OpenAPI Key Lambda Function - Using #[lambda_fn] attribute macro with environment variable access
//!
//! This demonstrates:
//! - Using the #[lambda_fn] attribute macro with environment variable access
//! - Retrieving environment variables from the host environment
//! - Automatic JSON schema generation for workflow validation
//! - Pure business logic without any WASM boilerplate

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== LAMBDA BUSINESS LOGIC =====

/// Input structure for the lambda (minimal since we just need to trigger it)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetOpenApiKeyInput {
    /// Optional message to include in response
    pub message: Option<String>,
}

/// Output structure containing the OpenAPI key and status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetOpenApiKeyOutput {
    /// The OpenAPI key value (None if not found)
    pub openapi_key: Option<String>,
    /// Whether the key was found
    pub found: bool,
    /// Status message
    pub status: String,
    /// Optional user message
    pub user_message: Option<String>,
}

// ===== LAMBDA IMPLEMENTATION USING ATTRIBUTE MACRO =====

#[lambda_fn(features = [env])]
fn get_openapi_key_lambda(input: GetOpenApiKeyInput) -> GetOpenApiKeyOutput {
    // 🎉 FULL IDE AUTOCOMPLETE WORKS HERE! 🎉

    // Retrieve the OPENAPI_KEY environment variable - IDE knows input type!
    match get_env("OPENAPI_KEY") {
        Ok(Some(key)) => GetOpenApiKeyOutput {
            openapi_key: Some(key),
            found: true,
            status: "Successfully retrieved OpenAPI key".to_string(),
            user_message: input.message, // IDE autocompletes this!
        },
        Ok(None) => GetOpenApiKeyOutput {
            openapi_key: None,
            found: false,
            status: "OpenAPI key not found in environment variables".to_string(),
            user_message: input.message,
        },
        Err(e) => GetOpenApiKeyOutput {
            openapi_key: None,
            found: false,
            status: format!("Error retrieving OpenAPI key: {}", e),
            user_message: input.message,
        },
    }
}

// ===== ATTRIBUTE MACRO USAGE NOTES =====
//
// This lambda uses the new #[lambda_fn(features = [env])] attribute macro.
// The macro automatically generates:
// - get_env() function for easy environment variable access
// - All WASM memory management and host function bindings
// - JSON schema export functions for workflow validation
//
// 🎉 NEW BENEFITS of the attribute macro:
// ✅ FULL IDE AUTOCOMPLETE: Your function body has complete IDE support!
// ✅ No boilerplate: Just business logic
// ✅ Type safety: Input/Output are strongly typed
// ✅ Schema generation: Automatic workflow validation support
// ✅ Error handling: Built-in environment variable access
// ✅ Environment access: Full environment variable functionality
// ✅ Testability: You can call get_openapi_key_lambda() directly for testing!
// ✅ Clean syntax: Looks like a normal Rust function
//
// Comparison:
// OLD: lambda_fn! { features: [env], handler: |input: Type| -> Type { ... } }
// NEW: #[lambda_fn(features = [env])] fn my_lambda(input: Type) -> Type { ... }
//
// Usage examples:
//
// - Retrieve environment variable:
//   match get_env("MY_VAR") {
//       Ok(Some(value)) => println!("Found: {}", value),
//       Ok(None) => println!("Variable not found"),
//       Err(e) => println!("Error: {}", e),
//   }
//
// - With default value:
//   let value = get_env("MY_VAR")
//       .unwrap_or_else(|_| None)
//       .unwrap_or_else(|| "default".to_string());
//
// - Testing the lambda directly:
//   let test_input = GetOpenApiKeyInput { message: Some("test".to_string()) };
//   let result = get_openapi_key_lambda(test_input);  // Can call directly!
//   assert_eq!(result.status.contains("Successfully"), true);
//
// - Check if variable exists:
//   let exists = get_env("MY_VAR")
//       .map(|opt| opt.is_some())
//       .unwrap_or(false);
