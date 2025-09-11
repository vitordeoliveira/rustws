# Error Handling and Logging Guide

## Why Logging in Error Handling is Critical

When errors occur in production, logs are often your only window into what went wrong. Poor error logging makes debugging nearly impossible, while good error logging can help you identify and fix issues quickly.

**The Golden Rule**: Every error that crosses a boundary (file I/O, network, database, external service) should be logged with sufficient context to understand what operation failed.

## The Problem with Silent Errors

### ❌ Bad: Silent Error Conversion
```rust
async fn load_user_config(user_id: u64) -> AppResult<UserConfig> {
    let config_path = format!("configs/user_{}.toml", user_id);
    
    // ❌ No logging - when this fails, you have no idea why
    let content = tokio::fs::read_to_string(&config_path)
        .await
        .map_err(|e| AppError::Configuration { 
            message: e.to_string() 
        })?;
    
    // ❌ No logging - parse errors are silent
    let config: UserConfig = toml::from_str(&content)
        .map_err(|e| AppError::Configuration { 
            message: e.to_string() 
        })?;
    
    Ok(config)
}
```

**What happens when this fails?**
- You get a generic "Configuration error" 
- No idea which user's config failed
- No idea if it's a file not found, permission issue, or parse error
- No path information to debug the issue
- Debugging becomes guesswork

### ✅ Good: Logged Error Conversion
```rust
async fn load_user_config(user_id: u64) -> AppResult<UserConfig> {
    let config_path = format!("configs/user_{}.toml", user_id);
    
    // ✅ Detailed logging with path context
    let content = tokio::fs::read_to_string(&config_path)
        .await
        .map_err(|e| AppError::configuration_from_io_error(
            e,
            Path::new(&config_path),
            &format!("Loading config for user {}", user_id)
        ))?;
    
    // ✅ Parse error logging with context
    let config: UserConfig = toml::from_str(&content)
        .map_err(|e| AppError::configuration_from_parse_error(
            e,
            &config_path,
            &format!("Parsing TOML config for user {}", user_id)
        ))?;
    
    Ok(config)
}
```

**When this fails, you get:**
```
ERROR configuration_error: IO error while accessing configuration file
  io_error="No such file or directory (os error 2)"
  path="configs/user_123.toml"
  current_dir="/app"
  full_path="/app/configs/user_123.toml" 
  exists=false
  context="Loading config for user 123"
```

Now you know exactly what went wrong and can fix it quickly!

## When to Use Each Logging Method

### 1. Configuration Errors

#### File I/O Operations
```rust
// Reading configuration files
let content = std::fs::read_to_string(&config_path)
    .map_err(|e| AppError::configuration_from_io_error(
        e,
        &config_path,
        "Loading application configuration"
    ))?;

// Writing configuration files  
std::fs::write(&config_path, &content)
    .map_err(|e| AppError::configuration_from_io_error(
        e,
        &config_path,
        "Saving user preferences"
    ))?;
```

#### Parsing Configuration
```rust
// TOML parsing
let config: AppConfig = toml::from_str(&content)
    .map_err(|e| AppError::configuration_from_parse_error(
        e,
        "app.toml",
        "Parsing application configuration"
    ))?;

// JSON parsing
let settings: Settings = serde_json::from_str(&content)
    .map_err(|e| AppError::configuration_from_parse_error(
        e,
        "settings.json", 
        "Parsing user settings"
    ))?;
```

#### Environment Variables
```rust
// Required environment variables
let database_url = env::var("DATABASE_URL")
    .map_err(|_| AppError::configuration_from_env_error(
        "DATABASE_URL",
        None,
        "Environment variable not set",
        "Database connection setup"
    ))?;

// Environment variable validation
let port: u16 = env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse()
    .map_err(|_| AppError::configuration_from_env_error(
        "PORT",
        env::var("PORT").ok().as_deref(),
        "Invalid port number format",
        "Server port configuration"
    ))?;
```

### 2. Database Operations

#### Query Errors
```rust
async fn find_user_by_email(email: &str) -> AppResult<User> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
        .fetch_one(&pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::not_found_with_context(
                "user",
                Some(&format!("email: {}", email)),
                "User lookup by email"
            ),
            _ => AppError::database_with_context(
                &e.to_string(),
                Some("SELECT"),
                Some("users"),
                &format!("Finding user by email: {}", email)
            )
        })
}
```

