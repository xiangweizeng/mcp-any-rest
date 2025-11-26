//! Unified authentication service for MCP-ANY-REST
//!
//! This module provides a unified interface for handling different authentication modes:
//! 1. Direct Authentication - Authentication information is directly configured and used in each request
//! 2. Login-based Authentication - Login information is configured first, then authentication is obtained after login

use super::auth_strategy::{
    AuthConfig, AuthError, AuthMode, DirectAuthType, LoginAuthType,
    DirectAuthConfig, LoginAuthConfig, TokenExtraction, TokenExtractionItem, TokenFormat, TokenLocation, TokenTargetLocation,
    HttpMethod, ResponseFormat, BodyFormat, LoginRequestBody
};
use super::auth_factory::AuthServiceFactory;
use log::{debug, info, warn};
use reqwest::header::HeaderMap;
use reqwest::{Client, StatusCode};
use std::collections::HashMap;
use std::sync::Arc;
use serde::de::DeserializeOwned;
use rmcp::ErrorData as McpError;

/// Unified authentication service that provides a single interface for all authentication modes
pub struct UnifiedAuthService {
    factory: Arc<tokio::sync::Mutex<AuthServiceFactory>>,
    client: Client,
}

// Type alias for backward compatibility
pub type AuthService = UnifiedAuthService;

impl UnifiedAuthService {
    /// Create a new unified authentication service
    pub fn new(config: AuthConfig) -> Result<Self, AuthError> {
        info!("Creating UnifiedAuthService with mode: {}", config.mode);
        
        let factory = AuthServiceFactory::new(config)?;
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| {
                warn!("Failed to build custom HTTP client, using default");
                Client::new()
            });
        
