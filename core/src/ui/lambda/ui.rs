//! Lambda domain UI logic

use crate::{
    auth::dto::User,
    business_logic::lambdas::{LambdaSummary, dto::LambdaMetadata},
    error_handling::types::AppResult,
    infrastructure::lambdas::{LambdaExecutionEntry, LambdasMetrics},
    ui::{TeraEngine, TeraRenderer, Ui, shared::layouts::BaseLayoutProps},
};
use axum::response::Html;
use serde::Serialize;
use serde_json::Value;

/// Helper function to convert JSON schemas to Rust struct format with full definitions support
fn json_schemas_to_rust_structs(
    input_schema: &Value,
    output_schema: &Value,
    input_type: &str,
    output_type: &str,
) -> (String, String) {
    // Extract definitions from both schemas
    let mut all_definitions = std::collections::HashMap::new();

    // Extract definitions from input schema
    if let Some(definitions) = input_schema.get("definitions").and_then(|d| d.as_object()) {
        for (key, value) in definitions {
            all_definitions.insert(key.clone(), value.clone());
        }
    }

    // Extract definitions from output schema
    if let Some(definitions) = output_schema.get("definitions").and_then(|d| d.as_object()) {
        for (key, value) in definitions {
            all_definitions.insert(key.clone(), value.clone());
        }
    }

    // Convert input schema
    let input_rust =
        json_schema_to_rust_struct_with_definitions(input_schema, input_type, &all_definitions);

    // Convert output schema
    let output_rust =
        json_schema_to_rust_struct_with_definitions(output_schema, output_type, &all_definitions);

    (input_rust, output_rust)
}

/// Convert JSON schema to Rust struct with access to definitions
fn json_schema_to_rust_struct_with_definitions(
    schema: &Value,
    struct_name: &str,
    definitions: &std::collections::HashMap<String, Value>,
) -> String {
    let mut rust_code = String::new();
    let mut nested_structs = Vec::new();
    let mut referenced_structs = Vec::new();

    // Generate the main struct
    rust_code.push_str(&generate_struct_with_definitions(
        schema,
        struct_name,
        &mut nested_structs,
        &mut referenced_structs,
        definitions,
    ));

    // Add nested structs
    for nested_struct in nested_structs {
        rust_code.push_str("\n");
        rust_code.push_str(&nested_struct);
    }

    // Add referenced structs from definitions
    for referenced_struct in referenced_structs {
        rust_code.push_str("\n");
        rust_code.push_str(&referenced_struct);
    }

    rust_code
}

/// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Generate a single struct definition with recursive object handling and definitions support
fn generate_struct_with_definitions(
    schema: &Value,
    struct_name: &str,
    nested_structs: &mut Vec<String>,
    referenced_structs: &mut Vec<String>,
    definitions: &std::collections::HashMap<String, Value>,
) -> String {
    let mut rust_code = format!(
        "#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]\npub struct {} {{\n",
        struct_name
    );

    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        for (field_name, field_schema) in properties {
            let field_type = get_rust_type_with_definitions(
                field_schema,
                field_name,
                nested_structs,
                referenced_structs,
                definitions,
            );

            let field_doc = field_schema
                .get("description")
                .and_then(|d| d.as_str())
                .map(|d| format!("    /// {}\n", d))
                .unwrap_or_default();

            rust_code.push_str(&field_doc);
            rust_code.push_str(&format!("    pub {}: {},\n", field_name, field_type));
        }
    }

    rust_code.push_str("}\n");
    rust_code
}

