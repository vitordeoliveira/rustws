# Error Conversion Patterns

## Overview

This guide explains why we use manual `From` trait implementations instead of thiserror's `#[from]` attribute for converting external library errors to our `AppError` types.

## The Two Approaches

### ❌ Thiserror `#[from]` Approach

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    
    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}
```

### ✅ Manual `From` Implementation Approach

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {message}")]
    Validation { message: String, field: Option<String> },
    
    #[error("Database error: {message}")]
    Database { message: String },
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Internal server error: {message}")]
    Internal { message: String, error_id: String },
}

// Smart conversion based on error meaning, not source
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        match () {
            _ if err.is_timeout() => AppError::Internal { 
                message: format!("HTTP request timeout: {}", err),
                error_id: uuid::Uuid::new_v4().to_string(),
            },
            _ if err.is_connect() => AppError::Internal { 
                message: format!("HTTP connection failed: {}", err),
                error_id: uuid::Uuid::new_v4().to_string(),
            },
            _ => AppError::Internal { 
                message: format!("HTTP request failed: {}", err),
                error_id: uuid::Uuid::new_v4().to_string(),
            },
        }
    }
}
```

## Why Manual `From` is Better

### 1. **Smart Error Categorization**

**Problem with `#[from]`**: Creates error variants based on library types, not error meaning.

```rust
// ❌ Library-based categorization
match app_error {
    AppError::Reqwest(reqwest_err) => { /* What does this mean to the user? */ }
    AppError::Sqlx(sqlx_err) => { /* Database? Configuration? */ }
}
```

**Solution with manual `From`**: Categorizes by business meaning.

```rust
// ✅ Meaning-based categorization
match app_error {
    AppError::Database { .. } => { /* Clear: database operation failed */ }
    AppError::Internal { .. } => { /* Clear: system error occurred */ }
    AppError::Configuration { .. } => { /* Clear: config problem */ }
}
```

### 2. **Stable Error Interface**

**Problem with `#[from]`**: Error enum grows with every new library.

```rust
// ❌ Grows indefinitely
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    
    #[error("Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    
    #[error("Redis: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("S3: {0}")]
    S3(#[from] aws_sdk_s3::Error),
    
    // ... keeps growing with every dependency
}
```

**Solution with manual `From`**: Stable, domain-focused interface.

```rust
// ✅ Stable interface regardless of dependencies
#[derive(Error, Debug)]
pub enum AppError {
    Validation { message: String, field: Option<String> },
    Authentication { message: String },
    Authorization { message: String },
    NotFound { resource: String },
    Database { message: String },
    Configuration { message: String },
    Internal { message: String, error_id: String },
}
```

### 3. **Security & Information Control**

**Problem with `#[from]`**: Exposes internal implementation details.

```rust
// ❌ Exposes sensitive information
AppError::Reqwest(reqwest::Error::from(
    "DNS resolution failed for internal-api.company.com:8080"
))
// User sees internal infrastructure details!
```

**Solution with manual `From`**: Controlled information exposure.

```rust
// ✅ Controlled exposure
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        // Log detailed error for developers
        tracing::error!("HTTP request failed: {}", err);
        
        // Return safe error for users
        AppError::Internal { 
            message: "External service unavailable".to_string(),
            error_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}
```

### 4. **Error Tracking & Debugging**

**Problem with `#[from]`**: No way to add tracking information.

```rust
// ❌ No error tracking
#[error("Reqwest error: {0}")]
Reqwest(#[from] reqwest::Error),  // Can't add error_id
```

**Solution with manual `From`**: Built-in error tracking.

```rust
// ✅ Automatic error tracking
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Internal { 
            message: format!("HTTP error: {}", err),
            error_id: uuid::Uuid::new_v4().to_string(),  // ✅ Unique ID for tracking
        }
    }
}
```

### 5. **Context Enrichment**

**Problem with `#[from]`**: No context about what operation failed.

```rust
// ❌ No operation context
let data = fetch_user_data().await?;  // Which HTTP call failed?
```

**Solution with manual `From`**: Rich context information.

