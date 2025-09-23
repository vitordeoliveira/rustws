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
    workflow: WorkflowSummary,
    definition: String,
}

impl EditStepFunctionPageUi {
    pub fn new(user: User, workflow: WorkflowSummary, definition: String) -> Self {
        Self {
            layout: BaseLayoutProps::new()
                .title(&format!("Edit {} - RUSTWS Core", workflow.name))
                .description("Edit serverless workflow configuration and state machine definition")
                .keywords("step functions, edit, workflows, state machines, serverless, rustws")
                .user(Some(user)),
            workflow,
            definition,
        }
    }
}

impl Ui for EditStepFunctionPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("workflow", &self.workflow);
        context.insert("definition", &self.definition);

        // Generate workflow graph SVG
        let definition_clone = self.definition.clone();
        let workflow_graph = Self::render_workflow_graph_static(&definition_clone);
        context.insert("workflow_graph", &workflow_graph);

        tera.render_template("step_functions/edit.html", &context)
    }
}

impl EditStepFunctionPageUi {
    /// Generate SVG workflow graph from the workflow definition (static method)
    pub fn render_workflow_graph_static(definition: &str) -> String {
        Self::render_workflow_graph_impl(definition)
    }

    /// Generate SVG workflow graph from the workflow definition
    pub fn render_workflow_graph(&self) -> String {
        Self::render_workflow_graph_impl(&self.definition)
    }

    /// Internal implementation for workflow graph generation
    fn render_workflow_graph_impl(definition: &str) -> String {
        // Parse the workflow definition
        let workflow: serde_json::Value = match serde_json::from_str(definition) {
            Ok(w) => w,
            Err(_) => return Self::render_error_graph_static("Invalid JSON definition"),
        };

        let states = match workflow.get("States") {
            Some(s) => s,
            None => return Self::render_error_graph_static("No States found in definition"),
        };

        let start_at = match workflow.get("StartAt") {
            Some(s) => s.as_str().unwrap_or(""),
            None => return Self::render_error_graph_static("No StartAt found in definition"),
        };

        // Generate graph nodes and edges
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_positions = std::collections::HashMap::<String, (i32, i32)>::new();

        // Calculate positions for nodes (simple grid layout)
        let state_names: Vec<String> = states
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.to_string())
            .collect();

        let cols = (state_names.len() as f64).sqrt().ceil() as i32;
        let node_width = 120i32;
        let node_height = 60i32;
        let spacing_x = 200i32;
        let spacing_y = 100i32;

        for (i, state_name) in state_names.iter().enumerate() {
            let row = i as i32 / cols;
            let col = i as i32 % cols;
            let x = 50 + col * spacing_x;
            let y = 50 + row * spacing_y;

            node_positions.insert(state_name.clone(), (x, y));

            let state = &states[state_name];
            let state_type = state
                .get("Type")
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown");
            let is_start = state_name == start_at;
            let is_end = state.get("End").and_then(|e| e.as_bool()).unwrap_or(false)
                || state_type == "Succeed"
                || state_type == "Fail";

            // Determine node color based on type and status
            let (fill_color, stroke_color, text_color) = match state_type {
                "Task" => {
                    if is_start {
                        ("#10b981", "#059669", "#ffffff")
                    } else {
                        ("#3b82f6", "#2563eb", "#ffffff")
                    }
                }
                "Choice" => ("#f59e0b", "#d97706", "#ffffff"),
                "Wait" => ("#8b5cf6", "#7c3aed", "#ffffff"),
                "Succeed" => ("#10b981", "#059669", "#ffffff"),
                "Fail" => ("#ef4444", "#dc2626", "#ffffff"),
                _ => ("#6b7280", "#4b5563", "#ffffff"),
            };

            let node_svg = format!(
                "<g>\
                    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"8\" \
                          fill=\"{}\" stroke=\"{}\" stroke-width=\"2\"/>\
                    <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" \
                          font-family=\"system-ui, -apple-system, sans-serif\" font-size=\"12\" \
                          font-weight=\"600\" fill=\"{}\">{}</text>\
                    <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" \
                          font-family=\"system-ui, -apple-system, sans-serif\" font-size=\"10\" \
                          fill=\"{}\">{}</text>\
                </g>",
                x,
                y,
                node_width,
                node_height,
                fill_color,
                stroke_color,
                x + node_width / 2,
                y + node_height / 2 - 8,
                text_color,
                state_name,
                x + node_width / 2,
                y + node_height / 2 + 8,
                text_color,
                state_type
            );

            nodes.push(node_svg);

            // Add start indicator
            if is_start {
                let start_svg = format!(
                    "<circle cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"#10b981\" stroke=\"#ffffff\" stroke-width=\"2\"/>\
                     <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" \
                           font-family=\"system-ui, -apple-system, sans-serif\" font-size=\"8\" \
                           font-weight=\"bold\" fill=\"#065f46\">START</text>",
                    x - 10,
                    y + node_height / 2,
                    x - 10,
                    y + node_height / 2 + 20
                );
                nodes.push(start_svg);
            }
        }