/// Get the Rust type for a field, handling nested objects recursively with definitions support
fn get_rust_type_with_definitions(
    field_schema: &Value,
    field_name: &str,
    nested_structs: &mut Vec<String>,
    referenced_structs: &mut Vec<String>,
    definitions: &std::collections::HashMap<String, Value>,
) -> String {
    // Check if this is an Option type (nullable field or anyOf/oneOf with null)
    let (is_nullable, actual_schema) = if let Some(any_of) = field_schema.get("anyOf") {
        // Handle anyOf case (common for Option<T>)
        if let Some(any_of_array) = any_of.as_array() {
            let mut nullable = false;
            let mut non_null_schema = None;

            for schema in any_of_array {
                if let Some(schema_obj) = schema.as_object() {
                    if schema_obj.get("type").and_then(|t| t.as_str()) == Some("null") {
                        nullable = true;
                    } else {
                        non_null_schema = Some(schema);
                    }
                }
            }

            if nullable && non_null_schema.is_some() {
                (true, non_null_schema.unwrap())
            } else {
                (false, field_schema)
            }
        } else {
            (false, field_schema)
        }
    } else if let Some(nullable) = field_schema.get("nullable").and_then(|n| n.as_bool()) {
        // Handle simple nullable case
        (nullable, field_schema)
    } else {
        (false, field_schema)
    };

    // Handle $ref references with definitions
    let actual_type = if let Some(ref_path) = actual_schema.get("$ref").and_then(|r| r.as_str()) {
        // Extract the referenced type name from the $ref path
        if let Some(type_name) = ref_path.split('/').last() {
            // Look up the referenced type in definitions
            if let Some(ref_schema) = definitions.get(type_name) {
                // Generate the referenced struct definition
                let referenced_struct = generate_struct_with_definitions(
                    ref_schema,
                    type_name,
                    nested_structs,
                    referenced_structs,
                    definitions,
                );
                referenced_structs.push(referenced_struct);
            }

            return if is_nullable {
                format!("Option<{}>", type_name)
            } else {
                type_name.to_string()
            };
        }
        "object"
    } else {
        actual_schema
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("object")
    };

    let rust_type = match actual_type {
        "string" => "String".to_string(),
        "integer" => "i32".to_string(),
        "number" => "f64".to_string(),
        "boolean" => "bool".to_string(),
        "array" => {
            // Handle array types
            if let Some(items) = actual_schema.get("items") {
                let item_type = get_rust_type_with_definitions(
                    items,
                    &format!("{}_item", field_name),
                    nested_structs,
                    referenced_structs,
                    definitions,
                );
                format!("Vec<{}>", item_type)
            } else {
                "Vec<serde_json::Value>".to_string()
            }
        }
        "object" => {
            // Handle object types recursively
            if let Some(properties) = actual_schema.get("properties").and_then(|p| p.as_object()) {
                if !properties.is_empty() {
                    // Generate a nested struct name
                    let nested_struct_name = format!("{}Struct", capitalize_first(field_name));

                    // Generate the nested struct
                    let nested_struct = generate_struct_with_definitions(
                        actual_schema,
                        &nested_struct_name,
                        nested_structs,
                        referenced_structs,
                        definitions,
                    );
                    nested_structs.push(nested_struct);

                    nested_struct_name
                } else {
                    "serde_json::Value".to_string()
                }
            } else {
                "serde_json::Value".to_string()
            }
        }
        _ => {
            // Try to handle complex types by looking for properties
            if let Some(properties) = actual_schema.get("properties").and_then(|p| p.as_object()) {
                if !properties.is_empty() {
                    // Generate a nested struct name
                    let nested_struct_name = format!("{}Struct", capitalize_first(field_name));

                    // Generate the nested struct
                    let nested_struct = generate_struct_with_definitions(
                        actual_schema,
                        &nested_struct_name,
                        nested_structs,
                        referenced_structs,
                        definitions,
                    );
                    nested_structs.push(nested_struct);

                    nested_struct_name
                } else {
                    "serde_json::Value".to_string()
                }
            } else {
                "serde_json::Value".to_string()
            }
        }
    };

    // Wrap in Option if nullable
    if is_nullable {
        format!("Option<{}>", rust_type)
    } else {
        rust_type
    }
}

#[derive(Debug, Serialize)]
pub struct LambdaPageUi {
    layout: BaseLayoutProps,
    lambdas: Vec<LambdaSummary>,
    metrics: LambdasMetrics,
}

impl LambdaPageUi {
    pub fn new(user: User, lambdas: Vec<LambdaSummary>, metrics: LambdasMetrics) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("Lambda Functions - RUSTWS Core")
                .description("Serverless compute functions with automatic scaling and event-driven execution")
                .keywords("lambda, serverless, functions, compute, scaling")
                .user(Some(user)),
            lambdas,
            metrics,
        }
    }
}

impl Ui for LambdaPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("lambdas", &self.lambdas);
        context.insert("metrics", &self.metrics);
        tera.render_template("lambda/index.html", &context)
    }
}

