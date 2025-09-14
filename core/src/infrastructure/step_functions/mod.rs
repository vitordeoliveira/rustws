use std::path::{Path, PathBuf};

pub mod workflow;

/// Step Functions storage implementation
///
/// Manages workflow definitions stored as JSON files in the filesystem.
/// Similar to LambdaStorage but for step function workflows.
#[derive(Debug, Clone)]
pub struct StepFunctionStorage {
    /// Base path for step functions infrastructure
    base_path: PathBuf,
}

impl StepFunctionStorage {
    /// Create a new StepFunctionStorage with default path
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("src/infrastructure/step_functions"),
        }
    }

    /// Create a new StepFunctionStorage with custom base path
    pub fn with_base_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            base_path: path.as_ref().to_path_buf(),
        }
    }

    /// Get the workflows directory path
    pub fn workflows_dir(&self) -> PathBuf {
        self.base_path.join("workflows")
    }
}

impl Default for StepFunctionStorage {
    fn default() -> Self {
        Self::new()
    }
}
