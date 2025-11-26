//! Authentication service module for ZenTao MCP Server
//!
//! This module provides a comprehensive authentication system for ZenTao API requests.
//! It implements a strategy pattern that allows for flexible authentication methods
//! while maintaining a consistent interface across different authentication approaches.
//!
//! # Architecture
//!
//! The authentication system is composed of several key components:
//!
//! - [`UnifiedAuthService`]: Main service that provides
//!   a unified interface for authentication operations
//! - [`AuthServiceFactory`]: Factory for creating authentication
//!   strategies based on configuration
//! - [`AuthStrategy`]: Trait defining the interface for authentication
//!   strategies
//! - [`AuthConfig`]: Configuration for authentication strategies
//!
//! # Examples
//!
//! ```rust,no_run
//! use mcp_any_rest::services::auth_service::{UnifiedAuthService, AuthConfig, AuthMode, DirectAuthConfig};
//!
//! // Create with direct configuration
//! let auth_config = AuthConfig {
//!     mode: AuthMode::Direct,
//!     direct_config: Some(DirectAuthConfig {
//!         auth_type: mcp_any_rest::services::auth_service::auth_strategy::DirectAuthType::Token,
//!         token: Some("pre-generated-token".to_string()),
//!         api_key_name: None,
//!         username: None,
//!         password: None,
//!         custom_headers: None,
//!     }),
//!     login_config: None,
//!     token_expiry: 3600,
//!     refresh_buffer: 300,
//!     max_retry_attempts: 3,
//! };
//!
//! let auth_service = UnifiedAuthService::new(auth_config).unwrap();
//!
//! // Get authentication headers
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let headers = auth_service.get_auth_headers().await.unwrap();
//! # });
//! ```
//!
//! # Features
//!
//! - Multiple authentication strategies (Direct Config, Login Request)
//! - Token management with automatic refresh
//! - Thread-safe implementation
//! - Comprehensive error handling
//! - Factory pattern for flexible strategy creation
//! - Backward compatibility with existing code

pub mod auth_factory;
pub mod auth_strategy;
pub mod auth_utils;
pub mod unified_auth_service;

// Re-export the unified authentication service and related types
pub use unified_auth_service::{UnifiedAuthService, AuthService};
pub use auth_factory::{AuthServiceFactory, AuthServiceFactoryBuilder};
pub use auth_strategy::{
    AuthConfig, AuthStrategy, AuthMode, DirectAuthConfig, LoginAuthConfig,
    HttpMethod, ResponseFormat, TokenExtraction, TokenExtractionItem, TokenLocation, TokenTargetLocation, 
    AuthError, DirectAuthType, LoginAuthType, TokenFormat
};

// Type alias for backward compatibility
pub type AuthServiceType = AuthService;

#[cfg(test)]
mod tests;