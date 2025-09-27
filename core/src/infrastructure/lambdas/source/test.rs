//! Lambda Function - Using the #[lambda_fn] attribute macro with full IDE support
//!
//! The #[lambda_fn] attribute macro eliminates all WASM boilerplate.
//! Just focus on your business logic with FULL IDE AUTOCOMPLETE!
//! No memory management, serialization, or handler code needed!

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== DEFINE YOUR DATA STRUCTURES =====

/// Input structure - customize for your needs
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MyInput {
    pub message: String,
    pub count: i32,
}

/// Output structure - customize for your needs  
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MyOutput {
    pub result: String,
    pub new_count: i32,
}

// ===== DEFINE YOUR LAMBDA =====

#[lambda_fn]
fn test_lambda(input: MyInput) -> MyOutput {
    // 🎉 FULL IDE AUTOCOMPLETE WORKS HERE! 🎉
    // YOUR BUSINESS LOGIC HERE!
    // Pure Rust with full IDE support - no WASM concerns needed

    // Example validation - IDE knows input is MyInput!
    if input.count < 0 {
        panic!("Count cannot be negative");
    }

    // Example business logic - IDE autocompletes field names!
    MyOutput {
        result: format!("Processed: {}", input.message),
        new_count: input.count + 1,
    }
}