```rust
// ✅ Context-aware error conversion
impl AppError {
    pub fn from_reqwest_with_context(err: reqwest::Error, operation: &str) -> Self {
        AppError::Internal { 
            message: format!("Failed to {}: {}", operation, err),
            error_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

// Usage with context
let response = reqwest::get(url).await
    .map_err(|e| AppError::from_reqwest_with_context(e, "fetch user profile"))?;
```

## Conversion Patterns

### Pattern 1: Simple Library Error

```rust
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_data() {
            AppError::Validation { 
                message: format!("Invalid JSON data: {}", err),
                field: None,
            }
        } else {
            AppError::Internal { 
                message: format!("JSON processing error: {}", err),
                error_id: uuid::Uuid::new_v4().to_string(),
            }
        }
    }
}
```

### Pattern 2: Complex Error Categorization

```rust
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(db_err) => {
                // Check for specific database constraints
                if db_err.code() == Some(Cow::Borrowed("23505")) {
                    AppError::Validation { 
                        message: "Duplicate entry".to_string(),
                        field: None,
                    }
                } else {
                    AppError::Database { 
                        message: format!("Database constraint error: {}", db_err),
                    }
                }
            }
            sqlx::Error::RowNotFound => {
                AppError::NotFound { 
                    resource: "Database record".to_string(),
                }
            }
            sqlx::Error::PoolTimedOut => {
                AppError::Internal { 
                    message: "Database connection pool timeout".to_string(),
                    error_id: uuid::Uuid::new_v4().to_string(),
                }
            }
            _ => {
                AppError::Internal { 
                    message: format!("Database operation failed: {}", err),
                    error_id: uuid::Uuid::new_v4().to_string(),
                }
            }
        }
    }
}
```

### Pattern 3: Context-Aware Helpers

```rust
impl AppError {
    // Helper for HTTP operations with context
    pub fn http_error(operation: &str, err: reqwest::Error) -> Self {
        tracing::error!("HTTP error during {}: {}", operation, err);
        
        AppError::Internal { 
            message: format!("Failed to {}", operation),
            error_id: uuid::Uuid::new_v4().to_string(),
        }
    }
    
    // Helper for file operations
    pub fn file_error(operation: &str, path: &str, err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                AppError::NotFound { 
                    resource: format!("File: {}", path),
                }
            }
            std::io::ErrorKind::PermissionDenied => {
                AppError::Internal { 
                    message: format!("Permission denied: {} on {}", operation, path),
                    error_id: uuid::Uuid::new_v4().to_string(),
                }
            }
            _ => {
                AppError::Internal { 
                    message: format!("File {} failed on {}: {}", operation, path, err),
                    error_id: uuid::Uuid::new_v4().to_string(),
                }
            }
        }
    }
}
```

## When to Use `#[from]`

There are still valid use cases for `#[from]`:

### 1. **Internal Error Types**

```rust
// For internal error hierarchies where you want to preserve structure
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(#[from] sqlx::Error),
    
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

// Then convert the internal error to AppError
impl From<DatabaseError> for AppError {
    fn from(err: DatabaseError) -> Self {
        AppError::Database { 
            message: err.to_string(),
        }
    }
}
```

### 2. **Rapid Prototyping**

```rust
// During development when you want quick error propagation
#[derive(Error, Debug)]
pub enum PrototypeError {
    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("DB: {0}")]
    Database(#[from] sqlx::Error),
}

// Later refactor to proper categorization
```

### 3. **Library Crates**

```rust
// When building a library that should expose underlying errors
#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}
```

## Best Practices

### 1. **Error Conversion Guidelines**

```rust
// ✅ DO: Categorize by business meaning
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        match () {
            _ if err.is_timeout() => AppError::Internal { /* timeout */ },
            _ if err.is_connect() => AppError::Internal { /* connection */ },
            _ => AppError::Internal { /* other */ },
        }
    }
}

// ❌ DON'T: Just wrap the error
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Http(err)  // Exposes implementation details
    }
}
```

### 2. **Logging Strategy**

