//! Configuration management for MCP-ANY-REST
//!
//! This module provides configuration structures for the MCP-ANY-REST application,
//! with support for different authentication modes:
//! 1. Direct Authentication - Authentication information is directly configured and used in each request
//! 2. Login-based Authentication - Login information is configured first, then authentication is obtained after login

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::config::module::GlobalModuleConfig;

/// Main configuration structure for MCP-ANY-REST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    
    /// API configuration
    pub api: ApiConfig,
    
    /// Authentication configuration
    pub auth: AuthConfig,
    
    /// Module configuration
    pub module_config: GlobalModuleConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server port
    pub port: u16,
    
    /// Log level
    pub log_level: String,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Base URL for API requests
    pub base_url: String,
    
    /// Request timeout in seconds
    pub timeout: u64,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication mode (direct or login)
    pub mode: AuthMode,
    
    /// Direct authentication configuration (used when mode is "direct")
    pub direct_config: Option<DirectAuthConfig>,
    
    /// Login authentication configuration (used when mode is "login")
    pub login_config: Option<LoginAuthConfig>,
    
    /// Token expiry time in seconds
    pub token_expiry: u64,
    
    /// Token refresh buffer in seconds
    pub refresh_buffer: u64,
    
    /// Maximum retry attempts for authentication
    pub max_retry_attempts: u32,
}

/// Authentication mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    /// Direct authentication - authentication information is directly configured and used in each request
    Direct,
    
    /// Login-based authentication - login information is configured first, then authentication is obtained after login
    Login,
}

/// Direct authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectAuthConfig {
    /// Type of direct authentication
    pub auth_type: DirectAuthType,
    
    /// Authentication token (for token-based authentication)
    pub token: Option<String>,
    
    /// API key name (for API key authentication)
    pub api_key_name: Option<String>,
    
    /// Username (for basic authentication)
    pub username: Option<String>,
    
    /// Password (for basic authentication)
    pub password: Option<String>,
    
    /// Custom headers (for custom headers authentication)
    pub custom_headers: Option<HashMap<String, String>>,
}

/// Direct authentication type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DirectAuthType {
    /// Bearer token authentication
    Bearer,
    
    /// API key authentication
    ApiKey,
    
    /// Basic authentication
    Basic,
    
    /// Token authentication
    Token,
    
    /// Custom headers authentication
    CustomHeaders,
}

/// Login authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAuthConfig {
    /// Type of login authentication
    pub auth_type: LoginAuthType,
    
    /// Login URL
    pub url: String,
    
    /// HTTP method for login request
    pub method: HttpMethod,
    
    /// Headers for login request
    pub headers: Option<HashMap<String, String>>,
    
    /// Body for login request
    pub body: Option<LoginRequestBody>,
    
    /// Response format
    pub response_format: ResponseFormat,
    
    /// Token extraction configuration
    pub token_extraction: TokenExtraction,
    
    /// Token refresh URL (optional)
    pub refresh_url: Option<String>,
    
    /// HTTP method for token refresh request (optional)
    pub refresh_method: Option<HttpMethod>,
}

/// Login authentication type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LoginAuthType {
    /// JSON login authentication
    Json,
    
    /// Form login authentication
    Form,
    
    /// OAuth2 authentication
    OAuth2,
    
    /// API key login authentication
    ApiKey,
    
    /// Custom login authentication
    Custom,
}

/// HTTP method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// Response format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    Json,
    Xml,
    Text,
}

/// Login request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequestBody {
    /// Body format
    pub format: BodyFormat,
    
    /// Body content
    pub content: HashMap<String, String>,
}

/// Body format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BodyFormat {
    Json,
    Form,
}

/// Single token extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExtractionItem {
    /// Token location in the response
    pub source_location: TokenLocation,
    
    /// Token key in the response
    pub source_key: String,
    
    /// Token format
    pub format: TokenFormat,
    
    /// Where to place the extracted token
    pub target_location: TokenTargetLocation,
    
    /// Key name for the extracted token in the target location
    pub target_key: String,
}

