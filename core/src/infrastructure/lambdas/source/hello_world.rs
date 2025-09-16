//! Hello World Lambda Function - Rust Example
//!
//! This is a lambda function that demonstrates:
//! - Structured data return with serialization
//! - JSON-based communication
//! - Memory allocation for complex types in WASM
//! - HTTP requests from WASM to external APIs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== WASM HOST FUNCTION DECLARATIONS =====

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

// ===== SIMPLE HTTP TYPES =====

/// Simple HTTP request for WASM ↔ Host communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Simple HTTP response for WASM ↔ Host communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

/// Send HTTP request to host
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

// ===== LAMBDA BUSINESS LOGIC =====

/// HelloWorld response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloWorld {
    pub text: String,
    pub count: isize,
}

/// Example JSONPlaceholder post structure for HTTP demo
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Post {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub userId: u32,
}

/// Main handler function for the lambda
/// Uses pointer-based memory management for better performance
#[no_mangle]
pub extern "C" fn handler(
    ptr_in: *const u8,
    len_in: usize,
    ptr_out: *mut u8,
    max_out_len: usize,
) -> usize {
    let input_slice = unsafe { std::slice::from_raw_parts(ptr_in, len_in) };
    let input: HelloWorld = serde_json::from_slice(input_slice).unwrap();

    let output = lambda_fn(input);
    let output_bytes = serde_json::to_vec(&output).unwrap();

    let copy_len = std::cmp::min(max_out_len, output_bytes.len());
    unsafe {
        std::ptr::copy_nonoverlapping(output_bytes.as_ptr(), ptr_out, copy_len);
    }
    copy_len
}

/// Business logic function demonstrating HTTP requests
/// Takes HelloWorld input and returns HelloWorld output with HTTP data
fn lambda_fn(input: HelloWorld) -> HelloWorld {
    if input.count > 50 {
        panic!("Count is too high");
    }

    // Simple HTTP request using basic Request struct
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "rustws-lambda/1.0".to_string());

    let request = Request {
        method: "GET".to_string(),
        url: "https://jsonplaceholder.typicode.com/posts/1".to_string(),
        headers,
        body: None,
    };

    match http_request(&request) {
        Ok(response) => {
            if response.status == 200 {
                // Try to parse the JSONPlaceholder API response
                match serde_json::from_str::<Post>(&response.body) {
                    Ok(post) => HelloWorld {
                        text: format!(
                            "Processed: {} (original count: {}). Got post '{}' by user {}!",
                            input.text, input.count, post.title, post.userId
                        ),
                        count: input.count + 10,
                    },
                    Err(_) => HelloWorld {
                        text: format!(
                            "Processed: {} (original count: {}). Got HTTP response but couldn't parse JSON",
                            input.text, input.count
                        ),
                        count: input.count + 10,
                    },
                }
            } else {
                HelloWorld {
                    text: format!(
                        "Processed: {} (original count: {}). HTTP request failed with status: {}",
                        input.text, input.count, response.status
                    ),
                    count: input.count + 10,
                }
            }
        }
        Err(e) => HelloWorld {
            text: format!(
                "Processed: {} (original count: {}). HTTP request error: {}",
                input.text, input.count, e
            ),
            count: input.count + 10,
        },
    }
}

// Compilation instructions:
// This file is compiled using cargo with proper dependency management
// The system automatically creates a temporary Cargo project with:
// - serde = { version = "1.0", features = ["derive"] }
// - serde_json = "1.0"
//
// Usage examples:
// - Simple GET:
//   let request = Request {
//       method: "GET".to_string(),
//       url: "https://api.example.com/data".to_string(),
//       headers: HashMap::new(),
//       body: None,
//   };
//   let response = http_request(&request)?;
//
// - POST with JSON body:
//   let mut headers = HashMap::new();
//   headers.insert("Content-Type".to_string(), "application/json".to_string());
//   let request = Request {
//       method: "POST".to_string(),
//       url: "https://api.example.com/create".to_string(),
//       headers,
//       body: Some(serde_json::to_string(&my_data)?),
//   };
//   let response = http_request(&request)?;
