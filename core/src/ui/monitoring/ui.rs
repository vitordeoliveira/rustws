use axum::response::Html;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    auth::dto::User,
    error_handling::types::AppResult,
    ui::{
        Ui,
        engine::{TeraEngine, TeraRenderer},
        shared::layouts::BaseLayoutProps,
    },
};

/// System metrics data
#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub disk_usage_percent: f32,
    pub network_in_mbps: f32,
    pub network_out_mbps: f32,
    pub uptime_hours: u32,
    pub last_updated: DateTime<Utc>,
}

/// Service health status
#[derive(Debug, Clone, Serialize)]
pub struct ServiceHealth {
    pub service_name: String,
    pub status: ServiceStatus,
    pub response_time_ms: Option<u32>,
    pub last_check: DateTime<Utc>,
    pub uptime_percent: f32,
    pub endpoint_url: Option<String>,
}

/// Status of a monitored service
#[derive(Debug, Clone, Serialize)]
pub enum ServiceStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Alert/incident information
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub service: String,
    pub status: AlertStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert status
#[derive(Debug, Clone, Serialize)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
}

/// Log entry for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub metadata: Option<String>,
}

/// Log levels
#[derive(Debug, Clone, Serialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Performance metrics over time
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMetric {
    pub metric_name: String,
    pub current_value: f32,
    pub unit: String,
    pub trend: MetricTrend,
    pub threshold_warning: Option<f32>,
    pub threshold_critical: Option<f32>,
}

/// Metric trend direction
#[derive(Debug, Clone, Serialize)]
pub enum MetricTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// UI component for the monitoring dashboard page
#[derive(Debug, Serialize)]
pub struct MonitoringPageUi {
    layout: BaseLayoutProps,
    system_metrics: SystemMetrics,
    service_health: Vec<ServiceHealth>,
    recent_alerts: Vec<Alert>,
    recent_logs: Vec<LogEntry>,
    performance_metrics: Vec<PerformanceMetric>,
}

impl MonitoringPageUi {
    pub fn new(
        user: User,
        system_metrics: SystemMetrics,
        service_health: Vec<ServiceHealth>,
        recent_alerts: Vec<Alert>,
        recent_logs: Vec<LogEntry>,
        performance_metrics: Vec<PerformanceMetric>,
    ) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("Monitoring - RUSTWS Core")
                .description(
                    "Real-time system monitoring, health checks, and observability dashboard",
                )
                .keywords("monitoring, observability, health, metrics, alerts, logs, rustws")
                .user(Some(user)),
            system_metrics,
            service_health,
            recent_alerts,
            recent_logs,
            performance_metrics,
        }
    }
}

impl Ui for MonitoringPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("system_metrics", &self.system_metrics);
        context.insert("service_health", &self.service_health);
        context.insert("recent_alerts", &self.recent_alerts);
        context.insert("recent_logs", &self.recent_logs);
        context.insert("performance_metrics", &self.performance_metrics);
        tera.render_template("monitoring/index.html", &context)
    }
}