/// Multiple token extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExtraction {
    /// List of token extraction configurations
    pub tokens: Vec<TokenExtractionItem>,
}

/// Target location for extracted tokens
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenTargetLocation {
    Header,
    Query,
    Cookie,
    Body,
}

/// Token location
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenLocation {
    Header,
    Body,
    Query,
}

/// Token format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenFormat {
    Bearer,
    Token,
    ApiKey,
    Raw,
    Basic,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            api: ApiConfig::default(),
            auth: AuthConfig::default(),
            module_config: GlobalModuleConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8082,
            log_level: "info".to_string(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.example.com".to_string(),
            timeout: 30,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            mode: AuthMode::Direct,
            direct_config: Some(DirectAuthConfig::default()),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        }
    }
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

impl Default for LoginAuthConfig {
    fn default() -> Self {
        Self {
            auth_type: LoginAuthType::Json,
            url: "https://api.example.com/login".to_string(),
            method: HttpMethod::Post,
            headers: None,
            body: None,
            response_format: ResponseFormat::Json,
            token_extraction: TokenExtraction::default(),
            refresh_url: None,
            refresh_method: None,
        }
    }
}

impl Default for HttpMethod {
    fn default() -> Self {
        Self::Post
    }
}

impl Default for ResponseFormat {
    fn default() -> Self {
        Self::Json
    }
}

impl Default for LoginAuthType {
    fn default() -> Self {
        Self::Json
    }
}

impl Default for BodyFormat {
    fn default() -> Self {
        Self::Json
    }
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
                },
                TokenExtractionItem {
                    source_location: TokenLocation::Body,
                    source_key: "refresh_token".to_string(),
                    format: TokenFormat::Raw,
                    target_location: TokenTargetLocation::Header,
                    target_key: "Refresh-Token".to_string(),
                },
            ],
        }
    }
}

impl Default for TokenTargetLocation {
    fn default() -> Self {
        Self::Header
    }
}

impl Default for TokenLocation {
    fn default() -> Self {
        Self::Body
    }
}

