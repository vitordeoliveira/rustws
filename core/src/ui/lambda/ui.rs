//! Lambda domain UI logic

use crate::{
    auth::dto::User,
    business_logic::lambdas::LambdaSummary,
    error_handling::types::AppResult,
    infrastructure::lambdas::{LambdaExecutionEntry, LambdasMetrics},
    ui::{TeraEngine, TeraRenderer, Ui, shared::layouts::BaseLayoutProps},
};
use axum::response::Html;
use serde::Serialize;

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
}

impl LambdaMetricsPageUi {
    pub fn new(user: User, lambda_name: String, executions: Vec<LambdaExecutionEntry>) -> Self {
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
        tera.render_template("lambda/metrics.html", &context)
    }
}
