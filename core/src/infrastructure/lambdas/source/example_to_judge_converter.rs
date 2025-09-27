//! Example to Judge Converter Lambda - Transforms example validator output for judge evaluation
//!
//! This lambda converts ExampleValidatorResponse into JudgeRequest for clarity
//! assessment using the judge lambda.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== INPUT/OUTPUT TYPES =====

/// Example validation results (input to this converter)
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

/// Judge Request (output from this converter)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JudgeRequest {
    /// The prompt/content to be judged
    pub prompt: String,
    /// What aspect to judge
    pub judgment_criteria: String,
    /// Optional context or additional rules for judgment
    pub context: Option<String>,
    /// Whether to use AI assistance for complex judgments
    pub use_ai_assistance: Option<bool>,
}

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features = [env])]
fn example_to_judge_converter_lambda(input: ExampleValidatorResponse) -> JudgeRequest {
    // Build comprehensive clarity assessment prompt
    let prompt = build_clarity_assessment_prompt(&input);

    // Build context for the judge with example analysis results
    let context = build_judge_context(&input);

    // Create judge request for clarity evaluation
    JudgeRequest {
        prompt,
        judgment_criteria: "content_safety".to_string(),
        context: Some(context),
        use_ai_assistance: Some(false), // Use rule-based evaluation for speed
    }
}

// ===== HELPER FUNCTIONS =====

/// Build clarity assessment prompt
fn build_clarity_assessment_prompt(input: &ExampleValidatorResponse) -> String {
    let mut prompt_parts = Vec::new();

    // Overall validation summary
    prompt_parts.push(format!(
        "EXAMPLE VALIDATION ANALYSIS RESULTS:\n\nOverall Valid: {}\nValidation Score: {:.2}/1.0",
        input.is_valid, input.validation_score
    ));

    // Example statistics
    prompt_parts.push(format!(
        "EXAMPLE STATISTICS:\n- Total Examples: {}\n- Valid Examples: {}\n- Examples with Issues: {}\n- Primary Language: {}",
        input.validation_summary.total_examples,
        input.validation_summary.valid_examples,
        input.validation_summary.examples_with_issues,
        input.validation_summary.primary_language.as_deref().unwrap_or("None")
    ));

    // Languages used
    if !input.validation_summary.languages_found.is_empty() {
        prompt_parts.push(format!(
            "LANGUAGES FOUND: {}",
            input.validation_summary.languages_found.join(", ")
        ));
    }

    // Individual example analysis
    if !input.examples.is_empty() {
        prompt_parts.push("INDIVIDUAL EXAMPLE ANALYSIS:".to_string());

        for example in &input.examples {
            let mut example_details = Vec::new();
            example_details.push(format!("ID: {}", example.example_id));
            example_details.push(format!("Language: {}", example.language));
            example_details.push(format!("Valid: {}", example.is_valid));
            example_details.push(format!("Score: {:.2}", example.score));
            example_details.push(format!(
                "Complexity: {:?}",
                example.validation_details.complexity
            ));
            example_details.push(format!(
                "Syntax Valid: {}",
                example.validation_details.syntax_valid
            ));
            example_details.push(format!(
                "Best Practices Score: {:.2}",
                example.validation_details.best_practices_score
            ));

            if !example.issues.is_empty() {
                example_details.push(format!("Issues: {}", example.issues.join("; ")));
            }

            prompt_parts.push(format!("  {}", example_details.join(" | ")));
        }
    }

    // General issues summary
    if !input.issues.is_empty() {
        let mut issues_by_severity = std::collections::HashMap::new();
        for issue in &input.issues {
            let severity_key = format!("{:?}", issue.severity);
            issues_by_severity
                .entry(severity_key)
                .or_insert_with(Vec::new)
                .push(&issue.description);
        }

        prompt_parts.push("VALIDATION ISSUES FOUND:".to_string());
        for (severity, descriptions) in issues_by_severity {
            prompt_parts.push(format!("{} Issues:", severity));
            for desc in descriptions {
                prompt_parts.push(format!("  - {}", desc));
            }
        }
    } else {
        prompt_parts.push("No validation issues detected.".to_string());
    }

    // Clarity assessment requirements
    prompt_parts.push("CLARITY ASSESSMENT NEEDED:".to_string());
    prompt_parts.push(
        "Please evaluate if these examples provide clear, understandable documentation."
            .to_string(),
    );

    prompt_parts.join("\n\n")
}

