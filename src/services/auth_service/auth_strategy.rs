//! Authentication strategy definitions for MCP-ANY-REST
//!
//! This module defines the authentication strategy trait and related types used
//! throughout the authentication service. It provides a flexible way to implement
//! different authentication methods while maintaining a consistent interface.
//!
//! The authentication system is divided into two main categories:
//! 1. Direct Authentication - Authentication information is directly configured and used in each request
//! 2. Login-based Authentication - Login information is configured first, then authentication is obtained after login

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authentication mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthMode {
    /// Direct authentication - use pre-configured authentication information
    Direct,
    /// Login-based authentication - obtain authentication after login
    Login,
}

impl Default for AuthMode {
    fn default() -> Self {
        AuthMode::Direct
    }
}

impl std::fmt::Display for AuthMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMode::Direct => write!(f, "direct"),
            AuthMode::Login => write!(f, "login"),
        }
    }
}

impl std::str::FromStr for AuthMode {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "direct" => Ok(AuthMode::Direct),
            "login" => Ok(AuthMode::Login),
            _ => Err(format!("Unknown authentication mode: {}", s)),
        }
    }
}

/// Direct authentication strategy types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DirectAuthType {
    /// Token-based authentication
    Token,
    /// Bearer token authentication
    Bearer,
    /// Basic authentication
    Basic,
    /// API key authentication
    ApiKey,
    /// Custom headers authentication
    CustomHeaders,
}

impl Default for DirectAuthType {
    fn default() -> Self {
        DirectAuthType::Token
    }
}

impl std::fmt::Display for DirectAuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectAuthType::Token => write!(f, "token"),
            DirectAuthType::Bearer => write!(f, "bearer"),
            DirectAuthType::Basic => write!(f, "basic"),
            DirectAuthType::ApiKey => write!(f, "apikey"),
            DirectAuthType::CustomHeaders => write!(f, "customheaders"),
        }
    }
}

impl std::str::FromStr for DirectAuthType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "token" => Ok(DirectAuthType::Token),
            "bearer" => Ok(DirectAuthType::Bearer),
            "basic" => Ok(DirectAuthType::Basic),
            "apikey" => Ok(DirectAuthType::ApiKey),
            "customheaders" => Ok(DirectAuthType::CustomHeaders),
            _ => Err(format!("Unknown direct authentication type: {}", s)),
        }
    }
}

/// Login-based authentication strategy types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoginAuthType {
    /// Form-based login
    Form,
    /// JSON-based login
    Json,
    /// OAuth2 authentication
    OAuth2,
    /// API key authentication
    ApiKey,
    /// Custom login method
    Custom,
}

impl Default for LoginAuthType {
    fn default() -> Self {
        LoginAuthType::Form
    }
}

impl std::fmt::Display for LoginAuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginAuthType::Form => write!(f, "form"),
            LoginAuthType::Json => write!(f, "json"),
            LoginAuthType::OAuth2 => write!(f, "oauth2"),
            LoginAuthType::ApiKey => write!(f, "apikey"),
            LoginAuthType::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for LoginAuthType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "form" => Ok(LoginAuthType::Form),
            "json" => Ok(LoginAuthType::Json),
            "oauth2" => Ok(LoginAuthType::OAuth2),
            "apikey" => Ok(LoginAuthType::ApiKey),
            "custom" => Ok(LoginAuthType::Custom),
            _ => Err(format!("Unknown login authentication type: {}", s)),
        }
    }
}

/// HTTP method enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::POST
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
        }
    }
}

/// Request body format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyFormat {
    Json,
    Form,
    Xml,
    Text,
}

impl Default for BodyFormat {
    fn default() -> Self {
        BodyFormat::Json
    }
}

impl std::fmt::Display for BodyFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyFormat::Json => write!(f, "Json"),
            BodyFormat::Form => write!(f, "Form"),
            BodyFormat::Xml => write!(f, "Xml"),
            BodyFormat::Text => write!(f, "Text"),
        }
    }
}

/// Response format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseFormat {
    Json,
    Xml,
    Text,
}

impl Default for ResponseFormat {
    fn default() -> Self {
        ResponseFormat::Json
    }
}

impl std::fmt::Display for ResponseFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseFormat::Json => write!(f, "Json"),
            ResponseFormat::Xml => write!(f, "Xml"),
            ResponseFormat::Text => write!(f, "Text"),
        }
    }
}

/// Token location in response
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenLocation {
    Header,
    Body,
    Query,
}

impl Default for TokenLocation {
    fn default() -> Self {
        TokenLocation::Body
    }
}

impl std::fmt::Display for TokenLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenLocation::Header => write!(f, "Header"),
            TokenLocation::Body => write!(f, "Body"),
            TokenLocation::Query => write!(f, "Query"),
        }
    }
}

/// Token target location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenTargetLocation {
    Header,
    Query,
    Cookie,
    Body,
}

impl Default for TokenTargetLocation {
    fn default() -> Self {
        TokenTargetLocation::Header
    }
}