#### Transaction Errors
```rust
async fn create_user_with_profile(user_data: CreateUserData) -> AppResult<User> {
    let mut tx = pool.begin().await
        .map_err(|e| AppError::database_with_context(
            &e.to_string(),
            Some("BEGIN_TRANSACTION"),
            None,
            "Starting user creation transaction"
        ))?;
    
    let user = sqlx::query_as!(...)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::database_with_context(
            &e.to_string(),
            Some("INSERT"),
            Some("users"),
            &format!("Creating user: {}", user_data.email)
        ))?;
    
    tx.commit().await
        .map_err(|e| AppError::database_with_context(
            &e.to_string(),
            Some("COMMIT"),
            None,
            "Committing user creation transaction"
        ))?;
    
    Ok(user)
}
```

### 3. External Service Calls

#### HTTP Requests
```rust
async fn fetch_user_profile(user_id: u64) -> AppResult<UserProfile> {
    let url = format!("https://api.example.com/users/{}", user_id);
    
    let response = reqwest::get(&url)
        .await
        .map_err(|e| {
            // Different error types get different handling
            match e {
                _ if e.is_timeout() => AppError::internal_with_context(
                    "External service timeout",
                    Some("fetch_user_profile"),
                    &format!("Fetching profile for user {}", user_id)
                ),
                _ if e.is_connect() => AppError::internal_with_context(
                    "Connection failed to external service",
                    Some("fetch_user_profile"),
                    &format!("Connecting to user service for user {}", user_id)
                ),
                _ => AppError::internal_with_context(
                    &format!("HTTP request failed: {}", e),
                    Some("fetch_user_profile"),
                    &format!("External user service call for user {}", user_id)
                )
            }
        })?;
    
    let profile: UserProfile = response.json()
        .await
        .map_err(|e| AppError::internal_with_context(
            &format!("Failed to parse response: {}", e),
            Some("parse_user_profile_response"),
            &format!("Parsing user profile response for user {}", user_id)
        ))?;
    
    Ok(profile)
}
```

### 4. Validation Errors

#### Form Validation
```rust
fn validate_user_registration(data: &CreateUserData) -> AppResult<()> {
    // Email validation
    if !is_valid_email(&data.email) {
        return Err(AppError::validation_with_context(
            "Invalid email format",
            Some("email"),
            "User registration form validation"
        ));
    }
    
    // Password strength
    if data.password.len() < 8 {
        return Err(AppError::validation_with_context(
            "Password must be at least 8 characters",
            Some("password"),
            "User registration password validation"
        ));
    }
    
    // Username uniqueness (this might involve a database call)
    check_username_available(&data.username)
        .await
        .map_err(|e| e.with_additional_context("Username uniqueness check during registration"))?;
    
    Ok(())
}
```

### 5. Authentication & Authorization

#### Login Attempts
```rust
async fn authenticate_user(credentials: LoginCredentials) -> AppResult<User> {
    let user = find_user_by_email(&credentials.email)
        .await
        .map_err(|e| match e {
            AppError::NotFound { .. } => AppError::authentication_with_context(
                "User not found",
                Some(&credentials.email),
                "User login authentication"
            ),
            _ => e.with_additional_context("User lookup during authentication")
        })?;
    
    if !verify_password(&credentials.password, &user.password_hash) {
        return Err(AppError::authentication_with_context(
            "Invalid password",
            Some(&credentials.email),
            "Password verification during login"
        ));
    }
    
    Ok(user)
}
```

#### Permission Checks
```rust
async fn check_admin_permission(user_id: u64, resource: &str) -> AppResult<()> {
    let user = find_user(user_id)
        .await
        .map_err(|e| e.with_additional_context("User lookup for permission check"))?;
    
    if !user.is_admin {
        return Err(AppError::authorization_with_context(
            "Admin access required",
            Some(resource),
            Some(&user.email),
            &format!("Admin permission check for {}", resource)
        ));
    }
    
    Ok(())
}
```

## Error Context Chaining

For complex operations that span multiple layers, chain context at each level:

