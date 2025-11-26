//! Authentication utility functions for MCP-ANY-REST
//!
//! This module provides utility functions for authentication operations, including
//! parameter validation, response parsing, and error handling. These functions are
//! used throughout the authentication service to ensure consistent behavior.
//!
//! # Examples
//!
//! ```rust
//! use mcp_any_rest::services::auth_service::auth_utils::{
//!     validate_auth_params, parse_auth_response, is_auth_error_requiring_refresh
//! };
//! use serde_json::json;
//!
//! // Validate authentication parameters
//! validate_auth_params("admin", "password").unwrap();
//!
//! // Parse authentication response
//! let response = json!({"token": "abc123"});
//! let token = parse_auth_response(&response).unwrap();
//!
//! // Check if error requires token refresh
//! let needs_refresh = is_auth_error_requiring_refresh("UNAUTHORIZED: Token expired");
//! ```

use log::{debug, warn};
use rmcp::ErrorData as McpError;
use serde_json::Value;

/// Validate authentication parameters
pub fn validate_auth_params(account: &str, password: &str) -> Result<(), McpError> {
    if account.trim().is_empty() {
        return Err(McpError::invalid_params(
            "Account cannot be empty",
            None,
        ));
    }
    
    if password.trim().is_empty() {
        return Err(McpError::invalid_params(
            "Password cannot be empty",
            None,
        ));
    }
    
    debug!("Authentication parameters validated successfully");
    Ok(())
}

/// Parse authentication response and extract token
pub fn parse_auth_response(response: &Value) -> Result<String, McpError> {
    if let Some(token) = response.get("token").and_then(|v| v.as_str()) {
        if token.trim().is_empty() {
            warn!("Received empty token from authentication response");
            return Err(McpError::internal_error(
                "Received empty authentication token",
                None,
            ));
        }
        Ok(token.to_string())
    } else {
        warn!("Authentication response missing token field: {:?}", response);
        Err(McpError::internal_error(
            "Authentication response missing token field",
            None,
        ))
    }
}

/// Check if authentication error requires token refresh
pub fn is_auth_error_requiring_refresh(error_message: &str) -> bool {
    error_message.contains("UNAUTHORIZED") || 
    error_message.contains("unauthorized") ||
    error_message.contains("token") && error_message.contains("expired")
}

/// Extract error information from authentication response
pub fn extract_auth_error_info(response_text: &str) -> (String, String) {
    if let Ok(response_value) = serde_json::from_str::<Value>(response_text) {
        if let Some(error) = response_value.get("error").and_then(|v| v.as_str()) {
            let message = response_value
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown authentication error");
            
            return (error.to_string(), message.to_string());
        }
    }
    
    ("unknown".to_string(), response_text.to_string())
}