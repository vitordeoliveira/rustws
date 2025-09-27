//! Format to Judge Converter Lambda - Transforms format checker output for judge evaluation
//!
//! This lambda converts FormatCheckerResponse into JudgeRequest for technical
//! accuracy evaluation using the judge lambda.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== INPUT/OUTPUT TYPES =====

/// Format checker response (input to this converter)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormatCheckerResponse {
    /// Overall format validation result
    pub is_valid: bool,
    /// Format quality score (0.0-1.0)
    pub format_score: f64,
    /// List of validation issues found
    pub issues: Vec<FormatIssue>,
    /// Sections detected in the document
    pub sections_found: Vec<String>,
    /// Format statistics
    pub format_stats: FormatStats,
    /// The content type that was checked
    pub content_type: String,
}

/// Individual format issue
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormatIssue {
    /// Type of issue
    pub issue_type: String,
    /// Human-readable description of the issue
    pub description: String,
    /// Line number where issue occurs (if applicable)
    pub line_number: Option<u32>,
    /// Severity level
    pub severity: IssueSeverity,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum IssueSeverity {
    /// Critical issue that makes document unusable
    Critical,
    /// Major issue that significantly impacts quality
    Major,
    /// Minor issue that slightly impacts quality
    Minor,
    /// Suggestion for improvement
    Suggestion,
}

/// Format statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormatStats {
    /// Total lines in document
    pub total_lines: u32,
    /// Number of headers found
    pub header_count: u32,
    /// Number of code blocks found
    pub code_block_count: u32,
    /// Number of links found
    pub link_count: u32,
    /// Average line length
    pub avg_line_length: f32,
    /// Longest line length
    pub max_line_length: u32,
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
fn format_to_judge_converter_lambda(input: FormatCheckerResponse) -> JudgeRequest {
    // Build comprehensive technical accuracy assessment prompt
    let prompt = build_technical_accuracy_prompt(&input);

    // Build context for the judge with format analysis results
    let context = build_judge_context(&input);

    // Create judge request for technical accuracy evaluation
    JudgeRequest {
        prompt,
        judgment_criteria: "code_safety".to_string(),
        context: Some(context),
        use_ai_assistance: Some(false), // Use rule-based evaluation for speed
    }
}

// ===== HELPER FUNCTIONS =====

/// Build technical accuracy assessment prompt
fn build_technical_accuracy_prompt(input: &FormatCheckerResponse) -> String {
    let mut prompt_parts = Vec::new();

    // Overall format assessment
    prompt_parts.push(format!(
        "DOCUMENT FORMAT ANALYSIS RESULTS:\n\nOverall Valid: {}\nFormat Score: {:.2}/1.0",
        input.is_valid, input.format_score
    ));

    // Statistical summary
    prompt_parts.push(format!(
        "DOCUMENT STATISTICS:\n- Total Lines: {}\n- Headers: {}\n- Code Blocks: {}\n- Links: {}\n- Average Line Length: {:.1}\n- Max Line Length: {}",
        input.format_stats.total_lines,
        input.format_stats.header_count,
        input.format_stats.code_block_count,
        input.format_stats.link_count,
        input.format_stats.avg_line_length,
        input.format_stats.max_line_length
    ));

    // Sections found
    if !input.sections_found.is_empty() {
        prompt_parts.push(format!(
            "SECTIONS DETECTED:\n{}",
            input.sections_found.join(", ")
        ));
    }

    // Issues breakdown
    if !input.issues.is_empty() {
        let mut issues_by_severity = std::collections::HashMap::new();
        for issue in &input.issues {
            let severity_key = format!("{:?}", issue.severity);
            issues_by_severity
                .entry(severity_key)
                .or_insert_with(Vec::new)
                .push(&issue.description);
        }

        prompt_parts.push("FORMAT ISSUES FOUND:".to_string());
        for (severity, descriptions) in issues_by_severity {
            prompt_parts.push(format!("{} Issues:", severity));
            for desc in descriptions {
                prompt_parts.push(format!("  - {}", desc));
            }
        }
    } else {
        prompt_parts.push("No format issues detected.".to_string());
    }

    // Technical accuracy indicators
    prompt_parts.push("TECHNICAL ACCURACY ASSESSMENT NEEDED:".to_string());
    prompt_parts.push(
        "Please evaluate if this format analysis indicates technically accurate documentation."
            .to_string(),
    );

    prompt_parts.join("\n\n")
}

/// Build context information for the judge
fn build_judge_context(input: &FormatCheckerResponse) -> String {
    let mut context_parts = Vec::new();

    // Format validation summary
    context_parts.push(format!(
        "Format Validation: {}",
        if input.is_valid { "PASSED" } else { "FAILED" }
    ));
    context_parts.push(format!("Format Quality Score: {:.2}", input.format_score));
    context_parts.push(format!("Content Type: {}", input.content_type));

    // Issue severity breakdown
    let critical_issues = input
        .issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::Critical))
        .count();
    let major_issues = input
        .issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::Major))
        .count();
    let minor_issues = input
        .issues
        .iter()
        .filter(|i| matches!(i.severity, IssueSeverity::Minor))
        .count();

    context_parts.push(format!("Critical Issues: {}", critical_issues));
    context_parts.push(format!("Major Issues: {}", major_issues));
    context_parts.push(format!("Minor Issues: {}", minor_issues));

    // Technical accuracy criteria
    context_parts.push("TECHNICAL ACCURACY CRITERIA:".to_string());
    context_parts.push("- Documentation should be well-structured".to_string());
    context_parts.push("- Code examples should be properly formatted".to_string());
    context_parts.push("- Headers should follow logical hierarchy".to_string());
    context_parts.push("- Links should be valid format".to_string());
    context_parts.push("- No critical formatting errors".to_string());

    // Decision thresholds
    context_parts.push("APPROVAL THRESHOLDS:".to_string());
    context_parts.push("- Format score >= 0.8".to_string());
    context_parts.push("- No critical issues".to_string());
    context_parts.push("- <= 2 major issues".to_string());

    // Current assessment
    let should_approve = input.format_score >= 0.8 && critical_issues == 0 && major_issues <= 2;
    context_parts.push(format!(
        "RECOMMENDED DECISION: {}",
        if should_approve { "APPROVE" } else { "DENY" }
    ));

    context_parts.join("\n")
}