```rust
// Top level: HTTP handler
async fn create_user_handler(data: CreateUserData) -> AppResult<User> {
    create_user_account(data)
        .await
        .map_err(|e| e.with_additional_context("HTTP request: POST /users"))
}

// Business logic layer
async fn create_user_account(data: CreateUserData) -> AppResult<User> {
    // Validation
    validate_user_data(&data)
        .map_err(|e| e.with_additional_context("User account creation validation"))?;
    
    // Database operation
    let user = insert_user(&data)
        .await
        .map_err(|e| e.with_additional_context("User account creation database operation"))?;
    
    // External service notification
    notify_user_created(&user)
        .await
        .map_err(|e| e.with_additional_context("Post creation user notification"))?;
    
    Ok(user)
}

// Data layer
async fn insert_user(data: &CreateUserData) -> AppResult<User> {
    sqlx::query_as!(...)
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::database_with_context(
            &e.to_string(),
            Some("INSERT"),
            Some("users"),
            &format!("Inserting new user: {}", data.email)
        ))
}
```

This produces a rich error trail:
```
ERROR database_error: Database error occurred
  message="unique constraint violation: users_email_key"
  operation="INSERT"
  table="users" 
  context="Inserting new user: john@example.com"

ERROR database_error: Database error with additional context
  additional_context="User account creation database operation"

ERROR database_error: Database error with additional context  
  additional_context="HTTP request: POST /users"
```

## Common Patterns and Anti-Patterns

### ❌ Anti-Pattern: Generic Error Messages
```rust
// Don't do this - no context about what failed
.map_err(|e| AppError::Internal { 
    message: "Something went wrong".to_string(),
    error_id: uuid::Uuid::new_v4().to_string(),
})
```

### ✅ Pattern: Specific Error Context
```rust
// Do this - clear context about the operation
.map_err(|e| AppError::internal_with_context(
    &format!("Failed to process payment: {}", e),
    Some("stripe_payment_processing"),
    &format!("Processing payment for order {}", order_id)
))
```

### ❌ Anti-Pattern: Logging Sensitive Data
```rust
// Don't log passwords, tokens, or sensitive data
.map_err(|e| AppError::authentication_with_context(
    &format!("Login failed with password: {}", password), // ❌ DON'T DO THIS
    Some(&email),
    "User authentication"
))
```

### ✅ Pattern: Safe Logging
```rust
// Log identifiers and non-sensitive context
.map_err(|e| AppError::authentication_with_context(
    "Invalid credentials",
    Some(&email), // Email is okay to log
    "User authentication"
))
```

### ❌ Anti-Pattern: Over-Logging
```rust
// Don't log expected business logic errors at ERROR level
if user.balance < amount {
    // This is business logic, not an error - use a different log level
    tracing::error!("Insufficient funds for user {}", user.id);
    return Err(AppError::validation_with_context(...));
}
```

### ✅ Pattern: Appropriate Log Levels
```rust
// Use appropriate log levels for different scenarios
if user.balance < amount {
    // Business logic validation - use info or debug
    tracing::info!("Payment declined: insufficient funds for user {}", user.id);
    return Err(AppError::validation_with_context(...));
}

// System errors should use error level  
.map_err(|e| AppError::internal_with_context(...)) // This uses ERROR level appropriately
```

## Debugging Workflow

With proper error logging, your debugging workflow becomes:

1. **Check logs** for the error with context
2. **Search by context** to find related operations  
3. **Follow the error chain** through different layers
4. **Identify the root cause** from the detailed error information
5. **Fix the issue** with confidence

### Example Log Search Queries

```bash
# Find all database errors for a specific user
grep "user_id=123" logs/app.log | grep "ERROR.*database"

# Find all configuration errors 
grep "ERROR.*configuration" logs/app.log

# Find specific operation failures
grep "context.*user registration" logs/app.log

# Find errors by operation type
grep "operation.*INSERT" logs/app.log
```

## Summary

**Always use context-aware error methods when:**
- Reading/writing files → `configuration_from_io_error`
- Parsing data → `configuration_from_parse_error`  
- Environment variables → `configuration_from_env_error`
- Database operations → `database_with_context`
- External API calls → `internal_with_context`
- Validation → `validation_with_context`
- Authentication → `authentication_with_context`
- Authorization → `authorization_with_context`

**The goal is**: When an error occurs in production, you should be able to understand what went wrong just from reading the logs, without needing to reproduce the issue or guess at the cause.

**Remember**: Logs are your best friend when debugging production issues. Invest in good error logging now, and save hours of debugging later! 