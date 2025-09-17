//! Lambda Function Macro Crate
//!
//! This crate provides the `lambda_fn!` macro that eliminates all WASM infrastructure
//! code, allowing developers to focus purely on business logic.
//!
//! # Examples
//!
//! ## Basic Lambda (without HTTP)
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
//!     input: MyInput,
//!     output: MyOutput,
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
//!     input: ApiInput,
//!     output: ApiOutput,
//!     enable_http: true,
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

/// Macro to create a complete lambda function with WASM infrastructure, schema generation, and optional HTTP support
///
/// This macro generates all the necessary WASM boilerplate including:
/// - Memory management functions (`wasm_malloc`, `wasm_free_impl`)
/// - Handler function with pointer-based interface
/// - Serialization/deserialization logic
/// - Error handling
/// - JSON Schema export functions (`get_input_schema`, `get_output_schema`)
/// - Lambda metadata export function (`get_lambda_metadata`)
/// - Optional HTTP functionality (`Request`, `Response`, `http_request`) when `enable_http: true`
///
/// Users only need to define their input/output types (with JsonSchema derive) and business logic.
/// The generated schema functions enable workflow validation and type checking.
/// HTTP functionality provides seamless access to external APIs from within lambda functions.
#[macro_export]
macro_rules! lambda_fn {
    // Enhanced pattern with optional HTTP support
    (
        input: $input_type:ty,
        output: $output_type:ty,
        enable_http: true,
        handler: |$input_param:ident: $input_param_type:ty| -> $output_param_type:ty $handler_body:block
    ) => {
        // ===== HTTP FUNCTIONALITY (AUTO-GENERATED) =====

        extern "C" {
            /// Host function for making HTTP requests from WASM
            fn host_http_request(
                req_ptr: *const u8,
                req_len: usize,
                out_ptr: *mut *mut u8,
                out_len: *mut usize,
            ) -> i32;

            /// Host function to free memory allocated by the host
            fn wasm_free(ptr: *mut u8, size: usize);
        }

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
            let schema = schema_for!($input_type);
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
            let schema = schema_for!($output_type);
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
                    "http_enabled": true
                },
                "input_type": stringify!($input_type),
                "output_type": stringify!($output_type),
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
            let input: $input_type = match serde_json::from_slice(input_slice) {
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
        fn lambda_fn_impl($input_param: $input_param_type) -> $output_param_type $handler_body
    };

    // Original pattern (backward compatibility)
    (
        input: $input_type:ty,
        output: $output_type:ty,
        handler: |$input_param:ident: $input_param_type:ty| -> $output_param_type:ty $handler_body:block
    ) => {
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
            let schema = schema_for!($input_type);
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
            let schema = schema_for!($output_type);
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

        /// Get metadata as JSON (function name, types, features, etc.)
        #[no_mangle]
        pub extern "C" fn get_lambda_metadata(ptr_out: *mut u8, max_out_len: usize) -> usize {
            let metadata = serde_json::json!({
                "features": {
                    "http_enabled": false
                },
                "input_type": stringify!($input_type),
                "output_type": stringify!($output_type),
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
            let input: $input_type = match serde_json::from_slice(input_slice) {
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
        fn lambda_fn_impl($input_param: $input_param_type) -> $output_param_type $handler_body
    };
}
