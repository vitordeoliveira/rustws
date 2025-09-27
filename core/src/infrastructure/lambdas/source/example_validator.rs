//! Example Validator Lambda - Validates code examples in documentation
//!
//! This lambda extracts and validates code examples from documentation to ensure
//! they are syntactically correct, executable, and follow best practices.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== INPUT/OUTPUT TYPES =====

/// Input for example validation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExampleValidatorRequest {
    /// The documentation content containing examples
    pub documentation: String,
    /// Type of examples to validate (e.g., "rust", "javascript", "shell", "json")
    pub example_types: Vec<String>,
    /// Validation configuration
    pub validation_config: ValidationConfig,
    /// Additional context for validation
    pub context: Option<String>,
}

/// Configuration for example validation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationConfig {
    /// Whether to check syntax for code examples
    pub check_syntax: Option<bool>,
    /// Whether examples should be executable/runnable
    pub require_executable: Option<bool>,
    /// Whether to validate that examples compile (for compiled languages)
    pub check_compilation: Option<bool>,
    /// Maximum number of examples to validate (for performance)
    pub max_examples: Option<u32>,
    /// Whether to check for best practices
    pub check_best_practices: Option<bool>,
}

/// Example validation results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExampleValidatorResponse {
    /// Overall validation result
    pub is_valid: bool,
    /// Validation quality score (0.0-1.0)
    pub validation_score: f64,
    /// Examples found and their validation results
    pub examples: Vec<ExampleValidation>,
    /// Summary of validation results
    pub validation_summary: ValidationSummary,
    /// Issues found during validation
    pub issues: Vec<ValidationIssue>,
}

/// Validation result for a single example
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExampleValidation {
    /// Unique identifier for this example
    pub example_id: String,
    /// Programming language of the example
    pub language: String,
    /// The example code content
    pub code: String,
    /// Line number where example starts
    pub line_number: u32,
    /// Whether this example is valid
    pub is_valid: bool,
    /// Validation score for this example (0.0-1.0)
    pub score: f64,
    /// Issues found in this example
    pub issues: Vec<String>,
    /// Validation details
    pub validation_details: ExampleValidationDetails,
}

/// Detailed validation information for an example
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExampleValidationDetails {
    /// Whether syntax is correct
    pub syntax_valid: bool,
    /// Whether code would compile (if applicable)
    pub compilation_valid: Option<bool>,
    /// Whether example is executable
    pub executable: Option<bool>,
    /// Best practices score (0.0-1.0)
    pub best_practices_score: f64,
    /// Complexity assessment
    pub complexity: ExampleComplexity,
}

/// Example complexity levels
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ExampleComplexity {
    /// Simple, straightforward example
    Simple,
    /// Moderate complexity
    Moderate,
    /// Complex example with multiple concepts
    Complex,
    /// Very complex or unclear example
    VeryComplex,
}

/// Summary of all validation results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationSummary {
    /// Total number of examples found
    pub total_examples: u32,
    /// Number of valid examples
    pub valid_examples: u32,
    /// Number of examples with issues
    pub examples_with_issues: u32,
    /// Most common language in examples
    pub primary_language: Option<String>,
    /// Languages found in examples
    pub languages_found: Vec<String>,
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationIssue {
    /// Type of issue
    pub issue_type: String,
    /// Description of the issue
    pub description: String,
    /// Example ID where issue occurred
    pub example_id: Option<String>,
    /// Line number of the issue
    pub line_number: Option<u32>,
    /// Severity of the issue
    pub severity: ValidationSeverity,
}

/// Validation issue severity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ValidationSeverity {
    /// Critical issue that makes example unusable
    Critical,
    /// Major issue that significantly impacts example quality
    Major,
    /// Minor issue that slightly impacts quality
    Minor,
    /// Warning or suggestion
    Warning,
}

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features = [env])]
fn example_validator_lambda(input: ExampleValidatorRequest) -> ExampleValidatorResponse {
    // Starting example validation

    // Validate input
    if input.documentation.trim().is_empty() {
        return create_empty_doc_response();
    }

    // Extract code examples from documentation
    let extracted_examples = extract_code_examples(&input.documentation, &input.example_types);

    if extracted_examples.is_empty() {
        return create_no_examples_response();
    }

    // Extracted code examples

    // Validate each example
    let mut validated_examples = Vec::new();
    let mut all_issues = Vec::new();
    let max_examples = input.validation_config.max_examples.unwrap_or(50);

    for (idx, example) in extracted_examples
        .into_iter()
        .take(max_examples as usize)
        .enumerate()
    {
        let example_id = format!("example_{}", idx + 1);
        let validation_result =
            validate_single_example(&example, &example_id, &input.validation_config);

        // Collect issues from this example
        for issue_desc in &validation_result.issues {
            all_issues.push(ValidationIssue {
                issue_type: "example_issue".to_string(),
                description: issue_desc.clone(),
                example_id: Some(example_id.clone()),
                line_number: Some(validation_result.line_number),
                severity: if validation_result.is_valid {
                    ValidationSeverity::Warning
                } else {
                    ValidationSeverity::Major
                },
            });
        }

        validated_examples.push(validation_result);
    }

    // Calculate overall validation score and summary
    let validation_summary = calculate_validation_summary(&validated_examples);
    let overall_score = calculate_overall_score(&validated_examples);
    let is_valid = overall_score >= 0.7 && !has_critical_issues(&all_issues);

    // Example validation completed

    ExampleValidatorResponse {
        is_valid,
        validation_score: overall_score,
        examples: validated_examples,
        validation_summary,
        issues: all_issues,
    }
}