```rust
impl From<ExternalError> for AppError {
    fn from(err: ExternalError) -> Self {
        // Always log detailed error for developers
        tracing::error!("External service error: {:?}", err);
        
        // Return user-safe error
        AppError::Internal { 
            message: "Service temporarily unavailable".to_string(),
            error_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}
```

### 3. **Testing Error Conversions**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reqwest_timeout_conversion() {
        let reqwest_err = reqwest::Error::from(/* timeout error */);
        let app_err = AppError::from(reqwest_err);
        
        match app_err {
            AppError::Internal { message, .. } => {
                assert!(message.contains("timeout"));
            }
            _ => panic!("Expected Internal error"),
        }
    }
}
```

## Migration Strategy

### From `#[from]` to Manual `From`

1. **Identify Current `#[from]` Usage**
   ```bash
   grep -r "#\[from\]" src/
   ```

2. **Categorize Each External Error**
   - Configuration errors → `AppError::Configuration`
   - Database errors → `AppError::Database` or `AppError::NotFound`
   - Network errors → `AppError::Internal`
   - Validation errors → `AppError::Validation`

3. **Implement Manual Conversions**
   ```rust
   // Replace this
   #[error("Reqwest: {0}")]
   Reqwest(#[from] reqwest::Error),
   
   // With this
   impl From<reqwest::Error> for AppError { /* ... */ }
   ```

4. **Update Error Handling Code**
   ```rust
   // Change from
   match err {
       AppError::Reqwest(reqwest_err) => { /* ... */ }
   }
   
   // To
   match err {
       AppError::Internal { message, error_id } => { /* ... */ }
   }
   ```

5. **Add Tests**
   ```rust
   #[test]
   fn test_error_conversions() {
       // Test each conversion path
   }
   ```

## Summary

Manual `From` implementations provide:

- ✅ **Smart categorization** by error meaning, not source
- ✅ **Stable error interface** independent of dependencies  
- ✅ **Security** through controlled information exposure
- ✅ **Error tracking** with unique IDs
- ✅ **Context enrichment** for better debugging
- ✅ **Future flexibility** for evolving requirements

While `#[from]` is convenient for rapid development, manual `From` implementations are essential for production-grade error handling systems that prioritize user experience, security, and maintainability.

---

**Next Steps:**
1. Review your current error handling for `#[from]` usage
2. Implement manual `From` conversions for external library errors
3. Add comprehensive error logging and tracking
4. Test error conversion paths thoroughly

## Advanced Error Handling with Context Methods

The `AppError` type includes specialized methods for creating errors with automatic logging and context. These methods combine error creation with structured logging for better debugging.

### Context-Aware Error Creation

Instead of creating errors and then logging them separately, use the `*_with_context` methods for automatic logging:

```rust
// ❌ Manual error creation and logging
let validation_error = AppError::Validation {
    message: "Invalid email format".to_string(),
    field: Some("email".to_string()),
};
tracing::error!("Validation failed: {}", validation_error);
return Err(validation_error);

// ✅ Context-aware error creation with automatic logging
return Err(AppError::validation_with_context(
    "Invalid email format",
    Some("email"),
    "User registration form validation"
));
```

### Available Context Methods

#### 1. **Validation Errors**
```rust
// Field-specific validation
AppError::validation_with_context(
    "Email format is invalid",
    Some("email"),
    "User registration form"
)

// General validation
AppError::validation_with_context(
    "User already exists with this email",
    None,
    "User registration uniqueness check"
)
```

#### 2. **Authentication Errors**
```rust
AppError::authentication_with_context(
    "Invalid credentials",
    Some("user123"),
    "User login attempt"
)
```

#### 3. **Authorization Errors**
```rust
AppError::authorization_with_context(
    "Insufficient permissions to delete user",
    Some("users"),
    Some("admin123"),
    "Admin user management"
)
```

#### 4. **Not Found Errors**
```rust
AppError::not_found_with_context(
    "user",
    Some("by ID: 12345"),
    "User profile retrieval"
)
```

