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

/// Represents an API Gateway endpoint
#[derive(Debug, Clone, Serialize)]
pub struct ApiGateway {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: ApiGatewayStatus,
    pub stage: String,
    pub base_url: String,
    pub endpoints_count: u32,
    pub last_deployment: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Status of an API Gateway
#[derive(Debug, Clone, Serialize)]
pub enum ApiGatewayStatus {
    Active,
    Inactive,
    Deploying,
    Warning,
}

/// UI component for the API Gateway dashboard page
#[derive(Debug, Serialize)]
pub struct ApiGatewayPageUi {
    layout: BaseLayoutProps,
    api_gateways: Vec<ApiGateway>,
}

impl ApiGatewayPageUi {
    pub fn new(user: User, api_gateways: Vec<ApiGateway>) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("API Gateway - RUSTWS Core")
                .description(
                    "Manage and monitor API gateways, endpoints, and routing configurations",
                )
                .keywords("api gateway, endpoints, routing, microservices, api management, rustws")
                .user(Some(user)),
            api_gateways,
        }
    }
}

impl Ui for ApiGatewayPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("api_gateways", &self.api_gateways);
        tera.render_template("api_gateway/index.html", &context)
    }
}

/// Generate mock API Gateway data for demonstration
pub fn get_mock_api_gateways() -> Vec<ApiGateway> {
    let now = Utc::now();

    vec![
        ApiGateway {
            id: "api-gw-1".to_string(),
            name: "Main API Gateway".to_string(),
            description: "Primary API gateway handling all public endpoints".to_string(),
            status: ApiGatewayStatus::Active,
            stage: "production".to_string(),
            base_url: "https://api.rustws.com".to_string(),
            endpoints_count: 24,
            last_deployment: now - chrono::Duration::hours(2),
            created_at: now - chrono::Duration::days(30),
        },
        ApiGateway {
            id: "api-gw-2".to_string(),
            name: "Internal Services Gateway".to_string(),
            description: "Gateway for internal microservices communication".to_string(),
            status: ApiGatewayStatus::Active,
            stage: "production".to_string(),
            base_url: "https://internal-api.rustws.com".to_string(),
            endpoints_count: 18,
            last_deployment: now - chrono::Duration::days(1),
            created_at: now - chrono::Duration::days(15),
        },
        ApiGateway {
            id: "api-gw-3".to_string(),
            name: "Development Gateway".to_string(),
            description: "API gateway for development and testing environments".to_string(),
            status: ApiGatewayStatus::Deploying,
            stage: "development".to_string(),
            base_url: "https://dev-api.rustws.com".to_string(),
            endpoints_count: 12,
            last_deployment: now - chrono::Duration::minutes(15),
            created_at: now - chrono::Duration::days(5),
        },
        ApiGateway {
            id: "api-gw-4".to_string(),
            name: "Analytics Gateway".to_string(),
            description: "Specialized gateway for analytics and reporting endpoints".to_string(),
            status: ApiGatewayStatus::Warning,
            stage: "production".to_string(),
            base_url: "https://analytics-api.rustws.com".to_string(),
            endpoints_count: 8,
            last_deployment: now - chrono::Duration::hours(6),
            created_at: now - chrono::Duration::days(45),
        },
        ApiGateway {
            id: "api-gw-5".to_string(),
            name: "Legacy Gateway".to_string(),
            description: "Gateway for legacy API endpoints (deprecated)".to_string(),
            status: ApiGatewayStatus::Inactive,
            stage: "staging".to_string(),
            base_url: "https://legacy-api.rustws.com".to_string(),
            endpoints_count: 6,
            last_deployment: now - chrono::Duration::days(7),
            created_at: now - chrono::Duration::days(90),
        },
    ]
}