// ===== VALIDATION FUNCTIONS =====

/// Extracted code example
#[derive(Debug, Clone)]
struct ExtractedExample {
    language: String,
    code: String,
    line_number: u32,
}

/// Extract code examples from documentation
fn extract_code_examples(content: &str, target_languages: &[String]) -> Vec<ExtractedExample> {
    let mut examples = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut in_code_block = false;
    let mut current_language = String::new();
    let mut current_code = String::new();
    let mut code_start_line = 0;

    for (line_num, line) in lines.iter().enumerate() {
        if line.trim().starts_with("```") {
            if in_code_block {
                // End of code block
                if should_validate_language(&current_language, target_languages) {
                    examples.push(ExtractedExample {
                        language: current_language.clone(),
                        code: current_code.trim().to_string(),
                        line_number: (code_start_line + 1) as u32,
                    });
                }
                in_code_block = false;
                current_code.clear();
                current_language.clear();
            } else {
                // Start of code block
                current_language = line
                    .trim()
                    .strip_prefix("```")
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if current_language.is_empty() {
                    current_language = "unknown".to_string();
                }
                in_code_block = true;
                code_start_line = line_num;
            }
        } else if in_code_block {
            if !current_code.is_empty() {
                current_code.push('\n');
            }
            current_code.push_str(line);
        }
    }

    examples
}

/// Check if we should validate this language
fn should_validate_language(language: &str, target_languages: &[String]) -> bool {
    if target_languages.is_empty() {
        return true; // Validate all languages if none specified
    }

    let lang_lower = language.to_lowercase();
    target_languages
        .iter()
        .any(|target| target.to_lowercase() == lang_lower)
}

/// Validate a single code example
fn validate_single_example(
    example: &ExtractedExample,
    example_id: &str,
    config: &ValidationConfig,
) -> ExampleValidation {
    let mut issues = Vec::new();
    let mut syntax_valid = true;
    let mut compilation_valid = None;
    let mut executable = None;
    let mut best_practices_score = 1.0;

    // Validate syntax based on language
    if config.check_syntax.unwrap_or(true) {
        let syntax_result = validate_syntax(&example.language, &example.code);
        syntax_valid = syntax_result.is_valid;
        if !syntax_valid {
            issues.extend(syntax_result.issues);
        }
    }

    // Check compilation (for compiled languages)
    if config.check_compilation.unwrap_or(false) {
        compilation_valid = Some(check_compilation(&example.language, &example.code));
        if compilation_valid == Some(false) {
            issues.push("Code does not compile".to_string());
        }
    }

    // Check if executable
    if config.require_executable.unwrap_or(false) {
        executable = Some(check_executable(&example.language, &example.code));
        if executable == Some(false) {
            issues.push("Example is not executable".to_string());
        }
    }

    // Check best practices
    if config.check_best_practices.unwrap_or(true) {
        let best_practices_result = check_best_practices(&example.language, &example.code);
        best_practices_score = best_practices_result.score;
        issues.extend(best_practices_result.issues);
    }

    // Assess complexity
    let complexity = assess_complexity(&example.code);

    // Calculate overall score for this example
    let mut score = 1.0;
    if !syntax_valid {
        score -= 0.5;
    }
    if compilation_valid == Some(false) {
        score -= 0.3;
    }
    if executable == Some(false) {
        score -= 0.2;
    }
    score *= best_practices_score;
    score = score.max(0.0_f64).min(1.0);

    let is_valid = syntax_valid && issues.len() < 3; // Allow minor issues

    ExampleValidation {
        example_id: example_id.to_string(),
        language: example.language.clone(),
        code: example.code.clone(),
        line_number: example.line_number,
        is_valid,
        score,
        issues,
        validation_details: ExampleValidationDetails {
            syntax_valid,
            compilation_valid,
            executable,
            best_practices_score,
            complexity,
        },
    }
}

