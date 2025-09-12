//! Hello World Lambda Function - Rust Example
//!
//! This is a simple lambda function that demonstrates:
//! - Basic WASM module structure
//! - Function exports for lambda runtime
//! - Simple numeric operations that work well in WASM

/// Main handler function for the lambda
/// This function will be called by the wasmer runtime
#[no_mangle]
pub extern "C" fn handler() -> i32 {
    // Simple computation that returns a meaningful result
    42
}

/// Add two numbers - simple arithmetic
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiply two numbers
#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Fibonacci calculation (recursive)
#[no_mangle]
pub extern "C" fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

/// Simple greeting that returns a constant
#[no_mangle]
pub extern "C" fn get_greeting() -> i32 {
    // Return a meaningful constant that fits in i32
    // Magic number representing "RUSTWS"
    123456789
}

/// Check if number is prime
#[no_mangle]
pub extern "C" fn is_prime(n: i32) -> i32 {
    if n <= 1 {
        return 0; // false
    }
    if n <= 3 {
        return 1; // true
    }
    if n % 2 == 0 || n % 3 == 0 {
        return 0; // false
    }

    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return 0; // false
        }
        i += 6;
    }
    1 // true
}

// Compilation instructions:
// rustc --target wasm32-unknown-unknown -C opt-level=s -C panic=abort --crate-type=cdylib hello_world.rs -o hello_world.wasm
