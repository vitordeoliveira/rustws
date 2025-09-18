//! Lambda Function Macro Crate
//!
//! This crate provides the `lambda_fn!` macro that eliminates all WASM infrastructure
//! code, allowing developers to focus purely on business logic.
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
//! lambda_fn! {
//!     handler: |input: MyInput| -> MyOutput {
//!         MyOutput {
//!             result: format!("Processed: {}", input.message),
//!             new_count: input.count + 1,
//!         }
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
//! lambda_fn! {
//!     features: [http],
//!     handler: |input: ApiInput| -> ApiOutput {
//!         let request = Request {
//!             method: "GET".to_string(),
//!             url: input.endpoint,
//!             headers: HashMap::new(),
//!             body: None,
//!         };
//!         
//!         match http_request(&request) {
//!             Ok(response) => ApiOutput {
//!                 response: response.text().to_string(),
//!                 status: response.status(),
//!             },
//!             Err(e) => ApiOutput {
//!                 response: format!("Error: {}", e),
//!                 status: 500,
//!             },
//!         }
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
//! lambda_fn! {
//!     features: [env],
//!     handler: |input: ConfigInput| -> ConfigOutput {
//!         match get_env(&input.config_key) {
//!             Ok(Some(value)) => ConfigOutput {
//!                 value: Some(value),
//!                 found: true,
//!             },
//!             Ok(None) => ConfigOutput {
//!                 value: None,
//!                 found: false,
//!             },
//!             Err(e) => ConfigOutput {
//!                 value: Some(format!("Error: {}", e)),
//!                 found: false,
//!             },
//!         }
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
//! lambda_fn! {
//!     features: [env, http],
//!     handler: |input: ApiConfigInput| -> ApiConfigOutput {
//!         // Get API key from environment
//!         let api_key = match get_env(&input.api_key_env) {
//!             Ok(Some(key)) => key,
//!             Ok(None) => return ApiConfigOutput {
//!                 response: "API key not found".to_string(),
//!                 status: 401,
//!             },
//!             Err(e) => return ApiConfigOutput {
//!                 response: format!("Error getting API key: {}", e),
//!                 status: 500,
//!             },
//!         };
//!         
//!         // Make HTTP request with API key
//!         let mut headers = HashMap::new();
//!         headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
//!         
//!         let request = Request {
//!             method: "GET".to_string(),
//!             url: input.endpoint,
//!             headers,
//!             body: None,
//!         };
//!         
//!         match http_request(&request) {
//!             Ok(response) => ApiConfigOutput {
//!                 response: response.text().to_string(),
//!                 status: response.status(),
//!             },
//!             Err(e) => ApiConfigOutput {
//!                 response: format!("HTTP error: {}", e),
//!                 status: 500,
//!             },
//!         }
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ExprClosure, Ident, Token};

/// Macro to create a complete lambda function with WASM infrastructure, schema generation, and optional features
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
/// Users only need to provide a handler function with typed input/output parameters.
/// Input and output types are automatically inferred from the handler function signature.
/// The generated schema functions enable workflow validation and type checking.
/// Features are enabled by specifying them in the `features` array: `features: [env, http]`
#[proc_macro]
pub fn lambda_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LambdaFnInput);

    let features = input.features;
    let handler = input.handler;

    // Extract input and output types from handler signature
    let input_type = &handler.input;
    let output_type = &handler.output;
    let input_param = &handler.input_param;
    let handler_body = &handler.body;

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
        pub extern "C" fn wasm_malloc(size: usize) -> *mut u8 {
            let mut buf = Vec::with_capacity(size);
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf);
            ptr
        }

        /// Free memory allocated by wasm_malloc
        #[no_mangle]
        pub extern "C" fn wasm_free_impl(ptr: *mut u8, size: usize) {
            unsafe {
                let _ = Vec::from_raw_parts(ptr, 0, size);
            }
        }

        // ===== SCHEMA EXPORT FUNCTIONS =====

        /// Get JSON Schema for input type
        #[no_mangle]
        pub extern "C" fn get_input_schema(ptr_out: *mut u8, max_out_len: usize) -> usize {
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
        pub extern "C" fn get_output_schema(ptr_out: *mut u8, max_out_len: usize) -> usize {
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
        pub extern "C" fn get_lambda_metadata(ptr_out: *mut u8, max_out_len: usize) -> usize {
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
        pub extern "C" fn handler(
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
            let output = lambda_fn_impl(input);

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

        // ===== BUSINESS LOGIC FUNCTION =====

        /// User's business logic - PURE RUST, NO WASM CONCERNS
        fn lambda_fn_impl(#input_param: #input_type) -> #output_type #handler_body
    };

    TokenStream::from(generated_code)
}

// ===== PARSING STRUCTURES =====

struct LambdaFnInput {
    features: Vec<String>,
    handler: HandlerInfo,
}

struct HandlerInfo {
    input_param: Ident,
    input: syn::Type,
    output: syn::Type,
    body: syn::Block,
}

impl syn::parse::Parse for LambdaFnInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut features = Vec::new();
        let mut handler: Option<HandlerInfo> = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if lookahead.peek(syn::Ident) {
                let ident: Ident = input.parse()?;

                if ident == "features" {
                    input.parse::<Token![:]>()?;
                    let features_content;
                    syn::bracketed!(features_content in input);
                    let features_array: syn::punctuated::Punctuated<syn::Ident, Token![,]> =
                        syn::punctuated::Punctuated::parse_separated_nonempty(&features_content)?;

                    for feature in features_array {
                        features.push(feature.to_string());
                    }
                } else if ident == "handler" {
                    input.parse::<Token![:]>()?;
                    let closure: ExprClosure = input.parse()?;

                    // Extract handler information
                    if closure.inputs.len() != 1 {
                        return Err(syn::Error::new_spanned(
                            &closure,
                            "Handler must have exactly one input parameter",
                        ));
                    }

                    let input_param = match &closure.inputs[0] {
                        syn::Pat::Type(pat_type) => match &*pat_type.pat {
                            syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    pat_type,
                                    "Input parameter must be an identifier",
                                ))
                            }
                        },
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &closure.inputs[0],
                                "Input parameter must be typed",
                            ))
                        }
                    };

                    let input_type = match &closure.inputs[0] {
                        syn::Pat::Type(pat_type) => (*pat_type.ty).clone(),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &closure.inputs[0],
                                "Input parameter must be typed",
                            ))
                        }
                    };

                    let output_type = match &closure.output {
                        syn::ReturnType::Type(_, ty) => (**ty).clone(),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &closure.output,
                                "Handler must have explicit return type",
                            ))
                        }
                    };

                    let body = match &*closure.body {
                        syn::Expr::Block(block) => block.block.clone(),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &closure.body,
                                "Handler body must be a block",
                            ))
                        }
                    };

                    handler = Some(HandlerInfo {
                        input_param,
                        input: input_type,
                        output: output_type,
                        body,
                    });
                } else {
                    return Err(syn::Error::new_spanned(ident, "Unknown field"));
                }
            } else {
                return Err(lookahead.error());
            }

            // Parse comma if present
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let handler =
            handler.ok_or_else(|| syn::Error::new(input.span(), "Handler is required"))?;

        Ok(LambdaFnInput { features, handler })
    }
}