impl std::fmt::Display for TokenTargetLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenTargetLocation::Header => write!(f, "Header"),
            TokenTargetLocation::Query => write!(f, "Query"),
            TokenTargetLocation::Cookie => write!(f, "Cookie"),
            TokenTargetLocation::Body => write!(f, "Body"),
        }
    }
}

/// Token format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenFormat {
    Bearer,
    Basic,
    Raw,
}

impl Default for TokenFormat {
    fn default() -> Self {
        TokenFormat::Bearer
    }
}

impl std::fmt::Display for TokenFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenFormat::Bearer => write!(f, "Bearer"),
            TokenFormat::Basic => write!(f, "Basic"),
            TokenFormat::Raw => write!(f, "Raw"),
        }
    }
}

/// Authentication error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Token expired: {0}")]
    TokenExpired(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Strategy not supported: {0}")]
    StrategyNotSupported(String),
    
    #[error("Token not found: {0}")]
    TokenNotFound(String),
    
    #[error("Login failed: {0}")]
    LoginFailed(String),
}

/// Authentication strategy trait
#[async_trait::async_trait]
pub trait AuthStrategy: Send + Sync {
    /// Get authentication token
    async fn get_token(&self) -> Result<String, AuthError>;
    
    /// Refresh token
    async fn refresh_token(&self) -> Result<String, AuthError>;
    
    /// Validate token validity
    async fn validate_token(&self, token: &str) -> Result<bool, AuthError>;
    
    /// Get authentication mode
    fn get_auth_mode(&self) -> AuthMode;
    
    /// Check if token needs refresh
    async fn needs_refresh(&self) -> Result<bool, AuthError>;
    
    /// Get authentication headers
    async fn get_auth_headers(&self) -> Result<reqwest::header::HeaderMap, AuthError>;
    
    /// Login and get token at specific index
    async fn login_and_get_token(&self, token_index: usize) -> Result<String, AuthError>;
}

/// Direct authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectAuthConfig {
    /// Direct authentication type
    pub auth_type: DirectAuthType,
    /// Authentication token (for Token, Bearer, ApiKey types)
    pub token: Option<String>,
    /// API key name (for ApiKey type)
    pub api_key_name: Option<String>,
    /// Username (for Basic type)
    pub username: Option<String>,
    /// Password (for Basic type)
    pub password: Option<String>,
    /// Custom headers (for CustomHeaders type)
    pub custom_headers: Option<HashMap<String, String>>,
}

impl Default for DirectAuthConfig {
    fn default() -> Self {
        Self {
            auth_type: DirectAuthType::Token,
            token: None,
            api_key_name: None,
            username: None,
            password: None,
            custom_headers: None,
        }
    }
}

/// Login-based authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAuthConfig {
    /// Login authentication type
    pub auth_type: LoginAuthType,
    /// Login URL
    pub url: String,
    /// HTTP method for login
    pub method: HttpMethod,
    /// Request headers for login
    pub headers: Option<HashMap<String, String>>,
    /// Request body for login
    pub body: Option<LoginRequestBody>,
    /// Response format
    pub response_format: ResponseFormat,
    /// Token extraction configuration
    pub token_extraction: TokenExtraction,
    /// Token refresh URL (optional, defaults to login URL)
    pub refresh_url: Option<String>,
    /// Token refresh method (optional, defaults to login method)
    pub refresh_method: Option<HttpMethod>,
}

/// Login request body configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequestBody {
    /// Body format
    pub format: BodyFormat,
    /// Body content
    pub content: HashMap<String, String>,
}

/// Token extraction configuration item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExtractionItem {
    /// Source location where to extract the token from
    pub source_location: TokenLocation,
    /// Key to extract the token from the source
    pub source_key: String,
    /// Token format
    pub format: TokenFormat,
    /// Target location where to place the extracted token
    pub target_location: TokenTargetLocation,
    /// Key name for the target location
    pub target_key: String,
}

/// Token extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExtraction {
    /// List of token extraction configurations
    pub tokens: Vec<TokenExtractionItem>,
}

impl Default for TokenExtraction {
    fn default() -> Self {
        Self {
            tokens: vec![
                TokenExtractionItem {
                    source_location: TokenLocation::Body,
                    source_key: "token".to_string(),
                    format: TokenFormat::Bearer,
                    target_location: TokenTargetLocation::Header,
                    target_key: "Authorization".to_string(),
                }
            ],
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication mode
    pub mode: AuthMode,
    /// Direct authentication configuration (if mode is Direct)
    pub direct_config: Option<DirectAuthConfig>,
    /// Login-based authentication configuration (if mode is Login)
    pub login_config: Option<LoginAuthConfig>,
    /// Token expiry time in seconds
    pub token_expiry: u64,
    /// Refresh buffer time in seconds
    pub refresh_buffer: u64,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            mode: AuthMode::Direct,
            direct_config: Some(DirectAuthConfig::default()),
            login_config: None,
            token_expiry: 3600, // 1 hour
            refresh_buffer: 300, // 5 minutes
            max_retry_attempts: 3,
        }
    }
}

// Note: rmcp::ErrorData conversion is implemented in auth_utils.rs
// to avoid circular dependencies