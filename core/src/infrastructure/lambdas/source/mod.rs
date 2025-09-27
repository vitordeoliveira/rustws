//! Lambda source files - Templates and debugging support
//!
//! These are TEMPLATE files only - each lambda is compiled separately to its own WASM file.
//!
//! FOR DEBUGGING: If you need to debug a lambda function, temporarily uncomment it below:
//! 1. Uncomment the mod line (e.g., "mod hello_world;")
//! 2. Uncomment the pub use line (e.g., "pub use hello_world::*;")
//! 3. Debug your lambda
//! 4. Comment it back out when done
//!
//! This prevents symbol conflicts while allowing debugging when needed.

// ===== MODULES SECTION =====
// NOTE: Lambda source files are TEMPLATES ONLY - they should NOT be included in main compilation
// Each lambda is compiled separately to its own WASM file to avoid symbol conflicts.
// These files are used as examples and for individual WASM compilation.

// mod a; // TODO: Convert to attribute macro
// mod get_openapi_key; // ✅ Using new attribute macro - TEMPLATE ONLY
// mod hello_without_http_request; // TODO: Convert to attribute macro
// mod hello_world; // ✅ Using new attribute macro - TEMPLATE ONLY
// mod openai_request; // ✅ Using new attribute macro - TEMPLATE ONLY
// mod test; // TODO: Convert to attribute macro
// mod alignment_breaker; // ✅ Using new attribute macro - TEMPLATE ONLY
mod alignment_to_robustness_converter; // ✅ Using new attribute macro - TEMPLATE ONLY
// mod judge; // ✅ Using new attribute macro - TEMPLATE ONLY
// ===== RE-EXPORTS SECTION =====
// NOTE: No re-exports needed since these are templates, not part of main binary
// pub use a::*;  // TODO: Convert to attribute macro
// pub use get_openapi_key::*; // ✅ Using new attribute macro - TEMPLATE ONLY
// pub use hello_without_http_request::*;  // TODO: Convert to attribute macro
// pub use hello_world::*; // ✅ Using new attribute macro - TEMPLATE ONLY
// pub use openai_request::*; // ✅ Using new attribute macro - TEMPLATE ONLY
// pub use test::*;  // TODO: Convert to attribute macro
// pub use alignment_breaker::*; // ✅ Using new attribute macro - TEMPLATE ONLY
pub use alignment_to_robustness_converter::*; // ✅ Using new attribute macro - TEMPLATE ONLY
// pub use judge::*; // ✅ Using new attribute macro - TEMPLATE ONLY