        // Generate edges
        for state_name in &state_names {
            let state = &states[state_name];

            // Handle Next transitions
            if let Some(next) = state.get("Next").and_then(|n| n.as_str()) {
                if let Some((from_x, from_y)) = node_positions.get(state_name) {
                    if let Some((to_x, to_y)) = node_positions.get(next) {
                        let edge_svg = Self::render_edge_static(
                            *from_x + node_width,
                            *from_y + node_height / 2,
                            *to_x,
                            *to_y + node_height / 2,
                            "Next",
                        );
                        edges.push(edge_svg);
                    }
                }
            }

            // Handle Choice transitions
            if let Some(choices) = state.get("Choices").and_then(|c| c.as_array()) {
                for choice in choices {
                    if let Some(next) = choice.get("Next").and_then(|n| n.as_str()) {
                        if let Some((from_x, from_y)) = node_positions.get(state_name) {
                            if let Some((to_x, to_y)) = node_positions.get(next) {
                                let edge_svg = Self::render_edge_static(
                                    *from_x + node_width,
                                    *from_y + node_height / 2,
                                    *to_x,
                                    *to_y + node_height / 2,
                                    "Choice",
                                );
                                edges.push(edge_svg);
                            }
                        }
                    }
                }
            }

            // Handle Default transitions
            if let Some(default) = state.get("Default").and_then(|d| d.as_str()) {
                if let Some((from_x, from_y)) = node_positions.get(state_name) {
                    if let Some((to_x, to_y)) = node_positions.get(default) {
                        let edge_svg = Self::render_edge_static(
                            *from_x + node_width,
                            *from_y + node_height / 2,
                            *to_x,
                            *to_y + node_height / 2,
                            "Default",
                        );
                        edges.push(edge_svg);
                    }
                }
            }
        }

        // Calculate SVG dimensions
        let max_x = node_positions
            .values()
            .map(|(x, _)| *x + node_width)
            .max()
            .unwrap_or(200)
            + 50;
        let max_y = node_positions
            .values()
            .map(|(_, y)| *y + node_height)
            .max()
            .unwrap_or(200)
            + 50;

        format!(
            "<svg width=\"100%\" height=\"400\" viewBox=\"0 0 {} {}\" \
                 style=\"border: 1px solid #e5e7eb; border-radius: 8px; background: #f9fafb;\">\
                <defs>\
                    <marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"7\" \
                            refX=\"9\" refY=\"3.5\" orient=\"auto\">\
                        <polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#6b7280\"/>\
                    </marker>\
                </defs>\
                {}\
                {}\
            </svg>",
            max_x,
            max_y,
            edges.join(""),
            nodes.join("")
        )
    }

    /// Render an edge between two nodes (static method)
    fn render_edge_static(from_x: i32, from_y: i32, to_x: i32, to_y: i32, label: &str) -> String {
        let mid_x = (from_x + to_x) / 2;
        let mid_y = (from_y + to_y) / 2;

        format!(
            "<g>\
                <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" \
                      stroke=\"#6b7280\" stroke-width=\"2\" marker-end=\"url(#arrowhead)\"/>\
                <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" \
                      font-family=\"system-ui, -apple-system, sans-serif\" font-size=\"10\" \
                      fill=\"#6b7280\" font-weight=\"500\">{}</text>\
            </g>",
            from_x,
            from_y,
            to_x,
            to_y,
            mid_x,
            mid_y - 5,
            label
        )
    }

    /// Render an error graph when workflow parsing fails (static method)
    fn render_error_graph_static(error_msg: &str) -> String {
        format!(
            "<svg width=\"100%\" height=\"200\" viewBox=\"0 0 400 200\" \
                 style=\"border: 1px solid #e5e7eb; border-radius: 8px; background: #f9fafb;\">\
                <rect x=\"50\" y=\"50\" width=\"300\" height=\"100\" rx=\"8\" \
                      fill=\"#fef2f2\" stroke=\"#fecaca\" stroke-width=\"2\"/>\
                <text x=\"200\" y=\"100\" text-anchor=\"middle\" dominant-baseline=\"middle\" \
                      font-family=\"system-ui, -apple-system, sans-serif\" font-size=\"14\" \
                      font-weight=\"600\" fill=\"#dc2626\">⚠️ Workflow Graph Error</text>\
                <text x=\"200\" y=\"120\" text-anchor=\"middle\" dominant-baseline=\"middle\" \
                      font-family=\"system-ui, -apple-system, sans-serif\" font-size=\"12\" \
                      fill=\"#6b7280\">{}</text>\
            </svg>",
            error_msg
        )
    }
}
