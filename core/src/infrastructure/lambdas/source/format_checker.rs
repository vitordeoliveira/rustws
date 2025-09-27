//! Format Checker Lambda - Validates document format and structure
//!
//! This lambda validates that generated documentation follows proper formatting,
//! structure, and style guidelines.

use lambda_fn_macro::lambda_fn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ===== INPUT/OUTPUT TYPES =====

/// Input for format checking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormatCheckerRequest {
    /// The documentation content to check
    pub documentation: String,
    /// Type of content being checked (e.g., "markdown", "api_reference", "user_guide")
    pub content_type: String,
    /// Format requirements to apply
    pub format_requirements: FormatRequirements,
    /// Additional context for validation
    pub context: Option<String>,
}

/// Format requirements configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormatRequirements {
    /// Required sections that must be present
    pub required_sections: Vec<String>,
    /// Maximum line length allowed
    pub max_line_length: Option<u32>,
    /// Whether code blocks must have language specified
    pub require_code_block_language: Option<bool>,
    /// Whether headers must follow hierarchy (h1 -> h2 -> h3)
    pub enforce_header_hierarchy: Option<bool>,
    /// Whether links must be valid format
    pub validate_links: Option<bool>,
}

/// Format check results
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
    /// The original documentation that was checked
    pub documentation: String,
}

