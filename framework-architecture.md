# Framework Architecture

Our framework follows a simple 3-layer architecture that separates concerns clearly:

## The Three Layers

```
┌─────────────────┐
│     ROUTING     │  ← WHEN (decides when things happen)
│   (handlers)    │
└─────────────────┘
         │
         ▼
┌─────────────────┐
│ BUSINESS_LOGIC  │  ← WHAT (defines what should happen)
│   (traits)      │
└─────────────────┘
         │
         ▼
┌─────────────────┐
│ INFRASTRUCTURE  │  ← HOW (implements how things happen)
│ (implementations)│
└─────────────────┘
```

## Layer Responsibilities

### 1. Routing (WHEN)

- **Purpose**: Decides **when** things should run
- **Location**: `src/routing/` and `src/handlers/`
- **What it does**: HTTP routing, middleware, request handling
- **Example**: "When a POST request comes to `/login`, call the authentication handler"

### 2. Business Logic (WHAT)

- **Purpose**: Defines **what** should happen
- **Location**: `src/business_logic/`
- **What it does**: Defines traits and business rules
- **Key Rule**: **NEVER implements anything** - only defines interfaces
- **Example**: "We need a `UserRepository` trait with `find_by_email()` method"

### 3. Infrastructure (HOW)

- **Purpose**: Implements **how** things actually work
- **Location**: `src/infrastructure/`
- **What it does**: Concrete implementations of business logic traits
- **Example**: "Here's how `find_by_email()` works with PostgreSQL"

## Simple Example

```rust
// BUSINESS_LOGIC: Defines WHAT we need
pub trait UserRepository {
    async fn find_by_email(&self, email: &str) -> Result<User>;
}

// INFRASTRUCTURE: Implements HOW it works
pub struct PostgresUserRepository {
    pool: DatabasePool,
}

impl UserRepository for PostgresUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<User> {
        // Actual database implementation here
    }
}

// ROUTING: Decides WHEN it happens
pub async fn login_handler(
    user_repo: UserRepository,
    request: LoginRequest,
) -> Response {
    // Called when POST /login happens
    let user = user_repo.find_by_email(&request.email).await?;
    // ... rest of login logic
}
```

## Key Benefits

### Clean Separation

- **Routing** doesn't know about databases
- **Business Logic** doesn't know about HTTP or databases
- **Infrastructure** doesn't know about HTTP routing

### Easy Testing

- Mock the traits for unit tests
- Test each layer independently
- Business logic is pure and testable

### Flexibility

- Swap PostgreSQL for MongoDB by changing infrastructure
- Change HTTP framework by changing routing
- Business rules stay unchanged

## File Structure

```
src/
├── routing/           # WHEN: HTTP routes and middleware
├── handlers/          # WHEN: Request handlers
├── business_logic/    # WHAT: Traits and business rules
└── infrastructure/    # HOW: Concrete implementations
```

## Remember

- **Business Logic = Traits Only** (no implementations)
- **Infrastructure = Implementations Only** (no business rules)
- **Routing = Orchestration** (when to call what)

This keeps everything simple, testable, and maintainable.
