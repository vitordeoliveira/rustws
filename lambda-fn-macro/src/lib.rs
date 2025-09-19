//! Lambda Function Macro Crate
//!
//! This crate provides the `#[lambda_fn]` attribute macro that eliminates all WASM infrastructure
//! code, allowing developers to focus purely on business logic with full IDE support.
//!
//! # Examples
//!
//! ## Basic Lambda (without features)
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use schemars::JsonSchema;
//! use lambda_fn_macro::lambda_fn;
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
//! pub struct MyInput {
//!     pub message: String,
//!     pub count: i32,
//! }
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
//! pub struct MyOutput {
//!     pub result: String,
//!     pub new_count: i32,
//! }
//!
//! #[lambda_fn]
//! fn my_lambda(input: MyInput) -> MyOutput {
//!     // Full IDE autocomplete works here!
//!     MyOutput {
//!         result: format!("Processed: {}", input.message),
//!         new_count: input.count + 1,
//!     }
//! }
//! ```
//!
//! ## HTTP-Enabled Lambda
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use schemars::JsonSchema;
//! use lambda_fn_macro::lambda_fn;
//! use std::collections::HashMap;
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
//! pub struct ApiInput {
//!     pub endpoint: String,
//!     pub data: String,
//! }
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
//! pub struct ApiOutput {
//!     pub response: String,
//!     pub status: u16,
//! }
//!
//! #[lambda_fn(features = [http])]
//! fn api_lambda(input: ApiInput) -> ApiOutput {
//!     // Full IDE autocomplete + HTTP functions available!
//!     let request = Request {
//!         method: "GET".to_string(),
//!         url: input.endpoint,
//!         headers: HashMap::new(),
//!         body: None,
//!     };
//!     
//!     match http_request(&request) {
//!         Ok(response) => ApiOutput {
//!             response: response.text().to_string(),
//!             status: response.status(),
//!         },
//!         Err(e) => ApiOutput {
//!             response: format!("Error: {}", e),
//!             status: 500,
//!         },
//!     }
//! }
//! ```
//!
//! ## Lambda with Environment Variables
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use schemars::JsonSchema;
//! use lambda_fn_macro::lambda_fn;
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
//! pub struct ConfigInput {
//!     pub config_key: String,
//! }
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
//! pub struct ConfigOutput {
//!     pub value: Option<String>,
//!     pub found: bool,
//! }
//!
//! #[lambda_fn(features = [env])]
//! fn config_lambda(input: ConfigInput) -> ConfigOutput {
//!     // Full IDE autocomplete + environment functions available!
//!     match get_env(&input.config_key) {
//!         Ok(Some(value)) => ConfigOutput {
//!             value: Some(value),
//!             found: true,
//!         },
//!         Ok(None) => ConfigOutput {
//!             value: None,
//!             found: false,
//!         },
//!         Err(e) => ConfigOutput {
//!             value: Some(format!("Error: {}", e)),
//!             found: false,
//!         },
//!     }
//! }
//! ```
//!
//! ## Lambda with Multiple Features
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use schemars::JsonSchema;
//! use lambda_fn_macro::lambda_fn;
//! use std::collections::HashMap;
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
//! pub struct ApiConfigInput {
//!     pub endpoint: String,
//!     pub api_key_env: String,
//! }
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
//! pub struct ApiConfigOutput {
//!     pub response: String,
//!     pub status: u16,
//! }
//!
//! #[lambda_fn(features = [env, http])]
//! fn full_lambda(input: ApiConfigInput) -> ApiConfigOutput {
//!     // Full IDE autocomplete + env + HTTP functions available!
//!     // Get API key from environment
//!     let api_key = match get_env(&input.api_key_env) {
//!         Ok(Some(key)) => key,
//!         Ok(None) => return ApiConfigOutput {
//!             response: "API key not found".to_string(),
//!             status: 401,
//!         },
//!         Err(e) => return ApiConfigOutput {
//!             response: format!("Error getting API key: {}", e),
//!             status: 500,
//!         },
//!     };
//!     
//!     // Make HTTP request with API key
//!     let mut headers = HashMap::new();
//!     headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
//!     
//!     let request = Request {
//!         method: "GET".to_string(),
//!         url: input.endpoint,
//!         headers,
//!         body: None,
//!     };
//!     
//!     match http_request(&request) {
//!         Ok(response) => ApiConfigOutput {
//!             response: response.text().to_string(),
//!             status: response.status(),
//!         },
//!         Err(e) => ApiConfigOutput {
//!             response: format!("HTTP error: {}", e),
//!             status: 500,
//!         },
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType, Type};

