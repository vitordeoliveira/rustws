//! WASM runtime and execution engine
//!
//! This module provides WASM execution capabilities using wasmer, including
//! host function support for HTTP requests and environment variable access.

use crate::error_handling::types::{AppError, AppResult};
use tracing::instrument;

/// Environment data for WASM functions
#[derive(Clone)]
pub(crate) struct WasmEnv {
    pub memory: Option<wasmer::Memory>,
}

impl WasmEnv {
    pub fn new() -> Self {
        Self { memory: None }
    }

    #[allow(dead_code)]
    fn with_memory(memory: wasmer::Memory) -> Self {
        Self {
            memory: Some(memory),
        }
    }
}

/// Host function for HTTP requests from WASM
#[instrument(skip_all, fields(operation = "host_http_request"))]
fn host_http_request(
    mut env: wasmer::FunctionEnvMut<WasmEnv>,
    req_ptr: u32,
    req_len: u32,
    out_ptr_ptr: u32,
    out_len_ptr: u32,
) -> i32 {
    tracing::debug!("HTTP request from WASM lambda");

    // Get memory from the environment
    let memory = match env.data().memory.clone() {
        Some(mem) => mem,
        None => {
            tracing::error!("Failed to get WASM memory in HTTP host function");
            return -1;
        }
    };

    // Read request from WASM memory
    let request_bytes = {
        let view = memory.view(&env);
        let mut bytes = Vec::with_capacity(req_len as usize);
        for i in 0..req_len {
            match view.read_u8((req_ptr + i) as u64) {
                Ok(byte) => bytes.push(byte),
                Err(e) => {
                    tracing::error!("Failed to read request from WASM memory: {}", e);
                    return -1;
                }
            }
        }
        bytes
    };

    // Parse request JSON
    let request: std::collections::HashMap<String, serde_json::Value> =
        match serde_json::from_slice(&request_bytes) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to parse HTTP request JSON: {}", e);
                return -1;
            }
        };

    // Extract request fields
    let method = request
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET");
    let url = match request.get("url").and_then(|v| v.as_str()) {
        Some(url) => url,
        None => {
            tracing::error!("Request missing URL field");
            return -1;
        }
    };

    tracing::info!(method = %method, url = %url, "Making real HTTP request from lambda");

    // Clone data for the blocking thread
    let method = method.to_string();
    let url = url.to_string();
    let headers = request.get("headers").and_then(|v| v.as_object()).cloned();
    let body = request
        .get("body")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Execute HTTP request in a blocking thread to avoid tokio runtime conflicts
    let http_result = std::thread::spawn(move || {
        // Create HTTP client in the blocking thread
        let client = reqwest::blocking::Client::new();
        let mut req_builder = match method.as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            "PATCH" => client.patch(&url),
            "HEAD" => client.head(&url),
            "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url),
            _ => client.get(&url), // Default to GET
        };

        // Add headers if present
        if let Some(headers) = headers {
            for (key, value) in headers {
                if let Some(value_str) = value.as_str() {
                    req_builder = req_builder.header(key, value_str);
                }
            }
        }

        // Add body if present
        if let Some(body) = body {
            req_builder = req_builder.body(body);
        }

        // Execute HTTP request
        let response = req_builder.send()?;
        let status = response.status().as_u16();
        let body = response.text()?;

        Ok::<(u16, String), reqwest::Error>((status, body))
    });

    let (status, body) = match http_result.join() {
        Ok(Ok((status, body))) => (status, body),
        Ok(Err(e)) => {
            tracing::error!("HTTP request failed: {}", e);
            return -1;
        }
        Err(_) => {
            tracing::error!("HTTP request thread panicked");
            return -1;
        }
    };

    // Create response JSON
    let response_json = serde_json::json!({
        "status": status,
        "body": body
    });

    let response_bytes = match serde_json::to_vec(&response_json) {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to serialize response: {}", e);
            return -1;
        }
    };

    tracing::info!(status = %status, response_size = response_bytes.len(), "HTTP request completed");

    // Use a simple approach: allocate in existing WASM memory and return pointer
    let response_len = response_bytes.len();

    // Get current memory size and use it as allocation point
    let memory_view = memory.view(&env);
    let current_size = memory_view.data_size() as usize;
    let alloc_ptr = current_size as u32;

    // Grow memory if needed
    let needed_pages = ((current_size + response_len + 65535) / 65536) - (current_size / 65536);
    if needed_pages > 0 {
        if let Err(e) = memory.grow(&mut env, needed_pages as u32) {
            tracing::error!("Failed to grow WASM memory: {}", e);
            return -1;
        }
    }

    // Write response data to allocated memory
    let memory_view = memory.view(&env);
    for (i, &byte) in response_bytes.iter().enumerate() {
        if let Err(e) = memory_view.write_u8((alloc_ptr + i as u32) as u64, byte) {
            tracing::error!("Failed to write response to WASM memory: {}", e);
            return -1;
        }
    }

    // Write response pointer and length back to WASM using byte-by-byte approach
    let alloc_ptr_bytes = alloc_ptr.to_le_bytes();
    for (i, &byte) in alloc_ptr_bytes.iter().enumerate() {
        if let Err(e) = memory_view.write_u8((out_ptr_ptr + i as u32) as u64, byte) {
            tracing::error!("Failed to write response pointer: {}", e);
            return -1;
        }
    }

    let response_len_bytes = (response_len as u32).to_le_bytes();
    for (i, &byte) in response_len_bytes.iter().enumerate() {
        if let Err(e) = memory_view.write_u8((out_len_ptr + i as u32) as u64, byte) {
            tracing::error!("Failed to write response length: {}", e);
            return -1;
        }
    }

    0 // Success
}

