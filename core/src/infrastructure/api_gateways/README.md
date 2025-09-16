# API Gateway Infrastructure

This directory contains the infrastructure layer for managing API Gateway instances in RUSTWS Core.

## Directory Structure

```
api_gateways/
├── mod.rs           # API Gateway storage management module
├── README.md        # This documentation file
└── gateways/        # API Gateway configuration files (JSON)
```

## Directory Purposes

### `gateways/`

Contains JSON configuration files for API Gateway instances:

- **Configuration files**: `{gateway_name}.json`
- **Endpoint definitions**: Routes, methods, targets, authentication
- **Deployment settings**: Stages, base URLs, CORS configuration
- **Rate limiting**: Request throttling and burst limits

## API Gateway Lifecycle

1. **Creation**: Define gateway with endpoints and configuration
2. **Storage**: Save as JSON file in `gateways/` directory
3. **Loading**: Read configuration from JSON for runtime
4. **Deployment**: Apply configuration to routing system
5. **Management**: Update endpoints and settings

## Configuration Format

Each API Gateway is stored as a JSON file with:

- **Gateway metadata**: Name, description, status, stage
- **Endpoints array**: Individual route definitions
- **Deployment info**: URLs, timestamps, deployment status

## Endpoint Configuration

Each endpoint includes:

- **Path and method**: HTTP route definition
- **Target**: Lambda, Workflow, Proxy, or Static response
- **Security**: Authentication requirements, CORS settings
- **Performance**: Rate limiting configuration
- **Transformations**: Request/response modifications

## File Naming Conventions

### Gateway Files

- Use gateway name: `my-api.json`
- Use kebab-case for consistency
- Direct correlation to gateway name field

## Example Usage

```rust
use crate::infrastructure::api_gateways::ApiGatewayStorage;

let storage = ApiGatewayStorage::new();

// Get all gateways
let gateways = storage.get_api_gateways().await?;
```

## Integration

- **Business Logic**: Implements `ApiGatewayRepository` trait
- **Service Layer**: Used by `ApiGatewayService` for operations
- **HTTP Handlers**: Accessed via API endpoints
- **UI**: Displayed in dashboard and management pages
