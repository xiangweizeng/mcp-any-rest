# Authentication Service

This directory contains the authentication service for the ZenTao MCP Server. The service provides a flexible, strategy-based authentication system that supports multiple authentication methods for ZenTao APIs.

## Quick Start

```rust
use mcp_any_rest::services::auth_service::{UnifiedAuthService, AuthConfig};

// Create with direct configuration
let auth_config = AuthConfig::DirectConfig {
    url: "https://zentao.example.com".to_string(),
    username: "admin".to_string(),
    password: "password".to_string(),
    token: Some("pre-generated-token".to_string()),
};

let auth_service = UnifiedAuthService::new(auth_config, None).await?;

// Get authentication headers
let headers = auth_service.get_auth_headers().await?;
```

## Documentation

- [Detailed Documentation](doc.md) - Comprehensive guide to the authentication service
- [API Documentation](../../../target/doc/mcp_any_rest/services/auth_service/index.html) - Generated API documentation

## Architecture

The authentication service is composed of several key components:

- **UnifiedAuthService** - Main service providing a unified interface for authentication
- **AuthServiceFactory** - Factory for creating authentication strategies
- **AuthStrategy** - Trait defining the interface for authentication strategies
- **AuthConfig** - Configuration for authentication strategies

## Testing

Run the authentication service tests with:

```bash
cargo test --package mcp-any-rest --lib services::auth_service
```

## Files

- `mod.rs` - Module definition and re-exports
- `unified_auth_service.rs` - Main authentication service implementation
- `auth_factory.rs` - Factory for creating authentication strategies
- `auth_strategy.rs` - Authentication strategy trait and configurations
- `auth_utils.rs` - Utility functions for authentication
- `doc.md` - Detailed documentation
- `tests.rs` - Module tests