/// Host function for getting environment variables from WASM
#[instrument(skip_all, fields(operation = "host_get_env"))]
fn host_get_env(
    mut env: wasmer::FunctionEnvMut<WasmEnv>,
    key_ptr: u32,
    key_len: u32,
    out_ptr_ptr: u32,
    out_len_ptr: u32,
) -> i32 {
    tracing::debug!("Environment variable request from WASM lambda");

    // Get memory from the environment
    let memory = match env.data().memory.clone() {
        Some(mem) => mem,
        None => {
            tracing::error!("Failed to get WASM memory in environment host function");
            return -1;
        }
    };

    // Read environment variable key from WASM memory
    let key_bytes = {
        let view = memory.view(&env);
        let mut bytes = Vec::with_capacity(key_len as usize);
        for i in 0..key_len {
            match view.read_u8((key_ptr + i) as u64) {
                Ok(byte) => bytes.push(byte),
                Err(e) => {
                    tracing::error!("Failed to read key from WASM memory: {}", e);
                    return -1;
                }
            }
        }
        bytes
    };

    // Convert bytes to string
    let key = match String::from_utf8(key_bytes) {
        Ok(key) => key,
        Err(e) => {
            tracing::error!("Failed to convert key to UTF-8 string: {}", e);
            return -1;
        }
    };

    tracing::debug!(key = %key, "Retrieving environment variable");

    // Get environment variable
    let env_value = std::env::var(&key);

    // Create response JSON
    let response_json = match env_value {
        Ok(value) => serde_json::json!(Some(value)),
        Err(_) => serde_json::json!(None::<String>),
    };

    let response_bytes = match serde_json::to_vec(&response_json) {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to serialize environment variable response: {}", e);
            return -1;
        }
    };

    // Use a simple approach: allocate in existing WASM memory and return pointer
    let response_len = response_bytes.len();

    // Get current memory size and use it as allocation point
    let memory_view = memory.view(&env);
    let current_size = memory_view.data_size() as usize;
    let alloc_ptr = current_size as u32;

    // Grow memory if needed
    let needed_pages = ((current_size + response_len + 65535) / 65536) - (current_size / 65536);
    if needed_pages > 0 {
        if let Err(e) = memory.grow(&mut env, needed_pages as u32) {
            tracing::error!("Failed to grow WASM memory: {}", e);
            return -1;
        }
    }

    // Write response data to allocated memory
    let memory_view = memory.view(&env);
    for (i, &byte) in response_bytes.iter().enumerate() {
        if let Err(e) = memory_view.write_u8((alloc_ptr + i as u32) as u64, byte) {
            tracing::error!("Failed to write response to WASM memory: {}", e);
            return -1;
        }
    }

    // Write response pointer and length back to WASM using byte-by-byte approach
    let alloc_ptr_bytes = alloc_ptr.to_le_bytes();
    for (i, &byte) in alloc_ptr_bytes.iter().enumerate() {
        if let Err(e) = memory_view.write_u8((out_ptr_ptr + i as u32) as u64, byte) {
            tracing::error!("Failed to write response pointer: {}", e);
            return -1;
        }
    }

    let response_len_bytes = (response_len as u32).to_le_bytes();
    for (i, &byte) in response_len_bytes.iter().enumerate() {
        if let Err(e) = memory_view.write_u8((out_len_ptr + i as u32) as u64, byte) {
            tracing::error!("Failed to write response length: {}", e);
            return -1;
        }
    }

    0 // Success
}