#### 5. **Database Errors**
```rust
AppError::database_with_context(
    "Failed to insert user record",
    Some("INSERT"),
    Some("users"),
    "User registration database operation"
)
```

#### 6. **Internal Errors**
```rust
AppError::internal_with_context(
    "Service unavailable",
    Some("fetch_user_data"),
    "External user service integration"
)
```

### Specialized Configuration Error Methods

For configuration-related errors, use these specialized methods that include path context:

#### 1. **Path-Based Configuration Errors**
```rust
// Automatically includes current directory, full path, and file existence
AppError::configuration_with_path(
    "Failed to access templates directory",
    Path::new("templates"),
    "Initializing template engine"
)
```

#### 2. **IO Error Conversion**
```rust
std::fs::read_to_string(&config_path).map_err(|io_error| {
    AppError::configuration_from_io_error(
        io_error,
        &config_path,
        "Loading application configuration"
    )
})?
```

#### 3. **Parse Error Conversion**
```rust
toml::from_str(&content).map_err(|parse_error| {
    AppError::configuration_from_parse_error(
        parse_error,
        "config.toml",
        "Parsing TOML configuration"
    )
})?
```

#### 4. **Environment Variable Errors**
```rust
AppError::configuration_from_env_error(
    "DATABASE_URL",
    env::var("DATABASE_URL").ok().as_deref(),
    "Missing required environment variable",
    "Database connection setup"
)
```

#### 5. **Generic Error Conversion**
```rust
some_operation().map_err(|e| {
    AppError::configuration_from_error(
        e,
        "Setting up file watcher"
    )
})?
```

### Adding Context to Existing Errors

Use `with_additional_context` to add more context to existing errors:

```rust
configuration::load_config().map_err(|e| {
    e.with_additional_context("Loading configuration during application startup")
})?
```

### Structured Logging Output

These methods produce structured logs with relevant fields:

```
2024-01-01T12:00:00Z ERROR validation_error: Validation error occurred
  message="Invalid email format"
  field="email"
  context="User registration form validation"

2024-01-01T12:00:00Z ERROR configuration_error: Configuration error with path context
  message="Failed to access templates directory"
  path="templates"
  current_dir="/app"
  full_path="/app/templates"
  exists=false
  context="Initializing template engine"
```

### Best Practices for Context Methods

#### 1. **Provide Meaningful Context**
```rust
// ✅ Good: Specific context about what operation failed
AppError::database_with_context(
    "Unique constraint violation",
    Some("INSERT"),
    Some("users"),
    "Creating new user account during registration"
)

// ❌ Poor: Vague context
AppError::database_with_context(
    "DB error",
    None,
    None,
    "Database operation"
)
```

#### 2. **Use Appropriate Context Fields**
```rust
// ✅ Include relevant identifiers
AppError::not_found_with_context(
    "user",
    Some(&format!("ID: {}", user_id)),
    "User profile page rendering"
)

// ✅ Include operation details
AppError::internal_with_context(
    "External API rate limit exceeded",
    Some("fetch_user_data"),
    "User data synchronization background job"
)
```

#### 3. **Chain Context for Complex Operations**
```rust
async fn create_user_account(data: CreateUserData) -> AppResult<User> {
    // Validation with context
    validate_user_data(&data).map_err(|e| {
        e.with_additional_context("Pre-creation user data validation")
    })?;
    
    // Database operation with context
    let user = database::create_user(&data).map_err(|e| {
        e.with_additional_context("User account creation database transaction")
    })?;
    
    // External service with context
    notify_user_created(&user).map_err(|e| {
        e.with_additional_context("Post-creation user notification")
    })?;
    
    Ok(user)
}
```

### Error Context Guidelines

- **Context should answer**: What operation was being performed?
- **Include identifiers**: User IDs, resource names, file paths
- **Be specific**: "User login form validation" vs "Validation"
- **Chain context**: Add context at each layer of the call stack
- **Use consistent naming**: Follow naming patterns across your application

These context methods make debugging production issues much easier by providing structured, searchable logs with relevant operational context. 