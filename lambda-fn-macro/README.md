# Lambda Function Macro

A Rust macro that eliminates WASM boilerplate for lambda functions, allowing developers to focus purely on business logic.

## Overview

The `lambda_fn!` macro generates all necessary WASM infrastructure code including:

- Memory management functions (`wasm_malloc`, `wasm_free_impl`)
- Handler function with pointer-based interface
- JSON serialization/deserialization
- Error handling

## Usage

```rust
use serde::{Deserialize, Serialize};
use lambda_fn_macro::lambda_fn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyInput {
    pub message: String,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyOutput {
    pub result: String,
    pub new_count: i32,
}

lambda_fn! {
    input: MyInput,
    output: MyOutput,
    handler: |input: MyInput| -> MyOutput {
        // Pure business logic - no WASM concerns!
        MyOutput {
            result: format!("Processed: {}", input.message),
            new_count: input.count + 1,
        }
    }
}
```

## Benefits

- **Zero WASM knowledge required**: Focus on business logic only
- **Type safety**: Compile-time validation of input/output types
- **Error handling**: Automatic serialization error management
- **Performance**: Generates efficient WASM code
- **Maintainability**: Single source of truth for the macro

## Generated Code

The macro generates:

1. **Memory Management**:

   ```rust
   #[no_mangle]
   pub extern "C" fn wasm_malloc(size: usize) -> *mut u8 { ... }

   #[no_mangle]
   pub extern "C" fn wasm_free_impl(ptr: *mut u8, size: usize) { ... }
   ```

2. **Handler Function**:

   ```rust
   #[no_mangle]
   pub extern "C" fn handler(
       ptr_in: *const u8,
       len_in: usize,
       ptr_out: *mut u8,
       max_out_len: usize,
   ) -> usize { ... }
   ```

3. **Business Logic Wrapper**:
   ```rust
   fn lambda_fn_impl(input: YourInputType) -> YourOutputType {
       // Your code goes here
   }
   ```

## Dependencies

- `serde` with `derive` feature for serialization
- `serde_json` for JSON handling

## Integration

This crate is automatically included as a dependency when compiling lambdas in the rustws system. The compilation process:

1. Creates temporary Cargo project
2. Includes `lambda-fn-macro` as path dependency
3. Processes user source to fix outer doc comments
4. Compiles to WASM with all infrastructure generated

## Developer Experience

**Before (Manual WASM)**:

```rust
// 70+ lines of boilerplate...
#[no_mangle]
pub extern "C" fn wasm_malloc(size: usize) -> *mut u8 { ... }
// ... lots of pointer management ...
fn my_business_logic(input: MyType) -> MyType { ... }
```

**After (Macro)**:

```rust
lambda_fn! {
    input: MyInput,
    output: MyOutput,
    handler: |input| -> MyOutput {
        // Just business logic!
    }
}
```

## Compilation

The macro is included during lambda compilation through the rustws build system. No manual setup required for end users.