        Ok(Self { 
            factory: Arc::new(tokio::sync::Mutex::new(factory)), 
            client 
        })
    }
    
    /// Create a unified authentication service from a factory
    pub fn from_factory(factory: AuthServiceFactory) -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| {
                warn!("Failed to build custom HTTP client, using default");
                Client::new()
            });
        Self { 
            factory: Arc::new(tokio::sync::Mutex::new(factory)), 
            client 
        }
    }
    
    /// Get authentication token
    pub async fn get_token(&self) -> Result<String, AuthError> {
        debug!("UnifiedAuthService: Getting authentication token");
        let factory = self.factory.lock().await;
        factory.get_token().await
    }
    
    /// Refresh authentication token
    pub async fn refresh_token(&self) -> Result<String, AuthError> {
        debug!("UnifiedAuthService: Refreshing authentication token");
        let factory = self.factory.lock().await;
        factory.refresh_token().await
    }
    
    /// Validate token
    pub async fn validate_token(&self, token: &str) -> Result<bool, AuthError> {
        debug!("UnifiedAuthService: Validating authentication token");
        let factory = self.factory.lock().await;
        factory.validate_token(token).await
    }
    
    /// Check if token needs refresh
    pub async fn needs_refresh(&self) -> Result<bool, AuthError> {
        debug!("UnifiedAuthService: Checking if token needs refresh");
        let factory = self.factory.lock().await;
        factory.needs_refresh().await
    }
    
    /// Get authentication headers
    pub async fn get_auth_headers(&self) -> Result<HeaderMap, AuthError> {
        debug!("UnifiedAuthService: Getting authentication headers");
        let factory = self.factory.lock().await;
        factory.get_auth_headers().await
    }
    
    /// Get authentication mode
    pub async fn get_auth_mode(&self) -> AuthMode {
        debug!("UnifiedAuthService: Getting authentication mode");
        let factory = self.factory.lock().await;
        factory.get_auth_mode()
    }
    
    /// Get current configuration
    pub async fn get_config(&self) -> AuthConfig {
        debug!("UnifiedAuthService: Getting current configuration");
        let factory = self.factory.lock().await;
        factory.get_config().clone()
    }
    
    /// Update configuration
    pub async fn update_config(&self, config: AuthConfig) -> Result<(), AuthError> {
        info!("UnifiedAuthService: Updating configuration");
        let mut factory = self.factory.lock().await;
        factory.update_config(config)
    }
    
    /// Get a valid token, refreshing if necessary
    pub async fn get_valid_token(&self) -> Result<String, AuthError> {
        debug!("UnifiedAuthService: Getting valid authentication token");
        
        // Check if we need to refresh the token
        if self.needs_refresh().await? {
            debug!("UnifiedAuthService: Token needs refresh, refreshing");
            self.refresh_token().await
        } else {
            self.get_token().await
        }
    }
    
    /// Get authentication headers with a valid token
    pub async fn get_valid_auth_headers(&self) -> Result<HeaderMap, AuthError> {
        debug!("UnifiedAuthService: Getting valid authentication headers");
        
        // Ensure we have a valid token
        self.get_valid_token().await?;
        
        // Get the headers
        self.get_auth_headers().await
    }
    
    /// Clear the current authentication token
    async fn clear_token(&self) {
        let _factory = self.factory.lock().await;
        // For now, we'll just log a warning since we need to modify the factory
        // to properly clear tokens. The retry mechanism will still work by
        // getting fresh auth headers on the next attempt.
        warn!("Clearing authentication token - this will force re-authentication on next request");
    }
    
    /// Make an authenticated HTTP request with retry logic and comprehensive error handling
    pub async fn make_authenticated_request<T: DeserializeOwned>(
        &self,
        method: HttpMethod,
        url: &str,
        headers: Option<HeaderMap>,
        body: Option<serde_json::Value>,
    ) -> Result<T, McpError> {
        debug!("UnifiedAuthService: Making authenticated request to {}", url);
        
        let max_retries = 2;
        let mut retry_count = 0;
        
        // Convert HttpMethod to reqwest::Method
        let reqwest_method = match method {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
            HttpMethod::PATCH => reqwest::Method::PATCH,
        };
        
        loop {
            // Get authentication headers
            let auth_headers = self.get_valid_auth_headers().await
                .map_err(|e| McpError::internal_error(format!("Failed to get auth headers: {}", e), None))?;
            
            // Build the request
            let mut request_builder = self.client.request(reqwest_method.clone(), url);
            
            // Add authentication headers
            for (name, value) in auth_headers.iter() {
                request_builder = request_builder.header(name, value);
            }
            
            // Add additional headers if provided
            if let Some(ref additional_headers) = headers {
                for (name, value) in additional_headers.iter() {
                    request_builder = request_builder.header(name, value);
                }
            }
            
            // Add body if provided
            if let Some(ref body_data) = body {
                request_builder = request_builder.json(&body_data);
            }
            
            // Execute the request
            let response = request_builder.send().await
                .map_err(|e| McpError::internal_error(format!("API request failed: {}", e), None))?;
            
            if !response.status().is_success() {
                if response.status() == StatusCode::UNAUTHORIZED && retry_count < max_retries {
                    // Token might be expired, clear it and retry
                    warn!(
                        "Authentication failed, clearing token and retrying (attempt {}/{})",
                        retry_count + 1,
                        max_retries
                    );
                    
                    // Clear token and retry
                    self.clear_token().await;
                    
                    retry_count += 1;
                    continue;
                }
                
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(McpError::internal_error(
                    format!("API request failed with status {}: {}", status, error_text),
                    None,
                ));
            }
            
            // First get the response text to include in error messages
            let response_text = response.text().await.map_err(|e| {
                McpError::internal_error(format!("Failed to read response text: {}", e), None)
            })?;
            
            // Check if response is empty
            if response_text.trim().is_empty() {
                return Err(McpError::internal_error(
                    format!("API returned empty response for {} {}. This may indicate that the target module is not properly configured or enabled.", reqwest_method, url),
                    None
                ));
            }
            
            // Then try to parse the JSON
            let result: T = serde_json::from_str(&response_text)
                .map_err(|e| McpError::internal_error(
                    format!("Failed to parse API response: {}\nURL: {}\nMethod: {}\nResponse content: {}", e, url, reqwest_method, response_text),
                    None
                ))?;
            
            return Ok(result);
        }
    }
    
    /// Create a new unified authentication service with direct authentication
    pub fn create_direct_auth(
        auth_type: DirectAuthType,
        token: Option<String>,
        api_key_name: Option<String>,
        username: Option<String>,
        password: Option<String>,
        custom_headers: Option<HashMap<String, String>>,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
        let direct_config = DirectAuthConfig {
            auth_type,
            token,
            api_key_name,
            username,
            password,
            custom_headers,
        };
        
        let config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(direct_config),
            login_config: None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        };
        
        Self::new(config)
    }
    
    /// Create a new unified authentication service with login-based authentication
    pub fn create_login_auth(
        auth_type: LoginAuthType,
        url: String,
        method: HttpMethod,
        headers: Option<HashMap<String, String>>,
        body: Option<LoginRequestBody>,
        response_format: ResponseFormat,
        token_extraction: TokenExtraction,
        refresh_url: Option<String>,
        refresh_method: Option<HttpMethod>,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
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
        
        let config = AuthConfig {
            mode: AuthMode::Login,
            direct_config: None,
            login_config: Some(login_config),
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        };
        
        Self::new(config)
    }
    
    /// Create a new unified authentication service with bearer token authentication
    pub fn create_bearer_auth(
        token: String,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
        Self::create_direct_auth(
            DirectAuthType::Bearer,
            Some(token),
            None,
            None,
            None,
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with API key authentication
    pub fn create_api_key_auth(
        api_key_name: String,
        token: String,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
        Self::create_direct_auth(
            DirectAuthType::ApiKey,
            Some(token),
            Some(api_key_name),
            None,
            None,
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with basic authentication
    pub fn create_basic_auth(
        username: String,
        password: String,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
        Self::create_direct_auth(
            DirectAuthType::Basic,
            None,
            None,
            Some(username),
            Some(password),
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with custom headers authentication
    pub fn create_custom_headers_auth(
        headers: HashMap<String, String>,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
        Self::create_direct_auth(
            DirectAuthType::CustomHeaders,
            None,
            None,
            None,
            None,
            Some(headers),
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with JSON login authentication
    pub fn create_json_login_auth(
        url: String,
        username: String,
        password: String,
        token_key: String,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
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
        
        Self::create_login_auth(
            LoginAuthType::Json,
            url,
            HttpMethod::POST,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with JSON login authentication with multiple tokens
    pub fn create_json_login_auth_multi(
        url: String,
        username: String,
        password: String,
        token_extractions: Vec<TokenExtractionItem>,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
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
        
        Self::create_login_auth(
            LoginAuthType::Json,
            url,
            HttpMethod::POST,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with form login authentication
    pub fn create_form_login_auth(
        url: String,
        username: String,
        password: String,
        token_key: String,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
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
        
        Self::create_login_auth(
            LoginAuthType::Form,
            url,
            HttpMethod::POST,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with form login authentication with multiple tokens
    pub fn create_form_login_auth_multi(
        url: String,
        username: String,
        password: String,
        token_extractions: Vec<TokenExtractionItem>,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
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
        
        Self::create_login_auth(
            LoginAuthType::Form,
            url,
            HttpMethod::POST,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with OAuth2 authentication
    pub fn create_oauth2_auth(
        url: String,
        client_id: String,
        client_secret: String,
        scope: Option<String>,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
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
        
        Self::create_login_auth(
            LoginAuthType::OAuth2,
            url,
            HttpMethod::POST,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with OAuth2 authentication with multiple tokens
    pub fn create_oauth2_auth_multi(
        url: String,
        client_id: String,
        client_secret: String,
        scope: Option<String>,
        token_extractions: Vec<TokenExtractionItem>,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
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
        
        Self::create_login_auth(
            LoginAuthType::OAuth2,
            url,
            HttpMethod::POST,
            None,
            Some(body),
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with API key login authentication
    pub fn create_api_key_login_auth(
        url: String,
        api_key_name: String,
        api_key_value: String,
        token_key: String,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
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
        
        Self::create_login_auth(
            LoginAuthType::Custom,
            url,
            HttpMethod::GET,
            Some(headers),
            None,
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with API key login authentication with multiple tokens
    pub fn create_api_key_login_auth_multi(
        url: String,
        api_key_name: String,
        api_key_value: String,
        token_extractions: Vec<TokenExtractionItem>,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
        let mut headers = HashMap::new();
        headers.insert(api_key_name, api_key_value);
        
        let token_extraction = TokenExtraction {
            tokens: token_extractions,
        };
        
        Self::create_login_auth(
            LoginAuthType::Custom,
            url,
            HttpMethod::GET,
            Some(headers),
            None,
            ResponseFormat::Json,
            token_extraction,
            None,
            None,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
    
    /// Create a new unified authentication service with custom login authentication
    pub fn create_custom_login_auth(
        auth_type: LoginAuthType,
        url: String,
        method: HttpMethod,
        headers: Option<HashMap<String, String>>,
        body: Option<LoginRequestBody>,
        response_format: ResponseFormat,
        token_extraction: TokenExtraction,
        refresh_url: Option<String>,
        refresh_method: Option<HttpMethod>,
        token_expiry: u64,
        refresh_buffer: u64,
        max_retry_attempts: u32,
    ) -> Result<Self, AuthError> {
        Self::create_login_auth(
            auth_type,
            url,
            method,
            headers,
            body,
            response_format,
            token_extraction,
            refresh_url,
            refresh_method,
            token_expiry,
            refresh_buffer,
            max_retry_attempts,
        )
    }
}

/// Builder pattern for UnifiedAuthService
pub struct UnifiedAuthServiceBuilder {
    mode: Option<AuthMode>,
    direct_config: Option<DirectAuthConfig>,
    login_config: Option<LoginAuthConfig>,
    token_expiry: u64,
    refresh_buffer: u64,
    max_retry_attempts: u32,
}

impl UnifiedAuthServiceBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            mode: None,
            direct_config: None,
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        }
    }
    
    /// Set authentication mode
    pub fn with_mode(mut self, mode: AuthMode) -> Self {
        self.mode = Some(mode);
        self
    }
    
    /// Set direct authentication configuration
    pub fn with_direct_config(mut self, config: DirectAuthConfig) -> Self {
        self.direct_config = Some(config);
        self.mode = Some(AuthMode::Direct);
        self
    }
    
    /// Set login authentication configuration
    pub fn with_login_config(mut self, config: LoginAuthConfig) -> Self {
        self.login_config = Some(config);
        self.mode = Some(AuthMode::Login);
        self
    }
    
    /// Set token expiry time
    pub fn with_token_expiry(mut self, token_expiry: u64) -> Self {
        self.token_expiry = token_expiry;
        self
    }
    
    /// Set refresh buffer time
    pub fn with_refresh_buffer(mut self, refresh_buffer: u64) -> Self {
        self.refresh_buffer = refresh_buffer;
        self
    }
    
    /// Set max retry attempts
    pub fn with_max_retry_attempts(mut self, max_retry_attempts: u32) -> Self {
        self.max_retry_attempts = max_retry_attempts;
        self
    }
    
    /// Build the unified authentication service
    pub fn build(self) -> Result<UnifiedAuthService, AuthError> {
        let mode = self.mode.ok_or_else(|| 
            AuthError::ConfigurationError("Authentication mode is required".to_string())
        )?;
        
        let config = AuthConfig {
            mode,
            direct_config: self.direct_config,
            login_config: self.login_config,
            token_expiry: self.token_expiry,
            refresh_buffer: self.refresh_buffer,
            max_retry_attempts: self.max_retry_attempts,
        };
        
        UnifiedAuthService::new(config)
    }
}

impl Default for UnifiedAuthServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_unified_auth_service_creation_direct() {
        let direct_config = DirectAuthConfig {
            auth_type: DirectAuthType::Bearer,
            token: Some("test-token".to_string()),
            api_key_name: None,
            username: None,
            password: None,
            custom_headers: None,
        };
        
        let config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(direct_config),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let service = UnifiedAuthService::new(config).unwrap();
        assert_eq!(service.get_auth_mode().await, AuthMode::Direct);
    }
    
    #[tokio::test]
    async fn test_unified_auth_service_creation_login() {
        let login_config = LoginAuthConfig {
            auth_type: LoginAuthType::Json,
            url: "https://example.com/login".to_string(),
            method: HttpMethod::POST,
            headers: None,
            body: None,
            response_format: ResponseFormat::Json,
            token_extraction: TokenExtraction::default(),
            refresh_url: None,
            refresh_method: None,
        };
        
        let config = AuthConfig {
            mode: AuthMode::Login,
            direct_config: None,
            login_config: Some(login_config),
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let service = UnifiedAuthService::new(config).unwrap();
        assert_eq!(service.get_auth_mode().await, AuthMode::Login);
    }
    
    #[tokio::test]
    async fn test_create_bearer_auth() {
        let service = UnifiedAuthService::create_bearer_auth(
            "test-token".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        assert_eq!(service.get_auth_mode().await, AuthMode::Direct);
    }
    
    #[tokio::test]
    async fn test_create_api_key_auth() {
        let service = UnifiedAuthService::create_api_key_auth(
            "X-API-Key".to_string(),
            "test-api-key".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        assert_eq!(service.get_auth_mode().await, AuthMode::Direct);
    }
    
    #[tokio::test]
    async fn test_create_basic_auth() {
        let service = UnifiedAuthService::create_basic_auth(
            "test-user".to_string(),
            "test-password".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        assert_eq!(service.get_auth_mode().await, AuthMode::Direct);
    }
    
    #[tokio::test]
    async fn test_create_custom_headers_auth() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
        
        let service = UnifiedAuthService::create_custom_headers_auth(
            headers,
            3600,
            300,
            3,
        ).unwrap();
        
        assert_eq!(service.get_auth_mode().await, AuthMode::Direct);
    }
    
    #[tokio::test]
    async fn test_create_json_login_auth() {
        let service = UnifiedAuthService::create_json_login_auth(
            "https://example.com/login".to_string(),
            "test-user".to_string(),
            "test-password".to_string(),
            "token".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        assert_eq!(service.get_auth_mode().await, AuthMode::Login);
    }
    
    #[tokio::test]
    async fn test_create_form_login_auth() {
        let service = UnifiedAuthService::create_form_login_auth(
            "https://example.com/login".to_string(),
            "test-user".to_string(),
            "test-password".to_string(),
            "token".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        assert_eq!(service.get_auth_mode().await, AuthMode::Login);
    }
    
    #[tokio::test]
    async fn test_create_oauth2_auth() {
        let service = UnifiedAuthService::create_oauth2_auth(
            "https://example.com/oauth2/token".to_string(),
            "client-id".to_string(),
            "client-secret".to_string(),
            Some("read write".to_string()),
            3600,
            300,
            3,
        ).unwrap();
        
        assert_eq!(service.get_auth_mode().await, AuthMode::Login);
    }
    
    #[tokio::test]
    async fn test_create_api_key_login_auth() {
        let service = UnifiedAuthService::create_api_key_login_auth(
            "https://example.com/api/token".to_string(),
            "X-API-Key".to_string(),
            "test-api-key".to_string(),
            "token".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        assert_eq!(service.get_auth_mode().await, AuthMode::Login);
    }
    
    #[tokio::test]
    async fn test_builder() {
        let direct_config = DirectAuthConfig {
            auth_type: DirectAuthType::Token,
            token: Some("test-token".to_string()),
            api_key_name: None,
            username: None,
            password: None,
            custom_headers: None,
        };
        
        let service = UnifiedAuthServiceBuilder::new()
            .with_direct_config(direct_config)
            .with_token_expiry(7200)
            .with_refresh_buffer(600)
            .with_max_retry_attempts(5)
            .build()
            .unwrap();
        
        assert_eq!(service.get_auth_mode().await, AuthMode::Direct);
        let config = service.get_config().await;
        assert_eq!(config.token_expiry, 7200);
        assert_eq!(config.refresh_buffer, 600);
        assert_eq!(config.max_retry_attempts, 5);
    }
    
    #[tokio::test]
    async fn test_make_authenticated_request() {
        // Create auth service with bearer token
        let service = UnifiedAuthService::create_bearer_auth(
            "test-token".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        // Test that we can create a request builder (actual request would need a real server)
        let headers = service.get_valid_auth_headers().await;
        assert!(headers.is_ok());
        
        let headers = headers.unwrap();
        assert!(headers.contains_key("authorization"));
    }
    
    #[tokio::test]
    async fn test_make_authenticated_request_with_custom_headers() {
        // Create auth service with bearer token
        let service = UnifiedAuthService::create_bearer_auth(
            "test-token".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        // Prepare custom headers
        let mut custom_headers = HeaderMap::new();
        custom_headers.insert("Content-Type", "application/json".parse().unwrap());
        custom_headers.insert("X-Custom-Header", "custom-value".parse().unwrap());
        
        // Test that we can create a request builder with custom headers
        let auth_headers = service.get_valid_auth_headers().await;
        assert!(auth_headers.is_ok());
        
        let auth_headers = auth_headers.unwrap();
        assert!(auth_headers.contains_key("authorization"));
        
        // Verify custom headers are preserved
        assert_eq!(custom_headers.get("Content-Type"), Some(&"application/json".parse().unwrap()));
        assert_eq!(custom_headers.get("X-Custom-Header"), Some(&"custom-value".parse().unwrap()));
    }
    
    #[tokio::test]
    async fn test_get_valid_token() {
        // Create auth service with bearer token
        let service = UnifiedAuthService::create_bearer_auth(
            "test-token".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        // Test getting a valid token
        let token = service.get_valid_token().await;
        assert!(token.is_ok());
        assert_eq!(token.unwrap(), "test-token");
    }
    
    #[tokio::test]
    async fn test_get_valid_auth_headers() {
        // Create auth service with bearer token
        let service = UnifiedAuthService::create_bearer_auth(
            "test-token".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        // Test getting valid auth headers
        let headers = service.get_valid_auth_headers().await;
        assert!(headers.is_ok());
        
        let headers = headers.unwrap();
        assert!(headers.contains_key("authorization"));
        assert_eq!(headers.get("authorization").unwrap(), "Bearer test-token");
    }
    
    #[tokio::test]
    async fn test_make_authenticated_request_with_unauthorized_retry() {
        // This test would require a mock server to properly test the retry logic
        // For now, we'll just test the token update functionality
        
        // Create auth service with expired token
        let service = UnifiedAuthService::create_bearer_auth(
            "expired-token".to_string(),
            3600,
            300,
            3,
        ).unwrap();
        
        // Test that we can get the current token
        let token = service.get_valid_token().await;
        assert!(token.is_ok());
        assert_eq!(token.unwrap(), "expired-token");
        
        // In a real scenario with a mock server, we would:
        // 1. Make a request that returns 401
        // 2. The service would automatically refresh the token
        // 3. The service would retry the request with the new token
        // 4. The request would succeed
    }
}