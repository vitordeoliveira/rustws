//! Step Functions UI components

use axum::response::Html;
use serde::Serialize;

use crate::{
    auth::dto::User,
    business_logic::workflows::WorkflowSummary,
    error_handling::types::AppResult,
    ui::{
        Ui,
        engine::{TeraEngine, TeraRenderer},
        shared::layouts::BaseLayoutProps,
    },
};

/// Step Functions dashboard page UI
#[derive(Debug, Serialize)]
pub struct StepFunctionsPageUi {
    layout: BaseLayoutProps,
    workflows: Vec<WorkflowSummary>,
}

impl StepFunctionsPageUi {
    pub fn new(user: User, workflows: Vec<WorkflowSummary>) -> Self {
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
            workflows,
        }
    }
}

impl Ui for StepFunctionsPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("workflows", &self.workflows);
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