/// Attribute macro to create a complete lambda function with WASM infrastructure, schema generation, and optional features
///
/// This macro generates all the necessary WASM boilerplate including:
/// - Memory management functions (`wasm_malloc`, `wasm_free_impl`)
/// - Handler function with pointer-based interface
/// - Serialization/deserialization logic
/// - Error handling
/// - JSON Schema export functions (`get_input_schema`, `get_output_schema`)
/// - Lambda metadata export function (`get_lambda_metadata`)
/// - Optional features based on `features` array:
///   - `env`: Environment variable access (`get_env`) for retrieving host environment variables
///   - `http`: HTTP functionality (`Request`, `Response`, `http_request`) for external API calls
///
/// The function signature is preserved and can be tested directly.
/// Input and output types are automatically inferred from the function signature.
/// The generated schema functions enable workflow validation and type checking.
/// Features are enabled by specifying them in the attribute: `#[lambda_fn(features = [env, http])]`
///
/// ## Usage
///
/// ### Basic usage (no features):
/// ```rust
/// #[lambda_fn]
/// fn my_lambda(input: MyInput) -> MyOutput {
///     // Your business logic here with full IDE support!
/// }
/// ```
///
/// ### With features:
/// ```rust
/// #[lambda_fn(features = [http, env])]
/// fn my_lambda(input: MyInput) -> MyOutput {
///     // HTTP and environment functions available
/// }
/// ```
#[proc_macro_attribute]
pub fn lambda_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // Parse features from attribute
    let features = if attr.is_empty() {
        Vec::new()
    } else {
        parse_features(attr)
    };

    // Extract function information
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;

    // Use simple function names for easier metadata extraction
    let input_schema_fn = syn::Ident::new("get_input_schema", fn_name.span());
    let output_schema_fn = syn::Ident::new("get_output_schema", fn_name.span());
    let metadata_fn = syn::Ident::new("get_lambda_metadata", fn_name.span());
    let handler_fn = syn::Ident::new("handler", fn_name.span());
    let wasm_malloc_fn = syn::Ident::new("wasm_malloc", fn_name.span());
    let wasm_free_impl_fn = syn::Ident::new("wasm_free_impl", fn_name.span());

    // Extract input and output types from function signature
    let (input_param, input_type) = extract_input_info(&input_fn.sig.inputs);
    let output_type = extract_output_type(&input_fn.sig.output);

    // Check which features are enabled
    let has_http = features.contains(&"http".to_string());
    let has_env = features.contains(&"env".to_string());

    // Generate conditional code based on features
    let http_extern = if has_http {
        quote! {
            /// Host function for making HTTP requests from WASM
            fn host_http_request(
                req_ptr: *const u8,
                req_len: usize,
                out_ptr: *mut *mut u8,
                out_len: *mut usize,
            ) -> i32;
        }
    } else {
        quote! {}
    };

    let env_extern = if has_env {
        quote! {
            /// Host function to get environment variable from WASM
            fn host_get_env(
                key_ptr: *const u8,
                key_len: usize,
                out_ptr: *mut *mut u8,
                out_len: *mut usize,
            ) -> i32;
        }
    } else {
        quote! {}
    };

    let http_impl = if has_http {
        quote! {
            /// Simple HTTP request for WASM ↔ Host communication (auto-generated)
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
            pub struct Request {
                pub method: String,
                pub url: String,
                pub headers: std::collections::HashMap<String, String>,
                pub body: Option<String>,
            }

            /// Simple HTTP response for WASM ↔ Host communication (auto-generated)
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
            pub struct Response {
                pub status: u16,
                pub body: String,
            }

            impl Response {
                /// Deserialize response body as JSON
                pub fn json<T: for<'de> serde::Deserialize<'de>>(&self) -> Result<T, String> {
                    serde_json::from_str(&self.body).map_err(|e| format!("Failed to deserialize JSON: {}", e))
                }

                /// Get response body as text
                pub fn text(&self) -> &str {
                    &self.body
                }

                /// Check if response was successful (2xx status)
                pub fn is_success(&self) -> bool {
                    (200..300).contains(&self.status)
                }

                /// Get status code
                pub fn status(&self) -> u16 {
                    self.status
                }
            }

            /// Send HTTP request to host (auto-generated)
            pub fn http_request(request: &Request) -> Result<Response, String> {
                // Serialize the request to JSON
                let request_json =
                    serde_json::to_vec(request).map_err(|e| format!("Failed to serialize request: {}", e))?;

                // Prepare variables to receive response pointer and length
                let mut out_ptr: u32 = 0;
                let mut out_len: u32 = 0;

                // Call the host function with pointers to our variables
                let result = unsafe {
                    host_http_request(
                        request_json.as_ptr(),
                        request_json.len(),
                        &mut out_ptr as *mut u32 as *mut *mut u8,
                        &mut out_len as *mut u32 as *mut usize,
                    )
                };

                if result != 0 {
                    return Err(format!("HTTP request failed with code: {}", result));
                }

                if out_ptr == 0 || out_len == 0 {
                    return Err("Empty response from host".to_string());
                }

                // Read the response from the host-allocated memory
                let response_bytes =
                    unsafe { std::slice::from_raw_parts(out_ptr as *const u8, out_len as usize) };

                // Deserialize the response
                let response: Response = serde_json::from_slice(response_bytes)
                    .map_err(|e| format!("Failed to deserialize response: {}", e))?;

                Ok(response)
            }
        }
    } else {
        quote! {}
    };

    let env_impl = if has_env {
        quote! {
            /// Get environment variable from host (auto-generated)
            pub fn get_env(key: &str) -> Result<Option<String>, String> {
                // Convert key to bytes
                let key_bytes = key.as_bytes();

                // Prepare variables to receive response pointer and length
                let mut out_ptr: u32 = 0;
                let mut out_len: u32 = 0;

                // Call the host function with pointers to our variables
                let result = unsafe {
                    host_get_env(
                        key_bytes.as_ptr(),
                        key_bytes.len(),
                        &mut out_ptr as *mut u32 as *mut *mut u8,
                        &mut out_len as *mut u32 as *mut usize,
                    )
                };

                if result != 0 {
                    return Err(format!("Failed to get environment variable '{}': {}", key, result));
                }

                if out_ptr == 0 || out_len == 0 {
                    // Environment variable not found
                    return Ok(None);
                }

                // Read the response from the host-allocated memory
                let response_bytes =
                    unsafe { std::slice::from_raw_parts(out_ptr as *const u8, out_len as usize) };

                // Deserialize the response
                let value: Option<String> = serde_json::from_slice(response_bytes)
                    .map_err(|e| format!("Failed to deserialize environment variable '{}': {}", key, e))?;

                Ok(value)
            }
        }
    } else {
        quote! {}
    };

    let generated_code = quote! {
        // ===== ORIGINAL FUNCTION (PRESERVED FOR TESTING) =====

        /// Original user function - can be called directly for testing
        #fn_vis fn #fn_name(#input_param: #input_type) -> #output_type #fn_block

        // ===== HOST FUNCTIONALITY (AUTO-GENERATED) =====

        extern "C" {
            #http_extern
            #env_extern

            /// Host function to free memory allocated by the host
            fn wasm_free(ptr: *mut u8, size: usize);
        }

        #http_impl
        #env_impl

        // ===== WASM MEMORY MANAGEMENT =====

        /// Allocate memory in WASM that can be accessed by the host
        #[no_mangle]
        pub extern "C" fn #wasm_malloc_fn(size: usize) -> *mut u8 {
            let mut buf = Vec::with_capacity(size);
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf);
            ptr
        }

        /// Free memory allocated by wasm_malloc
        #[no_mangle]
        pub extern "C" fn #wasm_free_impl_fn(ptr: *mut u8, size: usize) {
            unsafe {
                let _ = Vec::from_raw_parts(ptr, 0, size);
            }
        }

        // ===== SCHEMA EXPORT FUNCTIONS =====

        /// Get JSON Schema for input type
        #[no_mangle]
        pub extern "C" fn #input_schema_fn(ptr_out: *mut u8, max_out_len: usize) -> usize {
            use schemars::{schema_for, JsonSchema};

            // Generate schema for input type
            let schema = schema_for!(#input_type);
            let schema_json = match serde_json::to_string_pretty(&schema) {
                Ok(json) => json,
                Err(_) => return 0, // Return 0 on error
            };

            let schema_bytes = schema_json.as_bytes();
            let copy_len = std::cmp::min(max_out_len, schema_bytes.len());

            unsafe {
                std::ptr::copy_nonoverlapping(schema_bytes.as_ptr(), ptr_out, copy_len);
            }

            schema_bytes.len() // Return actual length needed
        }

        /// Get JSON Schema for output type
        #[no_mangle]
        pub extern "C" fn #output_schema_fn(ptr_out: *mut u8, max_out_len: usize) -> usize {
            use schemars::{schema_for, JsonSchema};

            // Generate schema for output type
            let schema = schema_for!(#output_type);
            let schema_json = match serde_json::to_string_pretty(&schema) {
                Ok(json) => json,
                Err(_) => return 0, // Return 0 on error
            };

            let schema_bytes = schema_json.as_bytes();
            let copy_len = std::cmp::min(max_out_len, schema_bytes.len());

            unsafe {
                std::ptr::copy_nonoverlapping(schema_bytes.as_ptr(), ptr_out, copy_len);
            }

            schema_bytes.len() // Return actual length needed
        }

        /// Get metadata as JSON (features, input/output types, macro version)
        #[no_mangle]
        pub extern "C" fn #metadata_fn(ptr_out: *mut u8, max_out_len: usize) -> usize {
            let metadata = serde_json::json!({
                "features": {
                    "http_enabled": #has_http,
                    "env_enabled": #has_env
                },
                "input_type": stringify!(#input_type),
                "output_type": stringify!(#output_type),
                "macro_version": env!("CARGO_PKG_VERSION")
            });

            let metadata_json = match serde_json::to_string_pretty(&metadata) {
                Ok(json) => json,
                Err(_) => return 0,
            };

            let metadata_bytes = metadata_json.as_bytes();
            let copy_len = std::cmp::min(max_out_len, metadata_bytes.len());

            unsafe {
                std::ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), ptr_out, copy_len);
            }

            metadata_bytes.len()
        }

        // ===== MAIN HANDLER FUNCTION =====

        /// Main handler function - GENERATED BY MACRO
        /// Handles all WASM interface complexity (pointers, serialization, etc.)
        #[no_mangle]
        pub extern "C" fn #handler_fn(
            ptr_in: *const u8,
            len_in: usize,
            ptr_out: *mut u8,
            max_out_len: usize,
        ) -> usize {
            // Deserialize input from WASM memory
            let input_slice = unsafe { std::slice::from_raw_parts(ptr_in, len_in) };
            let input: #input_type = match serde_json::from_slice(input_slice) {
                Ok(input) => input,
                Err(e) => {
                    // Create error response and serialize it
                    let error_msg = format!("Failed to deserialize input: {}", e);
                    let error_bytes = error_msg.as_bytes();
                    let copy_len = std::cmp::min(max_out_len, error_bytes.len());
                    unsafe {
                        std::ptr::copy_nonoverlapping(error_bytes.as_ptr(), ptr_out, copy_len);
                    }
                    return copy_len;
                }
            };

            // Call user's business logic function
            let output = #fn_name(input);

            // Serialize output to JSON
            let output_bytes = match serde_json::to_vec(&output) {
                Ok(bytes) => bytes,
                Err(e) => {
                    // Create error response and serialize it
                    let error_msg = format!("Failed to serialize output: {}", e);
                    let error_bytes = error_msg.as_bytes();
                    let copy_len = std::cmp::min(max_out_len, error_bytes.len());
                    unsafe {
                        std::ptr::copy_nonoverlapping(error_bytes.as_ptr(), ptr_out, copy_len);
                    }
                    return copy_len;
                }
            };

            // Copy output to WASM memory
            let copy_len = std::cmp::min(max_out_len, output_bytes.len());
            unsafe {
                std::ptr::copy_nonoverlapping(output_bytes.as_ptr(), ptr_out, copy_len);
            }
            copy_len
        }
    };

    TokenStream::from(generated_code)
}

