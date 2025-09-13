//! Step Functions UI components

use axum::response::Html;
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

/// Mock step function data for UI demonstration
#[derive(Debug, Serialize, Clone)]
pub struct StepFunction {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: StepFunctionStatus,
    pub definition: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Step function status
#[derive(Debug, Serialize, Clone)]
pub enum StepFunctionStatus {
    Active,
    Draft,
    Error,
}

/// Step Functions dashboard page UI
#[derive(Debug, Serialize)]
pub struct StepFunctionsPageUi {
    layout: BaseLayoutProps,
    step_functions: Vec<StepFunction>,
}

impl StepFunctionsPageUi {
    pub fn new(user: User, step_functions: Vec<StepFunction>) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("Step Functions - RUSTWS Core")
                .description(
                    "Manage and orchestrate serverless workflows with visual step functions",
                )
                .keywords(
                    "step functions, workflows, serverless, orchestration, state machines, rustws",
                )
                .user(Some(user)),
            step_functions,
        }
    }
}

impl Ui for StepFunctionsPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("step_functions", &self.step_functions);
        tera.render_template("step_functions/index.html", &context)
    }
}

/// Create step function page UI
#[derive(Debug, Serialize)]
pub struct CreateStepFunctionPageUi {
    layout: BaseLayoutProps,
}

impl CreateStepFunctionPageUi {
    pub fn new(user: User) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title("Create Step Function - RUSTWS Core")
                .description(
                    "Create a new serverless workflow with visual state machine definition",
                )
                .keywords("step functions, create, workflows, state machines, serverless, rustws")
                .user(Some(user)),
        }
    }
}

impl Ui for CreateStepFunctionPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let context = self.layout.to_context()?;
        tera.render_template("step_functions/create.html", &context)
    }
}

/// Edit step function page UI
#[derive(Debug, Serialize)]
pub struct EditStepFunctionPageUi {
    layout: BaseLayoutProps,
    step_function_name: String,
    current_definition: String,
}

impl EditStepFunctionPageUi {
    pub fn new(user: User, step_function_name: String, current_definition: String) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title(&format!(
                    "Edit Step Function: {} - RUSTWS Core",
                    step_function_name
                ))
                .description("Edit and update your serverless workflow state machine definition")
                .keywords("step functions, edit, update, workflows, state machines, rustws")
                .user(Some(user)),
            step_function_name,
            current_definition,
        }
    }
}

impl Ui for EditStepFunctionPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("step_function_name", &self.step_function_name);
        context.insert("current_definition", &self.current_definition);
        tera.render_template("step_functions/edit.html", &context)
    }
}

/// Generate mock step functions for UI demonstration
pub fn get_mock_step_functions() -> Vec<StepFunction> {
    vec![
        StepFunction {
            id: "sf_001".to_string(),
            name: "user_onboarding".to_string(),
            description: Some(
                "Complete user onboarding workflow with email verification and welcome sequence"
                    .to_string(),
            ),
            status: StepFunctionStatus::Active,
            definition: r#"{
  "Comment": "User onboarding workflow",
  "StartAt": "ValidateUser",
  "States": {
    "ValidateUser": {
      "Type": "Task",
      "Resource": "arn:aws:lambda:us-east-1:123456789012:function:ValidateUser",
      "Next": "SendWelcomeEmail"
    },
    "SendWelcomeEmail": {
      "Type": "Task", 
      "Resource": "arn:aws:lambda:us-east-1:123456789012:function:SendEmail",
      "End": true
    }
  }
}"#
            .to_string(),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            updated_at: "2024-01-20T14:45:00Z".to_string(),
        },
        StepFunction {
            id: "sf_002".to_string(),
            name: "order_processing".to_string(),
            description: Some(
                "E-commerce order processing pipeline with payment and inventory checks"
                    .to_string(),
            ),
            status: StepFunctionStatus::Active,
            definition: r#"{
  "Comment": "Order processing workflow",
  "StartAt": "CheckInventory",
  "States": {
    "CheckInventory": {
      "Type": "Task",
      "Resource": "arn:aws:lambda:us-east-1:123456789012:function:CheckInventory",
      "Next": "ProcessPayment"
    },
    "ProcessPayment": {
      "Type": "Task",
      "Resource": "arn:aws:lambda:us-east-1:123456789012:function:ProcessPayment",
      "Next": "FulfillOrder"
    },
    "FulfillOrder": {
      "Type": "Task",
      "Resource": "arn:aws:lambda:us-east-1:123456789012:function:FulfillOrder",
      "End": true
    }
  }
}"#
            .to_string(),
            created_at: "2024-01-10T08:15:00Z".to_string(),
            updated_at: "2024-01-18T16:20:00Z".to_string(),
        },
        StepFunction {
            id: "sf_003".to_string(),
            name: "data_pipeline".to_string(),
            description: Some(
                "Daily data processing pipeline with ETL operations and notifications".to_string(),
            ),
            status: StepFunctionStatus::Draft,
            definition: r#"{
  "Comment": "Data processing pipeline",
  "StartAt": "ExtractData",
  "States": {
    "ExtractData": {
      "Type": "Task",
      "Resource": "arn:aws:lambda:us-east-1:123456789012:function:ExtractData",
      "Next": "TransformData"
    },
    "TransformData": {
      "Type": "Task",
      "Resource": "arn:aws:lambda:us-east-1:123456789012:function:TransformData",
      "Next": "LoadData"
    },
    "LoadData": {
      "Type": "Task",
      "Resource": "arn:aws:lambda:us-east-1:123456789012:function:LoadData",
      "End": true
    }
  }
}"#
            .to_string(),
            created_at: "2024-01-22T12:00:00Z".to_string(),
            updated_at: "2024-01-22T12:00:00Z".to_string(),
        },
    ]
}