/// Execute WASM bytecode using wasmer with structured data support
#[instrument(
    skip_all,
    fields(infrastructure = "wasm_runtime", operation = "execute_wasm")
)]
pub(crate) async fn execute_wasm(wasm_bytes: &[u8], input_data: &[u8]) -> AppResult<Vec<u8>> {
    use wasmer::{Engine, Instance, Module, Store, Value};

    tracing::info!("Executing WASM lambda with structured data support");

    // Create wasmer engine and store using universal engine (avoids unwind issues)
    let engine = Engine::default();
    let mut store = Store::new(engine);

    // Compile WASM module
    let module = Module::new(&store, wasm_bytes)
        .map_err(|e| AppError::validation(&format!("Failed to compile WASM module: {}", e)))?;

    // Create function environment for host functions
    let env = wasmer::FunctionEnv::new(&mut store, WasmEnv::new());

    // Create instance first to get memory
    let imports = wasmer::imports! {
        "env" => {
            "host_http_request" => wasmer::Function::new_typed_with_env(&mut store, &env, host_http_request),
            "host_get_env" => wasmer::Function::new_typed_with_env(&mut store, &env, host_get_env),
            "wasm_free" => wasmer::Function::new_typed(&mut store, |ptr: u32, size: u32| {
                // This is called by WASM to free host-allocated memory
                // For now, we'll just log it since we're using simple allocation
                tracing::debug!(ptr = ptr, size = size, "WASM requested to free host memory");
            }),
        }
    };

    // Create instance
    let instance = Instance::new(&mut store, &module, &imports)
        .map_err(|e| AppError::validation(&format!("Failed to create WASM instance: {}", e)))?;

    // Get the exported handler function
    let handler = instance
        .exports
        .get_function("handler")
        .map_err(|e| AppError::validation(&format!("Handler function not found: {}", e)))?;

    // Get WASM memory for input/output operations
    let memory = instance
        .exports
        .get_memory("memory")
        .map_err(|_| AppError::validation("WASM memory not found"))?;

    // Update the environment with memory so host functions can access it
    env.as_mut(&mut store).memory = Some(memory.clone());

    // Allocate memory for input data
    let input_len = input_data.len();
    let input_ptr = {
        // Get current memory size in pages (64KB each)
        let memory_view = memory.view(&store);
        let current_size = memory_view.data_size() as usize;

        // Use current memory end as input pointer
        let ptr = current_size;

        // Grow memory if needed to accommodate input
        let needed_pages = ((ptr + input_len + 65535) / 65536) - (current_size / 65536);
        if needed_pages > 0 {
            memory
                .grow(&mut store, needed_pages as u32)
                .map_err(|e| AppError::validation(&format!("Failed to grow WASM memory: {}", e)))?;
        }

        ptr
    };

    // Write input data to WASM memory
    {
        let memory_view = memory.view(&store);
        for (i, &byte) in input_data.iter().enumerate() {
            memory_view
                .write_u8((input_ptr + i) as u64, byte)
                .map_err(|e| {
                    AppError::validation(&format!("Failed to write input to WASM memory: {}", e))
                })?;
        }
    }

    // Allocate memory for output buffer (max 1MB)
    let max_output_len = 1024 * 1024; // 1MB max output
    let output_ptr = {
        let memory_view = memory.view(&store);
        let current_size = memory_view.data_size() as usize;
        let ptr = current_size;

        // Grow memory if needed to accommodate output buffer
        let needed_pages = ((ptr + max_output_len + 65535) / 65536) - (current_size / 65536);
        if needed_pages > 0 {
            memory.grow(&mut store, needed_pages as u32).map_err(|e| {
                AppError::validation(&format!("Failed to grow WASM memory for output: {}", e))
            })?;
        }

        ptr
    };

    tracing::debug!(
        input_len = input_len,
        input_ptr = input_ptr,
        output_ptr = output_ptr,
        max_output_len = max_output_len,
        "Calling handler with pointer-based interface"
    );

    // Call the handler function with pointer-based interface
    let results = handler
        .call(
            &mut store,
            &[
                Value::I32(input_ptr as i32),      // ptr_in
                Value::I32(input_len as i32),      // len_in
                Value::I32(output_ptr as i32),     // ptr_out
                Value::I32(max_output_len as i32), // max_out_len
            ],
        )
        .map_err(|e| AppError::validation(&format!("WASM function call failed: {}", e)))?;

    if results.is_empty() {
        return Err(AppError::validation("Handler function returned no results"));
    }

    // Extract the actual output length from results
    let actual_output_len = match results[0] {
        Value::I32(len) => len as usize,
        _ => {
            return Err(AppError::validation(
                "Handler function must return output length (i32)",
            ));
        }
    };

    if actual_output_len > max_output_len {
        return Err(AppError::validation(&format!(
            "Handler returned invalid output length: {} > {}",
            actual_output_len, max_output_len
        )));
    }

    // Read the output data from WASM memory
    let output_data = {
        let memory_view = memory.view(&store);
        let mut output_bytes = Vec::with_capacity(actual_output_len);

        for i in 0..actual_output_len {
            let byte = memory_view.read_u8((output_ptr + i) as u64).map_err(|e| {
                AppError::validation(&format!("Failed to read output from WASM memory: {}", e))
            })?;
            output_bytes.push(byte);
        }

        output_bytes
    };

    tracing::debug!(
        output_len = actual_output_len,
        "Successfully received output from lambda"
    );

    Ok(output_data)
}
