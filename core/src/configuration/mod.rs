//! Configuration management

pub mod config;

pub use config::*;

use std::env;
use tracing::instrument;

/// Get required environment variable, panic if missing
fn get_required_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("❌ Required environment variable '{}' not found", key))
}

/// Load configuration from environment variables (via .env file)
#[instrument(skip_all, fields(operation = "load_configuration"))]
pub fn load_config() -> AppConfig {
    // Load .env file into environment variables
    if let Err(e) = dotenv::dotenv() {
        println!("ℹ️  No .env file found or error loading it: {}", e);
        println!("ℹ️  Will use system environment variables only");
    } else {
        println!("📁 Loaded .env file successfully");
    }

    // Build config from required environment variables
    let server_host = get_required_env("SERVER_HOST");

    let server_port = get_required_env("SERVER_PORT")
        .parse::<u16>()
        .unwrap_or_else(|e| panic!("❌ Invalid SERVER_PORT value: {}", e));

    let database_url = get_required_env("DATABASE_URL");
    let log_level = get_required_env("LOG_LEVEL");
    let log_format = get_required_env("LOG_FORMAT");
    let ui_template_path = get_required_env("UI_TEMPLATE_PATH");

    let config = AppConfig {
        server: ServerConfig {
            host: server_host,
            port: server_port,
        },
        database: DatabaseConfig { url: database_url },
        logging: LoggingConfig {
            level: log_level,
            format: log_format,
            file: None,
        },
        ui: UIConfig {
            template_path: ui_template_path,
        },
    };

    println!(
        "🎯 Configuration loaded: server={}:{}, log_level={}",
        config.server.host, config.server.port, config.logging.level
    );

    config
}
