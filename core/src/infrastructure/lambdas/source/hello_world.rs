//! Hello World Lambda Function - Rust Example
//!
//! This is a lambda function that demonstrates:
//! - Structured data return with serialization
//! - JSON-based communication
//! - Memory allocation for complex types in WASM

use serde::{Deserialize, Serialize};

// TODO: Future WASM Helper Functions
// Consider adding these common WASM patterns if we need them across multiple lambdas:
//
// - deserialize_input<T>(ptr: *const u8, len: usize) -> Result<T, String>
//   Safe JSON deserialization from raw byte slice with error handling
//
// - serialize_and_copy_output<T>(data: &T, ptr_out: *mut u8, max_len: usize) -> Result<usize, String>
//   Safe serialization and buffer copying with bounds checking
//
// - create_error_response(error_msg: &str) -> Vec<u8>
//   Standardized error response format for all lambdas
//
// - validate_buffer_bounds(required_len: usize, max_len: usize) -> bool
//   Memory bounds checking utility
//
// These would reduce code duplication and improve safety across lambda functions.

/// HelloWorld response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloWorld {
    pub text: String,
    pub count: isize,
}

/// Main handler function for the lambda
/// Uses pointer-based memory management for better performance
///
/// For WASM, we:
/// 1. Accept input as byte slice pointer and length
/// 2. Deserialize JSON to HelloWorld struct
/// 3. Call lambda_fn for business logic
/// 4. Serialize result and copy to output buffer
/// 5. Return actual output length
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

/// Business logic function that developers write
/// Takes HelloWorld input and returns HelloWorld output
fn lambda_fn(input: HelloWorld) -> HelloWorld {
    HelloWorld {
        text: format!(
            "Processed: {} (original count: {})",
            input.text, input.count
        ),
        count: input.count + 10,
    }
}

// Compilation instructions:
// This file is now compiled using cargo with proper dependency management
// The system automatically creates a temporary Cargo project with:
// - serde = { version = "1.0", features = ["derive"] }
// - serde_json = "1.0"
//
// Usage with new pointer-based interface:
// - Input: JSON bytes of HelloWorld struct
// - Output: JSON bytes of HelloWorld struct
// - Returns: actual output length written to buffer
