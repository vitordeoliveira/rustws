//! Hello World Lambda Function - Using the #[lambda_fn] attribute macro
//!
//! This demonstrates the new simplified approach using the #[lambda_fn] attribute macro.
//! No WASM boilerplate needed - just pure Rust business logic!

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== DEFINE YOUR DATA STRUCTURES =====

/// Input structure for Hello World
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HelloWorld {
    pub text: String,
    pub count: isize,
}

// ===== DEFINE YOUR LAMBDA WITH THE MACRO =====

#[lambda_fn(features = [http])]
fn hello_world_lambda(input: HelloWorld) -> HelloWorld {
    // Validation logic
    if input.count > 50 {
        panic!("Count is too high");
    }

    // Simple business logic - pure Rust!
    HelloWorld {
        text: format!("Hello from WASM! You said: {}", input.text),
        count: input.count + 1,
    }
}
