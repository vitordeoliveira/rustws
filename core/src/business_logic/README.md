# Business Logic Structure Standard

## Module Organization

Each business logic module should follow this standard structure:

```
module_name/
├── dto.rs          # Data Transfer Objects - request/response structs
├── mod.rs          # Module exports and public interface
├── repository.rs   # Data access layer (if needed)
└── service.rs      # Business logic implementation
```

## File Responsibilities

### `dto.rs`

- Request/response structs
- Validation attributes
- Serialization/deserialization

### `mod.rs`

- Public exports from the module
- Module interface definition

### `repository.rs`

- Database queries and data access
- CRUD operations
- Data mapping
- Only include if module needs database access

### `service.rs`

- Core business logic
- Orchestration between repository and external services
- Input validation and processing

## Naming Conventions

- Module names: snake_case (e.g., `client_features`, `client_addons`)
- Struct names: PascalCase (e.g., `CreateClientRequest`)
- Function names: snake_case (e.g., `create_client`)

## When to Create a New Module

Create a new module when you have:

- A distinct business domain/entity
- Separate data models
- Independent business rules
- Clear boundaries from other modules
