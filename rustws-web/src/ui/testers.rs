/// Template testers for Tera (used by init_ui)
use tera::{Error as TeraError, Value};
use tracing::instrument;

/// Test if a value is Some (not null)
#[instrument(skip_all, fields(operation = "is_some"))]
pub fn is_some(value: Option<&Value>, _args: &[Value]) -> Result<bool, TeraError> {
    match value {
        Some(Value::Null) => Ok(false),
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
