//! Documentation UI components for AI-generated documentation

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

/// AI-generated documentation data structure matching ai_doc_generator_v2 output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIDocumentation {
    pub documentation: String,
    pub content_type: String,
    pub target_audience: String,
    pub generation_info: GenerationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationInfo {
    pub generation_method: String,
    pub confidence: f64,
    pub word_count: u32,
    pub sections_included: Vec<String>,
}

/// Documentation summary for listing page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationSummary {
    pub execution_id: String,
    pub content_type: String,
    pub target_audience: String,
    pub generated_at: String,
    pub confidence: f64,
    pub word_count: u32,
    pub preview: String, // First few lines of documentation
}

/// Documentation page UI struct
#[derive(Debug, Serialize)]
pub struct DocumentationPageUi {
    pub layout: BaseLayoutProps,
    pub documentation_list: Vec<DocumentationSummary>,
    pub total_docs: usize,
    pub avg_confidence: f64,
    pub total_words: u32,
}

/// Individual documentation view UI struct
#[derive(Debug, Serialize)]
pub struct DocumentationViewUi {
    pub layout: BaseLayoutProps,
    pub documentation: AIDocumentation,
    pub execution_id: String,
    pub generated_at: String,
}

impl DocumentationPageUi {
    pub fn new(user: User, documentation_list: Vec<DocumentationSummary>) -> Self {
        let total_docs = documentation_list.len();
        let avg_confidence = if total_docs > 0 {
            documentation_list.iter().map(|d| d.confidence).sum::<f64>() / total_docs as f64
        } else {
            0.0
        };
        let total_words = documentation_list.iter().map(|d| d.word_count).sum::<u32>();

        Self {
            layout: BaseLayoutProps::new()
                .title("AI Documentation")
                .description("AI-generated documentation from your content")
                .keywords("documentation, ai, generated, content, markdown")
                .user(Some(user)),
            documentation_list,
            total_docs,
            avg_confidence,
            total_words,
        }
    }
}

impl Ui for DocumentationPageUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("documentation_list", &self.documentation_list);
        context.insert("total_docs", &self.total_docs);
        context.insert("avg_confidence", &self.avg_confidence);
        context.insert("total_words", &self.total_words);
        tera.render_template("documentation/index.html", &context)
    }
}

impl DocumentationViewUi {
    pub fn new(
        user: User,
        documentation: AIDocumentation,
        execution_id: String,
        generated_at: String,
    ) -> Self {
        let title = format!("Documentation - {}", documentation.content_type);
        Self {
            layout: BaseLayoutProps::new()
                .title(&title)
                .description("AI-generated documentation viewer")
                .keywords("documentation, ai, generated, content, markdown, viewer")
                .user(Some(user)),
            documentation,
            execution_id,
            generated_at,
        }
    }
}

impl Ui for DocumentationViewUi {
    fn render_html(self, tera: &TeraEngine) -> AppResult<Html<String>> {
        let mut context = self.layout.to_context()?;
        context.insert("documentation", &self.documentation);
        context.insert("execution_id", &self.execution_id);
        context.insert("generated_at", &self.generated_at);
        tera.render_template("documentation/view.html", &context)
    }
}
