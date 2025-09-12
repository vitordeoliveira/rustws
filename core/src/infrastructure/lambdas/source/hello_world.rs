//! Hello World Lambda Function - Rust Example
//! 
//! This is a simple lambda function that demonstrates:
//! - Basic WASM module structure
//! - Function exports for lambda runtime
//! - Memory management in WASM context

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Main handler function for the lambda
/// This function will be called by the wasmer runtime
#[no_mangle]
pub extern "C" fn handler() -> i32 {
    // Simple hello world logic
    println!("Hello from RUSTWS Lambda!");
    
    // Return success code
    0
}

/// Alternative entry point for compatibility
#[no_mangle]
pub extern "C" fn main() -> i32 {
    handler()
}

/// Process string input (example of memory handling)
#[no_mangle]
pub extern "C" fn process_string(input_ptr: *const c_char) -> *mut c_char {
    if input_ptr.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let input_cstr = CStr::from_ptr(input_ptr);
        if let Ok(input_str) = input_cstr.to_str() {
            let result = format!("Processed: {}", input_str);
            if let Ok(result_cstring) = CString::new(result) {
                return result_cstring.into_raw();
            }
        }
    }

    std::ptr::null_mut()
}

/// Free memory allocated by process_string
#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Get lambda metadata
#[no_mangle]
pub extern "C" fn get_version() -> *const c_char {
    "1.0.0\0".as_ptr() as *const c_char
}

// Compilation instructions:
// rustc --target wasm32-unknown-unknown -O --crate-type=cdylib hello_world.rs -o hello_world.wasm