/// Build context information for the judge
fn build_judge_context(input: &ExampleValidatorResponse) -> String {
    let mut context_parts = Vec::new();

    // Validation summary
    context_parts.push(format!(
        "Example Validation: {}",
        if input.is_valid { "PASSED" } else { "FAILED" }
    ));
    context_parts.push(format!("Validation Score: {:.2}", input.validation_score));
    context_parts.push(format!(
        "Examples Found: {}",
        input.validation_summary.total_examples
    ));
    context_parts.push(format!(
        "Valid Examples: {}",
        input.validation_summary.valid_examples
    ));

    // Quality indicators
    let success_rate = if input.validation_summary.total_examples > 0 {
        input.validation_summary.valid_examples as f64
            / input.validation_summary.total_examples as f64
    } else {
        0.0
    };
    context_parts.push(format!("Success Rate: {:.1}%", success_rate * 100.0));

    // Complexity analysis
    let complexity_distribution = analyze_complexity_distribution(&input.examples);
    if !complexity_distribution.is_empty() {
        context_parts.push(format!(
            "Complexity Distribution: {}",
            complexity_distribution
        ));
    }

    // Issue severity breakdown
    let critical_issues = input
        .issues
        .iter()
        .filter(|i| matches!(i.severity, ValidationSeverity::Critical))
        .count();
    let major_issues = input
        .issues
        .iter()
        .filter(|i| matches!(i.severity, ValidationSeverity::Major))
        .count();
    let minor_issues = input
        .issues
        .iter()
        .filter(|i| matches!(i.severity, ValidationSeverity::Minor))
        .count();

    context_parts.push(format!("Critical Issues: {}", critical_issues));
    context_parts.push(format!("Major Issues: {}", major_issues));
    context_parts.push(format!("Minor Issues: {}", minor_issues));

    // Clarity criteria
    context_parts.push("CLARITY CRITERIA:".to_string());
    context_parts.push("- Examples should be syntactically correct".to_string());
    context_parts.push("- Examples should demonstrate clear use cases".to_string());
    context_parts.push("- Code should follow best practices".to_string());
    context_parts.push("- Complexity should be appropriate for documentation".to_string());
    context_parts.push("- Examples should be executable where applicable".to_string());

    // Decision thresholds
    context_parts.push("APPROVAL THRESHOLDS:".to_string());
    context_parts.push("- Validation score >= 0.7".to_string());
    context_parts.push("- Success rate >= 70%".to_string());
    context_parts.push("- No critical issues".to_string());
    context_parts.push("- <= 1 major issue per example".to_string());

    // Current assessment
    let should_approve = input.validation_score >= 0.7
        && success_rate >= 0.7
        && critical_issues == 0
        && major_issues <= input.validation_summary.total_examples as usize;

    context_parts.push(format!(
        "RECOMMENDED DECISION: {}",
        if should_approve { "APPROVE" } else { "DENY" }
    ));

    context_parts.join("\n")
}

/// Analyze complexity distribution of examples
fn analyze_complexity_distribution(examples: &[ExampleValidation]) -> String {
    if examples.is_empty() {
        return "No examples to analyze".to_string();
    }

    let mut complexity_counts = std::collections::HashMap::new();
    for example in examples {
        let complexity_key = format!("{:?}", example.validation_details.complexity);
        *complexity_counts.entry(complexity_key).or_insert(0) += 1;
    }

    let total = examples.len();
    let mut distribution_parts = Vec::new();

    for (complexity, count) in complexity_counts {
        let percentage = (count as f64 / total as f64) * 100.0;
        distribution_parts.push(format!("{}: {:.0}%", complexity, percentage));
    }

    distribution_parts.join(", ")
}
