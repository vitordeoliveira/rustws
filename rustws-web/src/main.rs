use rustws::{app, error_handling::AppResult};
use tracing::instrument;

#[tokio::main]
#[instrument(skip_all, fields(service = "matrix"))]
async fn main() -> AppResult<()> {
    app::App::new().await.serve().await
}