/// Generate mock monitoring data for demonstration
pub fn get_mock_monitoring_data() -> (
    SystemMetrics,
    Vec<ServiceHealth>,
    Vec<Alert>,
    Vec<LogEntry>,
    Vec<PerformanceMetric>,
) {
    let now = Utc::now();

    let system_metrics = SystemMetrics {
        cpu_usage_percent: 23.5,
        memory_usage_percent: 68.2,
        disk_usage_percent: 45.1,
        network_in_mbps: 12.8,
        network_out_mbps: 8.4,
        uptime_hours: 720, // 30 days
        last_updated: now,
    };

    let service_health = vec![
        ServiceHealth {
            service_name: "Lambda Functions".to_string(),
            status: ServiceStatus::Healthy,
            response_time_ms: Some(145),
            last_check: now - chrono::Duration::minutes(1),
            uptime_percent: 99.9,
            endpoint_url: Some("https://api.rustws.com/lambda".to_string()),
        },
        ServiceHealth {
            service_name: "Step Functions".to_string(),
            status: ServiceStatus::Healthy,
            response_time_ms: Some(89),
            last_check: now - chrono::Duration::minutes(1),
            uptime_percent: 99.7,
            endpoint_url: Some("https://api.rustws.com/step-functions".to_string()),
        },
        ServiceHealth {
            service_name: "API Gateway".to_string(),
            status: ServiceStatus::Warning,
            response_time_ms: Some(320),
            last_check: now - chrono::Duration::minutes(2),
            uptime_percent: 98.5,
            endpoint_url: Some("https://api.rustws.com".to_string()),
        },
        ServiceHealth {
            service_name: "Database".to_string(),
            status: ServiceStatus::Healthy,
            response_time_ms: Some(25),
            last_check: now - chrono::Duration::seconds(30),
            uptime_percent: 99.99,
            endpoint_url: None,
        },
        ServiceHealth {
            service_name: "Authentication".to_string(),
            status: ServiceStatus::Critical,
            response_time_ms: None,
            last_check: now - chrono::Duration::minutes(5),
            uptime_percent: 97.2,
            endpoint_url: Some("https://auth.rustws.com".to_string()),
        },
    ];

    let recent_alerts = vec![
        Alert {
            id: "alert-001".to_string(),
            title: "High Response Time".to_string(),
            description: "API Gateway response time exceeded 300ms threshold".to_string(),
            severity: AlertSeverity::Warning,
            service: "API Gateway".to_string(),
            status: AlertStatus::Active,
            created_at: now - chrono::Duration::minutes(15),
            resolved_at: None,
        },
        Alert {
            id: "alert-002".to_string(),
            title: "Authentication Service Down".to_string(),
            description: "Authentication service is not responding to health checks".to_string(),
            severity: AlertSeverity::Critical,
            service: "Authentication".to_string(),
            status: AlertStatus::Acknowledged,
            created_at: now - chrono::Duration::minutes(8),
            resolved_at: None,
        },
        Alert {
            id: "alert-003".to_string(),
            title: "Memory Usage High".to_string(),
            description: "System memory usage exceeded 65% threshold".to_string(),
            severity: AlertSeverity::Warning,
            service: "System".to_string(),
            status: AlertStatus::Resolved,
            created_at: now - chrono::Duration::hours(2),
            resolved_at: Some(now - chrono::Duration::minutes(30)),
        },
    ];

    let recent_logs = vec![
        LogEntry {
            id: "log-001".to_string(),
            timestamp: now - chrono::Duration::minutes(2),
            level: LogLevel::Error,
            service: "Authentication".to_string(),
            message: "Failed to connect to authentication provider".to_string(),
            metadata: Some(r#"{"error": "connection_timeout", "provider": "oauth2"}"#.to_string()),
        },
        LogEntry {
            id: "log-002".to_string(),
            timestamp: now - chrono::Duration::minutes(5),
            level: LogLevel::Warn,
            service: "API Gateway".to_string(),
            message: "Response time threshold exceeded for /api/lambda endpoint".to_string(),
            metadata: Some(r#"{"response_time_ms": 320, "threshold_ms": 300}"#.to_string()),
        },
        LogEntry {
            id: "log-003".to_string(),
            timestamp: now - chrono::Duration::minutes(8),
            level: LogLevel::Info,
            service: "Lambda Functions".to_string(),
            message: "Successfully compiled lambda function: data-processor".to_string(),
            metadata: Some(
                r#"{"function_name": "data-processor", "compile_time_ms": 1250}"#.to_string(),
            ),
        },
        LogEntry {
            id: "log-004".to_string(),
            timestamp: now - chrono::Duration::minutes(12),
            level: LogLevel::Info,
            service: "Step Functions".to_string(),
            message: "Workflow execution completed successfully".to_string(),
            metadata: Some(
                r#"{"workflow_id": "user-onboarding", "execution_time_ms": 2500}"#.to_string(),
            ),
        },
    ];

    let performance_metrics = vec![
        PerformanceMetric {
            metric_name: "Request Rate".to_string(),
            current_value: 1247.5,
            unit: "req/min".to_string(),
            trend: MetricTrend::Increasing,
            threshold_warning: Some(2000.0),
            threshold_critical: Some(3000.0),
        },
        PerformanceMetric {
            metric_name: "Error Rate".to_string(),
            current_value: 0.8,
            unit: "%".to_string(),
            trend: MetricTrend::Stable,
            threshold_warning: Some(2.0),
            threshold_critical: Some(5.0),
        },
        PerformanceMetric {
            metric_name: "Average Latency".to_string(),
            current_value: 180.2,
            unit: "ms".to_string(),
            trend: MetricTrend::Increasing,
            threshold_warning: Some(250.0),
            threshold_critical: Some(500.0),
        },
        PerformanceMetric {
            metric_name: "Throughput".to_string(),
            current_value: 450.8,
            unit: "MB/s".to_string(),
            trend: MetricTrend::Stable,
            threshold_warning: Some(800.0),
            threshold_critical: Some(1000.0),
        },
    ];

    (
        system_metrics,
        service_health,
        recent_alerts,
        recent_logs,
        performance_metrics,
    )
}