impl Default for TokenFormat {
    fn default() -> Self {
        Self::Bearer
    }
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Create a configuration with direct authentication
    pub fn with_direct_auth(
        auth_type: DirectAuthType,
        token: Option<String>,
        api_key_name: Option<String>,
        username: Option<String>,
        password: Option<String>,
        custom_headers: Option<HashMap<String, String>>,
    ) -> Self {
        let direct_config = DirectAuthConfig {
            auth_type,
            token,
            api_key_name,
            username,
            password,
            custom_headers,
        };
        
        Self {
            auth: AuthConfig {
                mode: AuthMode::Direct,
                direct_config: Some(direct_config),
                login_config: None,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    /// Create a configuration with login-based authentication
    pub fn with_login_auth(
        auth_type: LoginAuthType,
        url: String,
        method: HttpMethod,
        headers: Option<HashMap<String, String>>,
        body: Option<LoginRequestBody>,
        response_format: ResponseFormat,
        token_extraction: TokenExtraction,
        refresh_url: Option<String>,
        refresh_method: Option<HttpMethod>,
    ) -> Self {
        let login_config = LoginAuthConfig {
            auth_type,
            url,
            method,
            headers,
            body,
            response_format,
            token_extraction,
            refresh_url,
            refresh_method,
        };
        
        Self {
            auth: AuthConfig {
                mode: AuthMode::Login,
                direct_config: None,
                login_config: Some(login_config),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    /// Create a configuration with bearer token authentication
    pub fn with_bearer_auth(token: String) -> Self {
        Self::with_direct_auth(
            DirectAuthType::Bearer,
            Some(token),
            None,
            None,
            None,
            None,
        )
    }
    
    /// Create a configuration with API key authentication
    pub fn with_api_key_auth(api_key_name: String, token: String) -> Self {
        Self::with_direct_auth(
            DirectAuthType::ApiKey,
            Some(token),
            Some(api_key_name),
            None,
            None,
            None,
        )
    }
    
    /// Create a configuration with basic authentication
    pub fn with_basic_auth(username: String, password: String) -> Self {
        Self::with_direct_auth(
            DirectAuthType::Basic,
            None,
            None,
            Some(username),
            Some(password),
            None,
        )
    }
    
    /// Create a configuration with custom headers authentication
    pub fn with_custom_headers_auth(headers: HashMap<String, String>) -> Self {
        Self::with_direct_auth(
            DirectAuthType::CustomHeaders,
            None,
            None,
            None,
            None,
            Some(headers),
        )
    }
    
    /// Create a configuration with JSON login authentication
    pub fn with_json_login_auth(
        url: String,
        username: String,
        password: String,
        token_key: String,
    ) -> Self {
        let mut body_content = HashMap::new();
        body_content.insert("username".to_string(), username);
        body_content.insert("password".to_string(), password);
        
        let body = LoginRequestBody {
            format: BodyFormat::Json,
            content: body_content,
        };
        
        let token_extraction = TokenExtraction {
            tokens: vec![
                TokenExtractionItem {
                    source_location: TokenLocation::Body,
                    source_key: token_key,
                    format: TokenFormat::Bearer,
                    target_location: TokenTargetLocation::Header,
                    target_key: "Authorization".to_string(),
                }
            ],
        };
        
        Self::with_login_auth(
            LoginAuthType::Json,
            url,
            HttpMethod::Post,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
        )
    }
    
    /// Create a configuration with form login authentication
    pub fn with_form_login_auth(
        url: String,
        username: String,
        password: String,
        token_key: String,
    ) -> Self {
        let mut body_content = HashMap::new();
        body_content.insert("username".to_string(), username);
        body_content.insert("password".to_string(), password);
        
        let body = LoginRequestBody {
            format: BodyFormat::Form,
            content: body_content,
        };
        
        let token_extraction = TokenExtraction {
            tokens: vec![
                TokenExtractionItem {
                    source_location: TokenLocation::Body,
                    source_key: token_key,
                    format: TokenFormat::Bearer,
                    target_location: TokenTargetLocation::Header,
                    target_key: "Authorization".to_string(),
                }
            ],
        };
        
        Self::with_login_auth(
            LoginAuthType::Form,
            url,
            HttpMethod::Post,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
        )
    }
    
    /// Create a configuration with OAuth2 authentication
    pub fn with_oauth2_auth(
        url: String,
        client_id: String,
        client_secret: String,
        scope: Option<String>,
    ) -> Self {
        let mut body_content = HashMap::new();
        body_content.insert("grant_type".to_string(), "client_credentials".to_string());
        body_content.insert("client_id".to_string(), client_id);
        body_content.insert("client_secret".to_string(), client_secret);
        
        if let Some(scope) = scope {
            body_content.insert("scope".to_string(), scope);
        }
        
        let body = LoginRequestBody {
            format: BodyFormat::Form,
            content: body_content,
        };
        
        let token_extraction = TokenExtraction {
            tokens: vec![
                TokenExtractionItem {
                    source_location: TokenLocation::Body,
                    source_key: "access_token".to_string(),
                    format: TokenFormat::Bearer,
                    target_location: TokenTargetLocation::Header,
                    target_key: "Authorization".to_string(),
                }
            ],
        };
        
        Self::with_login_auth(
            LoginAuthType::OAuth2,
            url,
            HttpMethod::Post,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
        )
    }
    
    /// Create a configuration with JSON login authentication and multiple tokens
    pub fn with_json_login_auth_multi(
        url: String,
        username: String,
        password: String,
        token_extractions: Vec<TokenExtractionItem>,
    ) -> Self {
        let mut body_content = HashMap::new();
        body_content.insert("username".to_string(), username);
        body_content.insert("password".to_string(), password);
        
        let body = LoginRequestBody {
            format: BodyFormat::Json,
            content: body_content,
        };
        
        let token_extraction = TokenExtraction {
            tokens: token_extractions,
        };
        
        Self::with_login_auth(
            LoginAuthType::Json,
            url,
            HttpMethod::Post,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
        )
    }
    
    /// Create a configuration with form login authentication and multiple tokens
    pub fn with_form_login_auth_multi(
        url: String,
        username: String,
        password: String,
        token_extractions: Vec<TokenExtractionItem>,
    ) -> Self {
        let mut body_content = HashMap::new();
        body_content.insert("username".to_string(), username);
        body_content.insert("password".to_string(), password);
        
        let body = LoginRequestBody {
            format: BodyFormat::Form,
            content: body_content,
        };
        
        let token_extraction = TokenExtraction {
            tokens: token_extractions,
        };
        
        Self::with_login_auth(
            LoginAuthType::Form,
            url,
            HttpMethod::Post,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
        )
    }
    
    /// Create a configuration with OAuth2 authentication and multiple tokens
    pub fn with_oauth2_auth_multi(
        url: String,
        client_id: String,
        client_secret: String,
        scope: Option<String>,
        token_extractions: Vec<TokenExtractionItem>,
    ) -> Self {
        let mut body_content = HashMap::new();
        body_content.insert("grant_type".to_string(), "client_credentials".to_string());
        body_content.insert("client_id".to_string(), client_id);
        body_content.insert("client_secret".to_string(), client_secret);
        
        if let Some(scope) = scope {
            body_content.insert("scope".to_string(), scope);
        }
        
        let body = LoginRequestBody {
            format: BodyFormat::Form,
            content: body_content,
        };
        
        let token_extraction = TokenExtraction {
            tokens: token_extractions,
        };
        
        Self::with_login_auth(
            LoginAuthType::OAuth2,
            url,
            HttpMethod::Post,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
        )
    }
    
    /// Create a configuration with API key login authentication
    pub fn with_api_key_login_auth(
        url: String,
        api_key_name: String,
        api_key_value: String,
        token_key: String,
    ) -> Self {
        let mut headers = HashMap::new();
        headers.insert(api_key_name, api_key_value);
        
        let token_extraction = TokenExtraction {
            tokens: vec![
                TokenExtractionItem {
                    source_location: TokenLocation::Body,
                    source_key: token_key,
                    format: TokenFormat::Bearer,
                    target_location: TokenTargetLocation::Header,
                    target_key: "Authorization".to_string(),
                }
            ],
        };
        
        Self::with_login_auth(
            LoginAuthType::ApiKey,
            url,
            HttpMethod::Get,
            Some(headers),
            None,
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
        )
    }
    
    /// Create a configuration with API key login authentication and multiple tokens
    pub fn with_api_key_login_auth_multi(
        url: String,
        api_key_name: String,
        api_key_value: String,
        token_extractions: Vec<TokenExtractionItem>,
    ) -> Self {
        let mut headers = HashMap::new();
        headers.insert(api_key_name, api_key_value);
        
        let token_extraction = TokenExtraction {
            tokens: token_extractions,
        };
        
        Self::with_login_auth(
            LoginAuthType::ApiKey,
            url,
            HttpMethod::Get,
            Some(headers),
            None,
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
        )
    }
    
    /// Set server port
    pub fn with_server_port(mut self, port: u16) -> Self {
        self.server.port = port;
        self
    }
    
    /// Set log level
    pub fn with_log_level(mut self, log_level: String) -> Self {
        self.server.log_level = log_level;
        self
    }
    
    /// Set base URL
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.api.base_url = base_url;
        self
    }
    
    /// Set request timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.api.timeout = timeout;
        self
    }
    
    /// Set token expiry time
    pub fn with_token_expiry(mut self, token_expiry: u64) -> Self {
        self.auth.token_expiry = token_expiry;
        self
    }
    
    /// Set refresh buffer time
    pub fn with_refresh_buffer(mut self, refresh_buffer: u64) -> Self {
        self.auth.refresh_buffer = refresh_buffer;
        self
    }
    
    /// Set maximum retry attempts
    pub fn with_max_retry_attempts(mut self, max_retry_attempts: u32) -> Self {
        self.auth.max_retry_attempts = max_retry_attempts;
        self
    }
    
    /// Get module configuration
    pub fn get_module_config(&self, module_name: &str) -> Option<&crate::config::module::ModuleConfig> {
        self.module_config.get_module_config(module_name)
    }
    
    /// Check if a module is enabled
    pub fn is_module_enabled(&self, module_name: &str) -> bool {
        self.module_config.is_module_enabled(module_name)
    }
    
    /// Check if a method is enabled
    pub fn is_method_enabled(&self, module_name: &str, method_name: &str) -> bool {
        self.module_config.is_method_enabled(module_name, method_name)
    }
    
    /// Check if a resource is enabled
    pub fn is_resource_enabled(&self, module_name: &str, resource_name: &str) -> bool {
        self.module_config.is_resource_enabled(module_name, resource_name)
    }
    
    /// Get method configuration
    pub fn get_method_config(&self, module_name: &str, method_name: &str) -> Option<&crate::config::module::MethodConfig> {
        self.module_config.get_method_config(module_name, method_name)
    }
    
    /// Get resource configuration
    pub fn get_resource_config(&self, module_name: &str, resource_name: &str) -> Option<&crate::config::module::ResourceConfig> {
        self.module_config.get_resource_config(module_name, resource_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 8082);
        assert_eq!(config.server.log_level, "info");
        assert_eq!(config.api.base_url, "https://api.example.com");
        assert_eq!(config.api.timeout, 30);
        assert_eq!(config.auth.mode, AuthMode::Direct);
        assert!(config.auth.direct_config.is_some());
        assert!(config.auth.login_config.is_none());
        assert_eq!(config.auth.token_expiry, 3600);
        assert_eq!(config.auth.refresh_buffer, 300);
        assert_eq!(config.auth.max_retry_attempts, 3);
    }

    #[test]
    fn test_with_bearer_auth() {
        let config = Config::with_bearer_auth("test-token".to_string());
        assert_eq!(config.auth.mode, AuthMode::Direct);
        let direct_config = config.auth.direct_config.unwrap();
        assert_eq!(direct_config.auth_type, DirectAuthType::Bearer);
        assert_eq!(direct_config.token.unwrap(), "test-token");
    }

    #[test]
    fn test_with_api_key_auth() {
        let config = Config::with_api_key_auth("X-API-Key".to_string(), "test-api-key".to_string());
        assert_eq!(config.auth.mode, AuthMode::Direct);
        let direct_config = config.auth.direct_config.unwrap();
        assert_eq!(direct_config.auth_type, DirectAuthType::ApiKey);
        assert_eq!(direct_config.api_key_name.unwrap(), "X-API-Key");
        assert_eq!(direct_config.token.unwrap(), "test-api-key");
    }

    #[test]
    fn test_with_basic_auth() {
        let config = Config::with_basic_auth("test-user".to_string(), "test-password".to_string());
        assert_eq!(config.auth.mode, AuthMode::Direct);
        let direct_config = config.auth.direct_config.unwrap();
        assert_eq!(direct_config.auth_type, DirectAuthType::Basic);
        assert_eq!(direct_config.username.unwrap(), "test-user");
        assert_eq!(direct_config.password.unwrap(), "test-password");
    }

    #[test]
    fn test_with_custom_headers_auth() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
        
        let config = Config::with_custom_headers_auth(headers.clone());
        assert_eq!(config.auth.mode, AuthMode::Direct);
        let direct_config = config.auth.direct_config.unwrap();
        assert_eq!(direct_config.auth_type, DirectAuthType::CustomHeaders);
        assert_eq!(direct_config.custom_headers.unwrap(), headers);
    }

    #[test]
    fn test_with_json_login_auth() {
        let config = Config::with_json_login_auth(
            "https://example.com/login".to_string(),
            "test-user".to_string(),
            "test-password".to_string(),
            "token".to_string(),
        );
        
        assert_eq!(config.auth.mode, AuthMode::Login);
        let login_config = config.auth.login_config.unwrap();
        assert_eq!(login_config.auth_type, LoginAuthType::Json);
        assert_eq!(login_config.url, "https://example.com/login");
        assert_eq!(login_config.method, HttpMethod::Post);
        assert_eq!(login_config.response_format, ResponseFormat::Json);
        assert_eq!(login_config.token_extraction.tokens.len(), 1);
        assert_eq!(login_config.token_extraction.tokens[0].source_key, "token");
    }

    #[test]
    fn test_with_form_login_auth() {
        let config = Config::with_form_login_auth(
            "https://example.com/login".to_string(),
            "test-user".to_string(),
            "test-password".to_string(),
            "token".to_string(),
        );
        
        assert_eq!(config.auth.mode, AuthMode::Login);
        let login_config = config.auth.login_config.unwrap();
        assert_eq!(login_config.auth_type, LoginAuthType::Form);
        assert_eq!(login_config.url, "https://example.com/login");
        assert_eq!(login_config.method, HttpMethod::Post);
        assert_eq!(login_config.response_format, ResponseFormat::Json);
        assert_eq!(login_config.token_extraction.tokens.len(), 1);
        assert_eq!(login_config.token_extraction.tokens[0].source_key, "token");
    }

    #[test]
    fn test_with_oauth2_auth() {
        let config = Config::with_oauth2_auth(
            "https://example.com/oauth2/token".to_string(),
            "client-id".to_string(),
            "client-secret".to_string(),
            Some("read write".to_string()),
        );
        
        assert_eq!(config.auth.mode, AuthMode::Login);
        let login_config = config.auth.login_config.unwrap();
        assert_eq!(login_config.auth_type, LoginAuthType::OAuth2);
        assert_eq!(login_config.url, "https://example.com/oauth2/token");
        assert_eq!(login_config.method, HttpMethod::Post);
        assert_eq!(login_config.response_format, ResponseFormat::Json);
        assert_eq!(login_config.token_extraction.tokens.len(), 1);
        assert_eq!(login_config.token_extraction.tokens[0].source_key, "access_token");
    }

    #[test]
    fn test_with_api_key_login_auth() {
        let config = Config::with_api_key_login_auth(
            "https://example.com/api/token".to_string(),
            "X-API-Key".to_string(),
            "test-api-key".to_string(),
            "token".to_string(),
        );
        
        assert_eq!(config.auth.mode, AuthMode::Login);
        let login_config = config.auth.login_config.unwrap();
        assert_eq!(login_config.auth_type, LoginAuthType::ApiKey);
        assert_eq!(login_config.url, "https://example.com/api/token");
        assert_eq!(login_config.method, HttpMethod::Get);
        assert_eq!(login_config.response_format, ResponseFormat::Json);
        assert_eq!(login_config.token_extraction.tokens.len(), 1);
        assert_eq!(login_config.token_extraction.tokens[0].source_key, "token");
    }

    #[test]
    fn test_builder_pattern() {
        let config = Config::default()
            .with_server_port(9000)
            .with_log_level("debug".to_string())
            .with_base_url("https://api.custom.com".to_string())
            .with_timeout(60)
            .with_token_expiry(7200)
            .with_refresh_buffer(600)
            .with_max_retry_attempts(5);
        
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.server.log_level, "debug");
        assert_eq!(config.api.base_url, "https://api.custom.com");
        assert_eq!(config.api.timeout, 60);
        assert_eq!(config.auth.token_expiry, 7200);
        assert_eq!(config.auth.refresh_buffer, 600);
        assert_eq!(config.auth.max_retry_attempts, 5);
    }

    #[test]
    fn test_serialization_deserialization() {
        let config = Config::with_bearer_auth("test-token".to_string())
            .with_server_port(9000)
            .with_base_url("https://api.custom.com".to_string());
        
        let serialized = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.server.port, deserialized.server.port);
        assert_eq!(config.server.log_level, deserialized.server.log_level);
        assert_eq!(config.api.base_url, deserialized.api.base_url);
        assert_eq!(config.api.timeout, deserialized.api.timeout);
        assert_eq!(config.auth.mode, deserialized.auth.mode);
        
        let config_direct = config.auth.direct_config.unwrap();
        let deserialized_direct = deserialized.auth.direct_config.unwrap();
        assert_eq!(config_direct.auth_type, deserialized_direct.auth_type);
        assert_eq!(config_direct.token, deserialized_direct.token);
    }
}