/// Individual format issue
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormatIssue {
    /// Type of issue (e.g., "missing_section", "invalid_markdown", "long_line")
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

// ===== LAMBDA IMPLEMENTATION =====

#[lambda_fn(features = [env])]
fn format_checker_lambda(input: FormatCheckerRequest) -> FormatCheckerResponse {
    // Starting format validation

    // Validate input
    if input.documentation.trim().is_empty() {
        return create_empty_doc_response(&input);
    }

    // Parse document structure
    let doc_analysis = analyze_document_structure(&input.documentation);

    // Run format validations
    let mut issues = Vec::new();

    // Check required sections
    check_required_sections(&input, &doc_analysis, &mut issues);

    // Check markdown formatting
    check_markdown_formatting(&input, &doc_analysis, &mut issues);

    // Check line lengths
    check_line_lengths(&input, &doc_analysis, &mut issues);

    // Check code blocks
    check_code_blocks(&input, &doc_analysis, &mut issues);

    // Check header hierarchy
    check_header_hierarchy(&input, &doc_analysis, &mut issues);

    // Check links
    check_links(&input, &doc_analysis, &mut issues);

    // Calculate format score
    let format_score = calculate_format_score(&issues, &doc_analysis);
    let is_valid = format_score >= 0.8 && !has_critical_issues(&issues);

    // Format validation completed

    FormatCheckerResponse {
        is_valid,
        format_score,
        issues,
        sections_found: doc_analysis.sections,
        format_stats: doc_analysis.stats,
        content_type: input.content_type,
        documentation: input.documentation,
    }
}

// ===== VALIDATION FUNCTIONS =====

/// Document structure analysis
#[derive(Debug)]
struct DocumentAnalysis {
    sections: Vec<String>,
    headers: Vec<Header>,
    code_blocks: Vec<CodeBlock>,
    links: Vec<Link>,
    lines: Vec<String>,
    stats: FormatStats,
}

#[derive(Debug)]
struct Header {
    level: u32,
    text: String,
    line_number: u32,
}

#[derive(Debug)]
struct CodeBlock {
    language: Option<String>,
    content: String,
    line_number: u32,
}

#[derive(Debug)]
struct Link {
    text: String,
    url: String,
    line_number: u32,
}

/// Analyze document structure and extract components
fn analyze_document_structure(content: &str) -> DocumentAnalysis {
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut sections = Vec::new();
    let mut headers = Vec::new();
    let mut code_blocks = Vec::new();
    let mut links = Vec::new();

    let mut in_code_block = false;
    let mut current_code_block: Option<CodeBlock> = None;

    for (line_num, line) in lines.iter().enumerate() {
        let line_number = (line_num + 1) as u32;

        // Check for code block boundaries
        if line.trim().starts_with("```") {
            if in_code_block {
                // End code block
                if let Some(block) = current_code_block.take() {
                    code_blocks.push(block);
                }
                in_code_block = false;
            } else {
                // Start code block
                let language = line
                    .trim()
                    .strip_prefix("```")
                    .map(|s| s.trim().to_string());
                current_code_block = Some(CodeBlock {
                    language: if language.as_ref().map_or(true, |s| s.is_empty()) {
                        None
                    } else {
                        language
                    },
                    content: String::new(),
                    line_number,
                });
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            if let Some(ref mut block) = current_code_block {
                if !block.content.is_empty() {
                    block.content.push('\n');
                }
                block.content.push_str(line);
            }
            continue;
        }

        // Check for headers
        if line.trim().starts_with('#') {
            let level = line.chars().take_while(|&c| c == '#').count() as u32;
            let text = line.trim_start_matches('#').trim().to_string();
            headers.push(Header {
                level,
                text: text.clone(),
                line_number,
            });
            sections.push(text.to_lowercase().replace(' ', "_"));
        }

        // Check for links
        if line.contains("[") && line.contains("](") {
            // Simple link detection - in real implementation would use proper regex
            if let Some(start) = line.find("[") {
                if let Some(middle) = line[start..].find("](") {
                    if let Some(end) = line[start + middle..].find(")") {
                        let text = &line[start + 1..start + middle];
                        let url_start = start + middle + 2;
                        let url_end = start + middle + end;
                        let url = &line[url_start..url_end];
                        links.push(Link {
                            text: text.to_string(),
                            url: url.to_string(),
                            line_number,
                        });
                    }
                }
            }
        }
    }

    // Calculate statistics
    let total_lines = lines.len() as u32;
    let header_count = headers.len() as u32;
    let code_block_count = code_blocks.len() as u32;
    let link_count = links.len() as u32;

    let line_lengths: Vec<usize> = lines.iter().map(|l| l.len()).collect();
    let avg_line_length = if !line_lengths.is_empty() {
        line_lengths.iter().sum::<usize>() as f32 / line_lengths.len() as f32
    } else {
        0.0
    };
    let max_line_length = line_lengths.iter().max().copied().unwrap_or(0) as u32;

    DocumentAnalysis {
        sections,
        headers,
        code_blocks,
        links,
        lines,
        stats: FormatStats {
            total_lines,
            header_count,
            code_block_count,
            link_count,
            avg_line_length,
            max_line_length,
        },
    }
}

/// Check if required sections are present
fn check_required_sections(
    input: &FormatCheckerRequest,
    analysis: &DocumentAnalysis,
    issues: &mut Vec<FormatIssue>,
) {
    for required_section in &input.format_requirements.required_sections {
        let section_found = analysis
            .sections
            .iter()
            .any(|s| s.to_lowercase().contains(&required_section.to_lowercase()));

        if !section_found {
            issues.push(FormatIssue {
                issue_type: "missing_section".to_string(),
                description: format!("Required section '{}' is missing", required_section),
                line_number: None,
                severity: IssueSeverity::Major,
            });
        }
    }
}

/// Check markdown formatting
fn check_markdown_formatting(
    _input: &FormatCheckerRequest,
    analysis: &DocumentAnalysis,
    issues: &mut Vec<FormatIssue>,
) {
    // Check for basic markdown issues
    for (line_num, line) in analysis.lines.iter().enumerate() {
        let line_number = (line_num + 1) as u32;

        // Check for headers without space after #
        if line.trim().starts_with('#')
            && !line.trim().starts_with("# ")
            && !line.trim().starts_with("##")
        {
            if let Some(first_non_hash) = line.trim().chars().skip_while(|&c| c == '#').next() {
                if first_non_hash != ' ' {
                    issues.push(FormatIssue {
                        issue_type: "header_formatting".to_string(),
                        description: "Headers should have a space after #".to_string(),
                        line_number: Some(line_number),
                        severity: IssueSeverity::Minor,
                    });
                }
            }
        }
    }
}

/// Check line lengths
fn check_line_lengths(
    input: &FormatCheckerRequest,
    analysis: &DocumentAnalysis,
    issues: &mut Vec<FormatIssue>,
) {
    if let Some(max_length) = input.format_requirements.max_line_length {
        for (line_num, line) in analysis.lines.iter().enumerate() {
            if line.len() > max_length as usize {
                issues.push(FormatIssue {
                    issue_type: "long_line".to_string(),
                    description: format!(
                        "Line exceeds maximum length of {} characters",
                        max_length
                    ),
                    line_number: Some((line_num + 1) as u32),
                    severity: IssueSeverity::Minor,
                });
            }
        }
    }
}

/// Check code blocks
fn check_code_blocks(
    input: &FormatCheckerRequest,
    analysis: &DocumentAnalysis,
    issues: &mut Vec<FormatIssue>,
) {
    if input
        .format_requirements
        .require_code_block_language
        .unwrap_or(false)
    {
        for block in &analysis.code_blocks {
            if block.language.is_none() {
                issues.push(FormatIssue {
                    issue_type: "missing_code_language".to_string(),
                    description: "Code block missing language specification".to_string(),
                    line_number: Some(block.line_number),
                    severity: IssueSeverity::Minor,
                });
            }
        }
    }
}

/// Check header hierarchy
fn check_header_hierarchy(
    input: &FormatCheckerRequest,
    analysis: &DocumentAnalysis,
    issues: &mut Vec<FormatIssue>,
) {
    if input
        .format_requirements
        .enforce_header_hierarchy
        .unwrap_or(false)
    {
        let mut prev_level = 0;
        for header in &analysis.headers {
            if header.level > prev_level + 1 && prev_level > 0 {
                issues.push(FormatIssue {
                    issue_type: "header_hierarchy".to_string(),
                    description: format!("Header level {} skips intermediate levels", header.level),
                    line_number: Some(header.line_number),
                    severity: IssueSeverity::Minor,
                });
            }
            prev_level = header.level;
        }
    }
}

/// Check links
fn check_links(
    input: &FormatCheckerRequest,
    analysis: &DocumentAnalysis,
    issues: &mut Vec<FormatIssue>,
) {
    if input.format_requirements.validate_links.unwrap_or(false) {
        for link in &analysis.links {
            // Basic URL validation
            if !link.url.starts_with("http")
                && !link.url.starts_with("#")
                && !link.url.starts_with("/")
            {
                issues.push(FormatIssue {
                    issue_type: "invalid_link".to_string(),
                    description: format!("Link '{}' may not be valid", link.url),
                    line_number: Some(link.line_number),
                    severity: IssueSeverity::Minor,
                });
            }
        }
    }
}

// ===== HELPER FUNCTIONS =====

/// Calculate overall format score
fn calculate_format_score(issues: &[FormatIssue], analysis: &DocumentAnalysis) -> f64 {
    if analysis.stats.total_lines == 0 {
        return 0.0;
    }

    let mut penalty = 0.0;

    for issue in issues {
        penalty += match issue.severity {
            IssueSeverity::Critical => 0.5,
            IssueSeverity::Major => 0.2,
            IssueSeverity::Minor => 0.05,
            IssueSeverity::Suggestion => 0.01,
        };
    }

    // Base score starts at 1.0, subtract penalties
    let score: f64 = 1.0 - penalty;
    score.max(0.0).min(1.0)
}

/// Check if there are any critical issues
fn has_critical_issues(issues: &[FormatIssue]) -> bool {
    issues
        .iter()
        .any(|issue| matches!(issue.severity, IssueSeverity::Critical))
}

/// Create response for empty document
fn create_empty_doc_response(input: &FormatCheckerRequest) -> FormatCheckerResponse {
    FormatCheckerResponse {
        is_valid: false,
        format_score: 0.0,
        issues: vec![FormatIssue {
            issue_type: "empty_document".to_string(),
            description: "Document is empty or contains only whitespace".to_string(),
            line_number: None,
            severity: IssueSeverity::Critical,
        }],
        sections_found: vec![],
        format_stats: FormatStats {
            total_lines: 0,
            header_count: 0,
            code_block_count: 0,
            link_count: 0,
            avg_line_length: 0.0,
            max_line_length: 0,
        },
        content_type: input.content_type.clone(),
        documentation: input.documentation.clone(),
    }
}