// ===== VALIDATION HELPERS =====

struct SyntaxValidationResult {
    is_valid: bool,
    issues: Vec<String>,
}

struct BestPracticesResult {
    score: f64,
    issues: Vec<String>,
}

/// Validate syntax for different languages
fn validate_syntax(language: &str, code: &str) -> SyntaxValidationResult {
    let mut issues = Vec::new();

    match language.to_lowercase().as_str() {
        "rust" => validate_rust_syntax(code, &mut issues),
        "javascript" | "js" => validate_javascript_syntax(code, &mut issues),
        "python" => validate_python_syntax(code, &mut issues),
        "json" => validate_json_syntax(code, &mut issues),
        "shell" | "bash" => validate_shell_syntax(code, &mut issues),
        _ => {
            // Basic validation for unknown languages
            if code.trim().is_empty() {
                issues.push("Example is empty".to_string());
            }
        }
    }

    SyntaxValidationResult {
        is_valid: issues.is_empty(),
        issues,
    }
}

/// Validate Rust syntax
fn validate_rust_syntax(code: &str, issues: &mut Vec<String>) {
    // Basic Rust syntax checks
    if !code.contains("fn ")
        && !code.contains("let ")
        && !code.contains("struct ")
        && !code.contains("impl ")
        && !code.contains("use ")
        && code.lines().count() > 1
    {
        issues.push("Rust code should contain recognizable Rust syntax".to_string());
    }

    // Check for unmatched braces
    let open_braces = code.matches('{').count();
    let close_braces = code.matches('}').count();
    if open_braces != close_braces {
        issues.push("Unmatched braces in Rust code".to_string());
    }

    // Check for unmatched parentheses
    let open_parens = code.matches('(').count();
    let close_parens = code.matches(')').count();
    if open_parens != close_parens {
        issues.push("Unmatched parentheses in Rust code".to_string());
    }
}

/// Validate JavaScript syntax
fn validate_javascript_syntax(code: &str, issues: &mut Vec<String>) {
    // Basic JavaScript syntax checks
    if code.trim().is_empty() {
        issues.push("JavaScript example is empty".to_string());
        return;
    }

    // Check for basic JS patterns
    let has_js_keywords = code.contains("function")
        || code.contains("const ")
        || code.contains("let ")
        || code.contains("var ")
        || code.contains("=>")
        || code.contains("console.");

    if !has_js_keywords && code.lines().count() > 1 {
        issues.push("JavaScript code should contain recognizable JavaScript syntax".to_string());
    }
}

/// Validate Python syntax
fn validate_python_syntax(code: &str, issues: &mut Vec<String>) {
    if code.trim().is_empty() {
        issues.push("Python example is empty".to_string());
        return;
    }

    // Check for Python patterns
    let has_python_keywords = code.contains("def ")
        || code.contains("class ")
        || code.contains("import ")
        || code.contains("print(")
        || code.contains("if ")
        || code.contains("for ");

    if !has_python_keywords && code.lines().count() > 1 {
        issues.push("Python code should contain recognizable Python syntax".to_string());
    }
}

/// Validate JSON syntax
fn validate_json_syntax(code: &str, issues: &mut Vec<String>) {
    if let Err(_) = serde_json::from_str::<serde_json::Value>(code) {
        issues.push("Invalid JSON syntax".to_string());
    }
}

/// Validate shell syntax
fn validate_shell_syntax(code: &str, issues: &mut Vec<String>) {
    if code.trim().is_empty() {
        issues.push("Shell example is empty".to_string());
        return;
    }

    // Basic shell command validation
    let lines: Vec<&str> = code.lines().collect();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue; // Skip empty lines and comments
        }

        // Check for dangerous commands (basic safety check)
        if trimmed.contains("rm -rf /") || trimmed.contains("format") {
            issues.push("Shell example contains potentially dangerous commands".to_string());
        }
    }
}

/// Check if code compiles (simplified)
fn check_compilation(language: &str, code: &str) -> bool {
    match language.to_lowercase().as_str() {
        "rust" => {
            // For Rust, we'd need to actually compile - for now, just check basic structure
            code.contains("fn ") || code.contains("struct ") || code.lines().count() == 1
        }
        "javascript" | "python" => {
            // Interpreted languages - if syntax is valid, they "compile"
            true
        }
        _ => true,
    }
}

