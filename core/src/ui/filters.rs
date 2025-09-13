//! Tera Template Engine Utilities
//!
//! This module provides helper functions for working with Tera templates.

use std::collections::HashMap;
use tera::{Error as TeraError, Value};
use tracing::instrument;

/// Truncate text to specified length
#[instrument(skip_all, fields(operation = "truncate"))]
pub fn truncate(value: &Value, args: &HashMap<String, Value>) -> Result<Value, TeraError> {
    let text = match value {
        Value::String(s) => s,
        _ => return Err(TeraError::msg("Expected a string")),
    };

    let length = args.get("length").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

    if text.len() <= length {
        Ok(Value::String(text.clone()))
    } else {
        Ok(Value::String(format!("{}...", &text[..length])))
    }
}

#[instrument(skip_all, fields(operation = "get_module_status"))]
pub fn get_module_status(value: &Value, _: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    match value {
        Value::String(value) => Ok(Value::String(value.clone())),
        Value::Object(map) => {
            if let Some((key, _)) = map.iter().next() {
                return Ok(Value::String(key.clone()));
            }

            Err(tera::Error::msg("Object is empty, no key found"))
        }
        _ => Err(tera::Error::msg(
            "Expected an object, but got a different type",
        )),
    }
}

/// Sanitize SVG content to prevent XSS attacks
/// Returns safe SVG or a fallback icon on sanitization failure
#[instrument(skip_all, fields(operation = "svg_safe"))]
pub fn svg_safe(value: &Value, args: &HashMap<String, Value>) -> Result<Value, TeraError> {
    let svg_content = match value {
        Value::String(s) => s,
        Value::Null => {
            // Return default icon for null values
            return Ok(Value::String(get_fallback_icon().to_string()));
        }
        _ => return Err(TeraError::msg("Expected a string for SVG content")),
    };

    // Get fallback behavior from args (default: "icon")
    let fallback = args
        .get("fallback")
        .and_then(|v| v.as_str())
        .unwrap_or("icon");

    match sanitize_svg(svg_content) {
        Ok(safe_svg) => {
            tracing::debug!("SVG sanitization successful");
            Ok(Value::String(safe_svg))
        }
        Err(error) => {
            // Log security attempt
            tracing::warn!(
                error = %error,
                svg_content = %svg_content,
                "Potentially malicious SVG content blocked"
            );

            match fallback {
                "error" => Err(TeraError::msg(format!(
                    "SVG sanitization failed: {}",
                    error
                ))),
                "empty" => Ok(Value::String(String::new())),
                _ => Ok(Value::String(get_fallback_icon().to_string())),
            }
        }
    }
}

/// Sanitize SVG content by removing dangerous elements and attributes
fn sanitize_svg(svg_content: &str) -> Result<String, String> {
    let trimmed = svg_content.trim();

    // Must be valid SVG structure
    if !trimmed.starts_with("<svg") || !trimmed.ends_with("</svg>") {
        return Err("Invalid SVG structure".to_string());
    }

    // Block dangerous elements
    let dangerous_elements = [
        "<script",
        "</script>",
        "<object",
        "</object>",
        "<embed",
        "</embed>",
        "<iframe",
        "</iframe>",
        "<link",
        "</link>",
        "<meta",
        "</meta>",
        "javascript:",
        "data:text/html",
    ];

    for dangerous in &dangerous_elements {
        if trimmed.to_lowercase().contains(dangerous) {
            return Err(format!("Contains dangerous element: {}", dangerous));
        }
    }

    // Block dangerous attributes (simple string matching for now)
    let dangerous_patterns = [
        "onclick=",
        "onload=",
        "onmouseover=",
        "onerror=",
        "onabort=",
        "href=\"javascript:",
        "src=\"javascript:",
        "xlink:href=\"javascript:",
        "href='javascript:",
        "src='javascript:",
        "xlink:href='javascript:",
    ];

    for pattern in &dangerous_patterns {
        if trimmed.to_lowercase().contains(pattern) {
            return Err(format!("Contains dangerous attribute pattern: {}", pattern));
        }
    }

    // Additional security: limit SVG size to prevent DoS
    if trimmed.len() > 10_000 {
        // 10KB limit
        return Err("SVG content too large".to_string());
    }

    Ok(trimmed.to_string())
}
/// Get fallback icon when SVG sanitization fails
fn get_fallback_icon() -> &'static str {
    r#"<svg class="w-4 h-4 mr-3 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
        <path d="M10 2L9 3H6a1 1 0 000 2h3l1-1 1 1h3a1 1 0 100-2h-3l-1-1z"/>
    </svg>"#
}
