//! Reports UI components for vulnerability analysis

use axum::response::Html;
use serde::{Deserialize, Serialize};

use crate::{
    auth::dto::User,
    error_handling::types::AppResult,
    ui::{
        Ui,
        engine::{TeraEngine, TeraRenderer},
        shared::layouts::BaseLayoutProps,
    },
};

/// Vulnerability report data structure matching vulnerability_reporter output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReport {
    pub report_id: String,
    pub execution_id: String,
    pub timestamp: String,
    pub executive_summary: ExecutiveSummary,
    pub detailed_findings: DetailedFindings,
    pub risk_assessment: RiskAssessment,
    pub remediation_plan: RemediationPlan,
    pub report_metadata: ReportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub security_rating: String,
    pub key_findings: Vec<String>,
    pub critical_vulnerabilities: u32,
    pub priority_recommendations: Vec<String>,
    pub business_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedFindings {
    pub vulnerability_breakdown: Vec<VulnerabilityFinding>,
    pub strategy_effectiveness: Vec<StrategyEffectiveness>,
    pub response_quality_analysis: ResponseQualityFindings,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityFinding {
    pub vulnerability_id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub exploitation_likelihood: String,
    pub potential_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEffectiveness {
    pub strategy: String,
    pub effectiveness: String,
    pub metrics: StrategyMetrics,
    pub defensive_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub success_rate_percent: f64,
    pub average_response_time_ms: u64,
    pub average_severity: f64,
    pub consistency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseQualityFindings {
    pub overall_quality: String,
    pub quality_issues: Vec<QualityIssue>,
    pub positive_indicators: Vec<String>,
    pub improvement_areas: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub category: String,
    pub description: String,
    pub frequency: u32,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_response_time_ms: u64,
    pub error_rates: ErrorRateMetrics,
    pub throughput_metrics: ThroughputMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRateMetrics {
    pub overall_error_rate: f64,
    pub network_error_rate: f64,
    pub server_error_rate: f64,
    pub timeout_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    pub requests_per_second: f64,
    pub max_concurrent_requests: u32,
    pub efficiency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_level: String,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_priorities: Vec<String>,
    pub residual_risk_assessment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_id: String,
    pub description: String,
    pub likelihood: String,
    pub impact: String,
    pub risk_score: f64,
    pub affected_stakeholders: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationPlan {
    pub short_term_actions: Vec<RemediationAction>,
    pub medium_term_actions: Vec<RemediationAction>,
    pub long_term_actions: Vec<RemediationAction>,
    pub success_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationAction {
    pub action_id: String,
    pub description: String,
    pub priority: String,
    pub estimated_effort: String,
    pub expected_impact: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub generated_at: String,
    pub report_version: String,
    pub generator: String,
    pub format_used: String,
    pub audience: String,
    pub classification: String,
}

/// Reports dashboard page UI
#[derive(Debug, Serialize)]
pub struct ReportsPageUi {
    layout: BaseLayoutProps,
    recent_reports: Vec<ReportSummary>,
    total_reports: u32,
}

/// Summary of a vulnerability report for dashboard display
#[derive(Debug, Clone, Serialize)]
pub struct ReportSummary {
    pub report_id: String,
    pub execution_id: String,
    pub generated_at: String,
    pub security_rating: String,
    pub critical_vulnerabilities: u32,
    pub overall_robustness_score: f64,
    pub tests_analyzed: u32,
}

impl ReportsPageUi {
    pub fn new(user: User, recent_reports: Vec<ReportSummary>, total_reports: u32) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("Vulnerability Reports - RUSTWS Core")
                .description(
                    "Comprehensive vulnerability assessment reports from adversarial AI alignment testing",
                )
                .keywords(
                    "vulnerability, security, ai safety, alignment, reports, analysis, rustws",
                )
                .user(Some(user)),
            recent_reports,
            total_reports,
        }
    }
}

impl Ui for ReportsPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("recent_reports", &self.recent_reports);
        context.insert("total_reports", &self.total_reports);
        tera.render_template("reports/index.html", &context)
    }
}

/// Individual vulnerability report page UI
#[derive(Debug, Serialize)]
pub struct ReportDetailPageUi {
    layout: BaseLayoutProps,
    report: VulnerabilityReport,
}

impl ReportDetailPageUi {
    pub fn new(user: User, report: VulnerabilityReport) -> Self {
        let title = format!("Report {} - RUSTWS Core", report.report_id);
        Self {
            layout: BaseLayoutProps::new()
                .title(&title)
                .description("Detailed vulnerability assessment report from adversarial AI alignment testing")
                .keywords("vulnerability, security, ai safety, alignment, report, analysis, rustws")
                .user(Some(user)),
            report,
        }
    }
}

impl Ui for ReportDetailPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("report", &self.report);
        tera.render_template("reports/detail.html", &context)
    }
}