// ===== HELPER FUNCTIONS =====

fn parse_features(attr: TokenStream) -> Vec<String> {
    if attr.is_empty() {
        return Vec::new();
    }

    // Simple string-based parsing for features = [http, env]
    let attr_str = attr.to_string();
    let mut features = Vec::new();

    // Look for "features = [...]" pattern
    if let Some(features_start) = attr_str.find("features") {
        let remaining = &attr_str[features_start..];
        if let Some(bracket_start) = remaining.find('[') {
            if let Some(bracket_end) = remaining.find(']') {
                let features_content = &remaining[bracket_start + 1..bracket_end];
                for feature in features_content.split(',') {
                    let feature = feature.trim();
                    if !feature.is_empty() {
                        features.push(feature.to_string());
                    }
                }
            }
        }
    }

    features
}

fn extract_input_info(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> (syn::Ident, &Type) {
    // Extract the first parameter (input parameter)
    if let Some(FnArg::Typed(pat_type)) = inputs.first() {
        if let Pat::Ident(pat_ident) = &*pat_type.pat {
            return (pat_ident.ident.clone(), &*pat_type.ty);
        }
    }
    panic!("Lambda function must have exactly one typed parameter");
}

fn extract_output_type(output: &ReturnType) -> &Type {
    if let ReturnType::Type(_, ty) = output {
        return &**ty;
    }
    panic!("Lambda function must have explicit return type");
}