/// Check if example is executable
fn check_executable(language: &str, code: &str) -> bool {
    match language.to_lowercase().as_str() {
        "rust" => code.contains("fn main") || code.lines().count() <= 3,
        "javascript" => !code.contains("function") || code.contains("()"),
        "python" => !code.contains("def ") || code.contains("if __name__"),
        "shell" => true, // Most shell commands are executable
        _ => true,
    }
}

/// Check best practices
fn check_best_practices(language: &str, code: &str) -> BestPracticesResult {
    let mut score: f64 = 1.0;
    let mut issues = Vec::new();

    // Generic best practices
    if code.lines().count() > 50 {
        score -= 0.1;
        issues.push("Example is quite long - consider breaking into smaller examples".to_string());
    }

    if !code.contains("//") && !code.contains("#") && code.lines().count() > 5 {
        score -= 0.1;
        issues.push("Complex example should include comments".to_string());
    }

    // Language-specific best practices
    match language.to_lowercase().as_str() {
        "rust" => {
            if code.contains("unwrap()") && !code.contains("// Example") {
                score -= 0.2;
                issues.push("Avoid unwrap() in examples - use proper error handling".to_string());
            }
        }
        "javascript" => {
            if code.contains("var ") {
                score -= 0.1;
                issues.push("Consider using 'let' or 'const' instead of 'var'".to_string());
            }
        }
        _ => {}
    }

    BestPracticesResult {
        score: score.max(0.0),
        issues,
    }
}

/// Assess example complexity
fn assess_complexity(code: &str) -> ExampleComplexity {
    let line_count = code.lines().count();
    let word_count = code.split_whitespace().count();

    if line_count <= 3 && word_count <= 20 {
        ExampleComplexity::Simple
    } else if line_count <= 10 && word_count <= 50 {
        ExampleComplexity::Moderate
    } else if line_count <= 25 && word_count <= 150 {
        ExampleComplexity::Complex
    } else {
        ExampleComplexity::VeryComplex
    }
}

// ===== HELPER FUNCTIONS =====

/// Calculate validation summary
fn calculate_validation_summary(examples: &[ExampleValidation]) -> ValidationSummary {
    let total_examples = examples.len() as u32;
    let valid_examples = examples.iter().filter(|e| e.is_valid).count() as u32;
    let examples_with_issues = examples.iter().filter(|e| !e.issues.is_empty()).count() as u32;

    // Find most common language
    let mut language_counts = std::collections::HashMap::new();
    let mut languages_found = Vec::new();

    for example in examples {
        *language_counts.entry(example.language.clone()).or_insert(0) += 1;
        if !languages_found.contains(&example.language) {
            languages_found.push(example.language.clone());
        }
    }

    let primary_language = language_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(lang, _)| lang);

    ValidationSummary {
        total_examples,
        valid_examples,
        examples_with_issues,
        primary_language,
        languages_found,
    }
}

/// Calculate overall validation score
fn calculate_overall_score(examples: &[ExampleValidation]) -> f64 {
    if examples.is_empty() {
        return 0.0;
    }

    let total_score: f64 = examples.iter().map(|e| e.score).sum();
    total_score / examples.len() as f64
}

/// Check if there are critical issues
fn has_critical_issues(issues: &[ValidationIssue]) -> bool {
    issues
        .iter()
        .any(|issue| matches!(issue.severity, ValidationSeverity::Critical))
}

/// Create response when no documentation provided
fn create_empty_doc_response() -> ExampleValidatorResponse {
    ExampleValidatorResponse {
        is_valid: false,
        validation_score: 0.0,
        examples: vec![],
        validation_summary: ValidationSummary {
            total_examples: 0,
            valid_examples: 0,
            examples_with_issues: 0,
            primary_language: None,
            languages_found: vec![],
        },
        issues: vec![ValidationIssue {
            issue_type: "empty_documentation".to_string(),
            description: "No documentation provided for validation".to_string(),
            example_id: None,
            line_number: None,
            severity: ValidationSeverity::Critical,
        }],
    }
}

/// Create response when no examples found
fn create_no_examples_response() -> ExampleValidatorResponse {
    ExampleValidatorResponse {
        is_valid: false,
        validation_score: 0.0,
        examples: vec![],
        validation_summary: ValidationSummary {
            total_examples: 0,
            valid_examples: 0,
            examples_with_issues: 0,
            primary_language: None,
            languages_found: vec![],
        },
        issues: vec![ValidationIssue {
            issue_type: "no_examples_found".to_string(),
            description: "No code examples found in documentation".to_string(),
            example_id: None,
            line_number: None,
            severity: ValidationSeverity::Major,
        }],
    }
}