#[derive(Debug, Serialize)]
pub struct CreateLambdaPageUi {
    layout: BaseLayoutProps,
}

impl CreateLambdaPageUi {
    pub fn new(user: User) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("Create Lambda Function - RUSTWS Core")
                .description("Create and deploy serverless lambda functions with automatic scaling")
                .keywords("lambda, serverless, create, functions, deploy, rustws")
                .user(Some(user)),
        }
    }
}

impl Ui for CreateLambdaPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let context = self.layout.to_context()?;
        tera.render_template("lambda/create.html", &context)
    }
}

#[derive(Debug, Serialize)]
pub struct EditLambdaPageUi {
    layout: BaseLayoutProps,
    lambda_name: String,
    current_source_code: String,
}

impl EditLambdaPageUi {
    pub fn new(user: User, lambda_name: String, current_source_code: String) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title(&format!(
                    "Edit Lambda Function: {} - RUSTWS Core",
                    lambda_name
                ))
                .description(
                    "Edit and update your serverless lambda function configuration and source code",
                )
                .keywords("lambda, serverless, edit, update, functions, rustws")
                .user(Some(user)),
            lambda_name,
            current_source_code,
        }
    }
}

impl Ui for EditLambdaPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("lambda_name", &self.lambda_name);
        context.insert("current_source_code", &self.current_source_code);
        tera.render_template("lambda/edit.html", &context)
    }
}

#[derive(Debug, Serialize)]
pub struct LambdaMetricsPageUi {
    layout: BaseLayoutProps,
    lambda_name: String,
    executions: Vec<LambdaExecutionEntry>,
    total_executions: u64,
    successful_executions: u64,
    failed_executions: u64,
    avg_execution_time_ms: f64,
    success_rate: f64,
    lambda_metadata: Option<LambdaMetadata>,
}

impl LambdaMetricsPageUi {
    pub fn new(
        user: User,
        lambda_name: String,
        executions: Vec<LambdaExecutionEntry>,
        lambda_metadata: Option<LambdaMetadata>,
    ) -> Self {
        // Calculate metrics from execution entries
        let total_executions = executions.len() as u64;
        let successful_executions = executions.iter().filter(|e| e.is_success()).count() as u64;
        let failed_executions = total_executions - successful_executions;

        let avg_execution_time_ms = if total_executions > 0 {
            executions
                .iter()
                .map(|e| e.execution_time_ms as f64)
                .sum::<f64>()
                / total_executions as f64
        } else {
            0.0
        };

        let success_rate = if total_executions > 0 {
            (successful_executions as f64 / total_executions as f64) * 100.0
        } else {
            0.0
        };

        Self {
            layout: BaseLayoutProps::new()
                .title(&format!("Lambda Metrics: {} - RUSTWS Core", lambda_name))
                .description(&format!(
                    "Performance metrics and execution history for lambda function {}",
                    lambda_name
                ))
                .keywords("lambda, metrics, performance, execution, monitoring, serverless")
                .user(Some(user)),
            lambda_name,
            executions,
            total_executions,
            successful_executions,
            failed_executions,
            avg_execution_time_ms,
            success_rate,
            lambda_metadata,
        }
    }
}

impl Ui for LambdaMetricsPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("lambda_name", &self.lambda_name);
        context.insert("executions", &self.executions);
        context.insert("total_executions", &self.total_executions);
        context.insert("successful_executions", &self.successful_executions);
        context.insert("failed_executions", &self.failed_executions);
        context.insert("avg_execution_time_ms", &self.avg_execution_time_ms);
        context.insert("success_rate", &self.success_rate);
        context.insert("lambda_metadata", &self.lambda_metadata);

        // Pre-compute Rust struct strings if metadata is available
        if let Some(ref metadata) = self.lambda_metadata {
            let (input_rust_struct, output_rust_struct) = json_schemas_to_rust_structs(
                &metadata.input_schema,
                &metadata.output_schema,
                &metadata.input_type,
                &metadata.output_type,
            );
            context.insert("input_rust_struct", &input_rust_struct);
            context.insert("output_rust_struct", &output_rust_struct);
        }

        tera.render_template("lambda/metrics.html", &context)
    }
}
