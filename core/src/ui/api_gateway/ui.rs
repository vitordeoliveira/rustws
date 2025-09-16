use axum::response::Html;
use serde::Serialize;

use crate::{
    auth::dto::User,
    business_logic::api_gateway::ApiGatewaySummary,
    error_handling::types::AppResult,
    ui::{
        Ui,
        engine::{TeraEngine, TeraRenderer},
        shared::layouts::BaseLayoutProps,
    },
};

/// UI component for the API Gateway dashboard page
#[derive(Debug, Serialize)]
pub struct ApiGatewayPageUi {
    layout: BaseLayoutProps,
    api_gateways: Vec<ApiGatewaySummary>,
}

impl ApiGatewayPageUi {
    pub fn new(user: User, api_gateways: Vec<ApiGatewaySummary>) -> Self {
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
