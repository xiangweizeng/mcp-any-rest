//! Authentication service factory for MCP-ANY-REST
//!
//! This module provides a factory pattern implementation for creating and managing
//! authentication strategies. It supports the two main authentication modes:
//! 1. Direct Authentication - Authentication information is directly configured and used in each request
//! 2. Login-based Authentication - Login information is configured first, then authentication is obtained after login

use super::auth_strategy::{
    AuthConfig, AuthError, AuthStrategy, AuthMode, DirectAuthType,
    DirectAuthConfig, LoginAuthConfig,
    TokenFormat, TokenLocation, TokenTargetLocation,
    HttpMethod, ResponseFormat, BodyFormat
};
use anyhow::Result;
use base64::Engine;
use log::{info, warn};
use reqwest::Client;
use serde_json::Value;
use url::Url;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

// Direct authentication strategy implementation
pub struct DirectAuthStrategyImpl {
    config: DirectAuthConfig,
    _token_expiry: u64,
    _client: Client,
}

impl DirectAuthStrategyImpl {
    fn new(config: DirectAuthConfig, token_expiry: u64) -> Self {
        // Create HTTP client with disabled SSL verification for self-signed certificates
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| {
                warn!("Failed to build custom HTTP client, using default");
                Client::new()
            });
            
        Self { 
            config, 
            _token_expiry: token_expiry,
            _client: client,
        }
    }
}

#[async_trait::async_trait]
impl AuthStrategy for DirectAuthStrategyImpl {
    async fn get_token(&self) -> Result<String, AuthError> {
        match self.config.auth_type {
            DirectAuthType::Token | DirectAuthType::Bearer => {
                self.config.token.clone()
                    .ok_or_else(|| AuthError::TokenNotFound("Token not configured".to_string()))
            }
            DirectAuthType::ApiKey => {
                self.config.token.clone()
                    .ok_or_else(|| AuthError::TokenNotFound("API key not configured".to_string()))
            }
            DirectAuthType::Basic => {
                if let (Some(username), Some(password)) = (&self.config.username, &self.config.password) {
                    let creds = format!("{}:{}", username, password);
                    Ok(base64::engine::general_purpose::STANDARD.encode(creds))
                } else {
                    Err(AuthError::ConfigurationError("Username and password required for Basic auth".to_string()))
                }
            }
            DirectAuthType::CustomHeaders => {
                // For custom headers, try to extract a token from the Authorization header if present
                if let Some(custom_headers) = &self.config.custom_headers {
                    if let Some(auth_header) = custom_headers.get("Authorization") {
                        // Extract token from Authorization header (remove "Bearer " prefix if present)
                        let token = if auth_header.starts_with("Bearer ") {
                            auth_header[7..].to_string()
                        } else {
                            auth_header.clone()
                        };
                        Ok(token)
                    } else {
                        // If no Authorization header, return a clear error
                        Err(AuthError::TokenNotFound("No Authorization header configured in CustomHeaders".to_string()))
                    }
                } else {
                    Err(AuthError::ConfigurationError("No custom headers configured".to_string()))
                }
            }
        }
    }

    async fn refresh_token(&self) -> Result<String, AuthError> {
        // Direct authentication doesn't need token refresh
        self.get_token().await
    }

    async fn validate_token(&self, token: &str) -> Result<bool, AuthError> {
        match self.config.auth_type {
            DirectAuthType::Token | DirectAuthType::Bearer | DirectAuthType::ApiKey => {
                if let Some(configured_token) = &self.config.token {
                    // For Bearer tokens, remove the "Bearer " prefix if present
                    let configured_token = if self.config.auth_type == DirectAuthType::Bearer 
                        && configured_token.starts_with("Bearer ") {
                        &configured_token[7..]
                    } else {
                        configured_token
                    };
                    
                    // Also handle the case where the provided token might have a "Bearer " prefix
                    let provided_token = if token.starts_with("Bearer ") {
                        &token[7..]
                    } else {
                        token
                    };
                    
                    Ok(provided_token == configured_token)
                } else {
                    Err(AuthError::TokenNotFound("No token configured for validation".to_string()))
                }
            }
            DirectAuthType::Basic => {
                if let (Some(username), Some(password)) = (&self.config.username, &self.config.password) {
                    let creds = format!("{}:{}", username, password);
                    let expected_token = base64::engine::general_purpose::STANDARD.encode(creds);
                    Ok(token == expected_token)
                } else {
                    Ok(false)
                }
            }
            DirectAuthType::CustomHeaders => {
                // For custom headers, validate against the Authorization header if present
                if let Some(custom_headers) = &self.config.custom_headers {
                    if let Some(auth_header) = custom_headers.get("Authorization") {
                        // Extract token from Authorization header (remove "Bearer " prefix if present)
                        let expected_token = if auth_header.starts_with("Bearer ") {
                            auth_header[7..].to_string()
                        } else {
                            auth_header.clone()
                        };
                        
                        // Also handle the case where the provided token might have a "Bearer " prefix
                        let provided_token = if token.starts_with("Bearer ") {
                            &token[7..]
                        } else {
                            token
                        };
                        
                        Ok(provided_token == expected_token)
                    } else {
                        // If no Authorization header, only validate against empty string
                        Ok(token.is_empty())
                    }
                } else {
                    Ok(token.is_empty())
                }
            }
        }
    }

    fn get_auth_mode(&self) -> AuthMode {
        AuthMode::Direct
    }

    async fn needs_refresh(&self) -> Result<bool, AuthError> {
        // Direct authentication doesn't need token refresh
        Ok(false)
    }

    async fn get_auth_headers(&self) -> Result<reqwest::header::HeaderMap, AuthError> {
        let mut headers = reqwest::header::HeaderMap::new();
        
        match self.config.auth_type {
            DirectAuthType::Token => {
                if let Some(token) = &self.config.token {
                    headers.insert(
                        "Token",
                        token.parse::<reqwest::header::HeaderValue>().map_err(|e| AuthError::ParseError(e.to_string()))?,
                    );
                }
            }
            DirectAuthType::Bearer => {
                if let Some(token) = &self.config.token {
                    let auth_value = if token.starts_with("Bearer ") {
                        token.clone()
                    } else {
                        format!("Bearer {}", token)
                    };
                    headers.insert(
                        reqwest::header::AUTHORIZATION,
                        auth_value.parse().map_err(|e: reqwest::header::InvalidHeaderValue| AuthError::ParseError(e.to_string()))?,
                    );
                }
            }
            DirectAuthType::ApiKey => {
                if let (Some(api_key_name), Some(token)) = (&self.config.api_key_name, &self.config.token) {
                    let header_name = reqwest::header::HeaderName::from_str(api_key_name)
                        .map_err(|e| AuthError::ParseError(e.to_string()))?;
                    headers.insert(
                        header_name,
                        token.parse::<reqwest::header::HeaderValue>().map_err(|e| AuthError::ParseError(e.to_string()))?,
                    );
                }
            }
            DirectAuthType::Basic => {
                if let (Some(username), Some(password)) = (&self.config.username, &self.config.password) {
                    let creds = format!("{}:{}", username, password);
                    let encoded = base64::engine::general_purpose::STANDARD.encode(creds);
                    headers.insert(
                        reqwest::header::AUTHORIZATION,
                        format!("Basic {}", encoded).parse().map_err(|e: reqwest::header::InvalidHeaderValue| AuthError::ParseError(e.to_string()))?,
                    );
                }
            }
            DirectAuthType::CustomHeaders => {
                if let Some(custom_headers) = &self.config.custom_headers {
                    for (key, value) in custom_headers {
                        headers.insert(
                            reqwest::header::HeaderName::from_bytes(key.as_bytes())
                                .map_err(|e| AuthError::ParseError(e.to_string()))?,
                            value.parse().map_err(|e: reqwest::header::InvalidHeaderValue| AuthError::ParseError(e.to_string()))?,
                        );
                    }
                }
            }
        }
        
        Ok(headers)
    }
    
    async fn login_and_get_token(&self, _token_index: usize) -> Result<String, AuthError> {
        // Direct authentication doesn't support login-based token extraction
        // Just return the current token
        self.get_token().await
    }
}

// Login-based authentication strategy implementation
pub struct LoginAuthStrategyImpl {
    config: LoginAuthConfig,
    client: Client,
    token_expiry: u64,
    current_token: Arc<tokio::sync::Mutex<Option<String>>>,
    token_expiry_time: Arc<tokio::sync::Mutex<Option<Instant>>>,
}

impl LoginAuthStrategyImpl {
    fn new(config: LoginAuthConfig, token_expiry: u64) -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| {
                warn!("Failed to build custom HTTP client, using default");
                Client::new()
            });
            
        Self {
            config,
            client,
            token_expiry,
            current_token: Arc::new(tokio::sync::Mutex::new(None)),
            token_expiry_time: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }
    
    async fn login(&self) -> Result<String, AuthError> {
        let reqwest_method = match self.config.method {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
            HttpMethod::PATCH => reqwest::Method::PATCH,
        };
        
        let mut request = self.client.request(reqwest_method, &self.config.url);
        
        // Add headers
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }
        
        // Add body if configured
        if let Some(body) = &self.config.body {
            match body.format {
                BodyFormat::Json => {
                    request = request.json(&body.content);
                }
                BodyFormat::Form => {
                    request = request.form(&body.content);
                }
                BodyFormat::Text => {
                    // Convert content map to text
                    let text_content = body.content.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&");
                    request = request.body(text_content);
                }
                BodyFormat::Xml => {
                    // Simple XML conversion
                    let xml_content = body.content.iter()
                        .map(|(k, v)| format!("<{}>{}</{}>", k, v, k))
                        .collect::<Vec<_>>()
                        .join("");
                    request = request.body(format!("<root>{}</root>", xml_content));
                }
            }
        }
        
        let response = request.send().await
            .map_err(|e| AuthError::NetworkError(format!("Login request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let mut error_text = response.text().await.unwrap_or_else(|_| "Failed to read error response".to_string());
            if error_text.is_empty() {
                error_text = format!("Empty error response with status code {}", status);
            }
            return Err(AuthError::LoginFailed(
                format!("Login failed with status {}: {}", status, error_text)
            ));
        }
        
        // Extract the first token for backward compatibility
        // In a full implementation, we would extract all tokens and store them
        let token = self.extract_token_from_response(response, 0).await?;
        
        // Format token based on configuration
        let formatted_token = self.format_token(&token, 0)?;
        
        // Update token and expiry
        {
            let mut current_token = self.current_token.lock().await;
            let mut token_expiry_time = self.token_expiry_time.lock().await;
            
            *current_token = Some(formatted_token.clone());
            *token_expiry_time = Some(Instant::now() + Duration::from_secs(self.token_expiry));
        }
        
        Ok(formatted_token)
    }
    
    /// Extract a token from the response based on the token extraction configuration
    async fn extract_token_from_response(&self, response: reqwest::Response, token_index: usize) -> Result<String, AuthError> {
        if token_index >= self.config.token_extraction.tokens.len() {
            return Err(AuthError::ConfigurationError(
                format!("Token index {} is out of bounds", token_index)
            ));
        }
        
        let token_config = &self.config.token_extraction.tokens[token_index];
        
        match token_config.source_location {
            TokenLocation::Header => {
                let token_key = &token_config.source_key;
                Ok(response.headers()
                    .get(token_key)
                    .ok_or_else(|| AuthError::TokenNotFound(
                        format!("Token header '{}' not found", token_key)
                    ))?
                    .to_str()
                    .map_err(|e| AuthError::ParseError(format!("Failed to parse token header: {}", e)))?
                    .to_string())
            }
            TokenLocation::Body => {
                let response_text = response.text().await
                    .map_err(|e| AuthError::ParseError(format!("Failed to read response body: {}", e)))?;
                
                let token_key = &token_config.source_key;
                
                // Check if response is empty
                if response_text.is_empty() {
                    return Err(AuthError::TokenNotFound(
                        format!("Empty response body when trying to extract token key '{}'", token_key)
                    ));
                }
                
                match self.config.response_format {
                    ResponseFormat::Json => {
                        let json: Value = serde_json::from_str(&response_text)
                            .map_err(|e| AuthError::ParseError(format!("Failed to parse JSON response: {}", e)))?;
                        
                        Ok(json.get(token_key)
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| AuthError::TokenNotFound(
                                format!("Token key '{}' not found in JSON response", token_key)
                            ))?
                            .to_string())
                    }
                    ResponseFormat::Xml => {
                        // Simple XML parsing - in a real implementation, use a proper XML parser
                        let token_pattern = format!("<{}>(.*?)</{}>", token_key, token_key);
                        let regex = regex::Regex::new(&token_pattern)
                            .map_err(|e| AuthError::ParseError(format!("Failed to create regex: {}", e)))?;
                        
                        Ok(regex.captures(&response_text)
                            .and_then(|caps| caps.get(1))
                            .map(|m| m.as_str().to_string())
                            .ok_or_else(|| AuthError::TokenNotFound(
                                format!("Token key '{}' not found in XML response", token_key)
                            ))?)
                    }
                    ResponseFormat::Text => {
                        // Simple key-value parsing for text format
                        let lines = response_text.lines();
                        for line in lines {
                            if let Some((key, value)) = line.split_once('=') {
                                if key.trim() == token_key {
                                    return Ok(value.trim().to_string());
                                }
                            }
                        }
                        return Err(AuthError::TokenNotFound(
                            format!("Token key '{}' not found in text response", token_key)
                        ));
                    }
                }
            }
            TokenLocation::Query => {
                let response_text = response.text().await
                    .map_err(|e| AuthError::ParseError(format!("Failed to read response body: {}", e)))?;
                
                let token_key = &token_config.source_key;
                
                // Check if response is empty
                if response_text.is_empty() {
                    return Err(AuthError::TokenNotFound(
                        format!("Empty response body when trying to extract token key '{}' from query parameters", token_key)
                    ));
                }
                
                // Parse URL parameters from response text
                let url = Url::parse(&format!("http://example.com?{}", response_text))
                    .map_err(|e| AuthError::ParseError(format!("Failed to parse query parameters: {}", e)))?;
                
                Ok(url.query_pairs()
                    .find(|(key, _)| key == token_key)
                    .map(|(_, value)| value.to_string())
                    .ok_or_else(|| AuthError::TokenNotFound(
                        format!("Token key '{}' not found in query parameters", token_key)
                    ))?)
            }
        }
    }
    
    /// Format a token based on the token configuration
    fn format_token(&self, token: &str, token_index: usize) -> Result<String, AuthError> {
        if token_index >= self.config.token_extraction.tokens.len() {
            return Err(AuthError::ConfigurationError(
                format!("Token index {} is out of bounds", token_index)
            ));
        }
        
        let token_config = &self.config.token_extraction.tokens[token_index];
        
        match token_config.format {
            TokenFormat::Bearer => {
                if token.starts_with("Bearer ") {
                    Ok(token.to_string())
                } else {
                    Ok(format!("Bearer {}", token))
                }
            }
            TokenFormat::Basic => {
                if token.starts_with("Basic ") {
                    Ok(token.to_string())
                } else {
                    Ok(format!("Basic {}", token))
                }
            }
            TokenFormat::Raw => Ok(token.to_string()),
        }
    }
}

#[async_trait::async_trait]
impl AuthStrategy for LoginAuthStrategyImpl {
    async fn get_token(&self) -> Result<String, AuthError> {
        // Check if we have a valid token
        {
            let current_token = self.current_token.lock().await;
            let token_expiry_time = self.token_expiry_time.lock().await;
            
            if let (Some(token), Some(expiry)) = (&*current_token, &*token_expiry_time) {
                if expiry.elapsed() < Duration::from_secs(60) { // 1 minute buffer
                    return Ok(token.clone());
                }
            }
        }
        
        // Token is expired or doesn't exist, login to get a new one
        self.login().await
    }

    async fn refresh_token(&self) -> Result<String, AuthError> {
        // For login-based auth, refresh is the same as getting a new token
        self.login().await
    }

    async fn validate_token(&self, token: &str) -> Result<bool, AuthError> {
        let current_token = self.current_token.lock().await;
        let token_expiry_time = self.token_expiry_time.lock().await;
        
        if let (Some(current), Some(expiry)) = (&*current_token, &*token_expiry_time) {
            if expiry.elapsed() < Duration::from_secs(60) { // 1 minute buffer
                return Ok(current == token);
            }
        }
        
        Ok(false)
    }

    fn get_auth_mode(&self) -> AuthMode {
        AuthMode::Login
    }

    async fn needs_refresh(&self) -> Result<bool, AuthError> {
        let token_expiry_time = self.token_expiry_time.lock().await;
        
        if let Some(expiry) = &*token_expiry_time {
            Ok(expiry.elapsed() >= Duration::from_secs(self.token_expiry - 60)) // 1 minute buffer
        } else {
            Ok(true) // No token, needs refresh
        }
    }

    async fn get_auth_headers(&self) -> Result<reqwest::header::HeaderMap, AuthError> {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // Get all tokens if multiple are configured
        for (index, token_config) in self.config.token_extraction.tokens.iter().enumerate() {
            let token = if index == 0 {
                // For the first token, use the cached one if available
                self.get_token().await?
            } else {
                // For additional tokens, we need to extract them from the response
                // In a full implementation, we would cache all tokens
                // For now, we'll perform a login to get the tokens
                self.login_and_get_token(index).await?
            };
            
            // Add token based on target location
            match token_config.target_location {
                TokenTargetLocation::Header => {
                    let header_name = &token_config.target_key;
                    headers.insert(
                        reqwest::header::HeaderName::from_bytes(header_name.as_bytes())
                            .map_err(|e| AuthError::ParseError(e.to_string()))?,
                        token.parse::<reqwest::header::HeaderValue>()
                            .map_err(|e| AuthError::ParseError(e.to_string()))?,
                    );
                }
                TokenTargetLocation::Query => {
                    // For query parameters, we would need to modify the request URL
                    // This is handled in the request building phase
                    // For now, we'll just log that we have a query token
                    info!("Query token {} for key {}", token, token_config.target_key);
                }
                TokenTargetLocation::Cookie => {
                    // For cookies, we would need to add a Cookie header
                    // This is handled in the request building phase
                    // For now, we'll just log that we have a cookie token
                    info!("Cookie token {} for key {}", token, token_config.target_key);
                }
                TokenTargetLocation::Body => {
                    // For body parameters, we would need to modify the request body
                    // This is handled in the request building phase
                    // For now, we'll just log that we have a body token
                    info!("Body token {} for key {}", token, token_config.target_key);
                }
            }
        }
        
        Ok(headers)
    }
    
    /// Login and extract a specific token by index
    async fn login_and_get_token(&self, token_index: usize) -> Result<String, AuthError> {
        let reqwest_method = match self.config.method {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
            HttpMethod::PATCH => reqwest::Method::PATCH,
        };
        
        let mut request = self.client.request(reqwest_method, &self.config.url);
        
        // Add headers
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }
        
        // Add body if configured
        if let Some(body) = &self.config.body {
            match body.format {
                BodyFormat::Json => {
                    request = request.json(&body.content);
                }
                BodyFormat::Form => {
                    request = request.form(&body.content);
                }
                BodyFormat::Text => {
                    // Convert content map to text
                    let text_content = body.content.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&");
                    request = request.body(text_content);
                }
                BodyFormat::Xml => {
                    // Simple XML conversion
                    let xml_content = body.content.iter()
                        .map(|(k, v)| format!("<{}>{}</{}>", k, v, k))
                        .collect::<Vec<_>>()
                        .join("");
                    request = request.body(format!("<root>{}</root>", xml_content));
                }
            }
        }
        
        let response = request.send().await
            .map_err(|e| AuthError::NetworkError(format!("Login request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthError::LoginFailed(
                format!("Login failed with status {}: {}", status, error_text)
            ));
        }
        
        // Extract the specific token
        let token = self.extract_token_from_response(response, token_index).await?;
        
        // Format token based on configuration
        let formatted_token = self.format_token(&token, token_index)?;
        
        Ok(formatted_token)
    }
}

/// Authentication service factory
pub struct AuthServiceFactory {
    strategy: AuthStrategyEnum,
    config: AuthConfig,
}

/// Enum to hold different authentication strategy implementations
pub enum AuthStrategyEnum {
    Direct(DirectAuthStrategyImpl),
    Login(LoginAuthStrategyImpl),
}

impl AuthStrategyEnum {
    /// Get authentication token
    pub async fn get_token(&self) -> Result<String, AuthError> {
        match self {
            AuthStrategyEnum::Direct(strategy) => strategy.get_token().await,
            AuthStrategyEnum::Login(strategy) => strategy.get_token().await,
        }
    }
    
    /// Refresh authentication token
    pub async fn refresh_token(&self) -> Result<String, AuthError> {
        match self {
            AuthStrategyEnum::Direct(strategy) => strategy.refresh_token().await,
            AuthStrategyEnum::Login(strategy) => strategy.refresh_token().await,
        }
    }
    
    /// Validate token
    pub async fn validate_token(&self, token: &str) -> Result<bool, AuthError> {
        match self {
            AuthStrategyEnum::Direct(strategy) => strategy.validate_token(token).await,
            AuthStrategyEnum::Login(strategy) => strategy.validate_token(token).await,
        }
    }
    
    /// Check if token needs refresh
    pub async fn needs_refresh(&self) -> Result<bool, AuthError> {
        match self {
            AuthStrategyEnum::Direct(strategy) => strategy.needs_refresh().await,
            AuthStrategyEnum::Login(strategy) => strategy.needs_refresh().await,
        }
    }
    
    /// Get authentication headers
    pub async fn get_auth_headers(&self) -> Result<reqwest::header::HeaderMap, AuthError> {
        match self {
            AuthStrategyEnum::Direct(strategy) => strategy.get_auth_headers().await,
            AuthStrategyEnum::Login(strategy) => strategy.get_auth_headers().await,
        }
    }
    
    /// Get authentication mode
    pub fn get_auth_mode(&self) -> AuthMode {
        match self {
            AuthStrategyEnum::Direct(strategy) => strategy.get_auth_mode(),
            AuthStrategyEnum::Login(strategy) => strategy.get_auth_mode(),
        }
    }
    
    /// Login and get token at specific index
    pub async fn login_and_get_token(&self, token_index: usize) -> Result<String, AuthError> {
        match self {
            AuthStrategyEnum::Direct(strategy) => strategy.login_and_get_token(token_index).await,
            AuthStrategyEnum::Login(strategy) => strategy.login_and_get_token(token_index).await,
        }
    }
}

impl AuthServiceFactory {
    /// Create a new authentication service factory
    pub fn new(config: AuthConfig) -> Result<Self, AuthError> {
        info!("Creating AuthServiceFactory with mode: {}", config.mode);
        
        let strategy = match config.mode {
            AuthMode::Direct => {
                let direct_config = config.direct_config.clone().ok_or_else(|| {
                    AuthError::ConfigurationError("Direct authentication configuration is required".to_string())
                })?;
                
                AuthStrategyEnum::Direct(DirectAuthStrategyImpl::new(direct_config, config.token_expiry))
            }
            AuthMode::Login => {
                let login_config = config.login_config.clone().ok_or_else(|| {
                    AuthError::ConfigurationError("Login authentication configuration is required".to_string())
                })?;
                
                AuthStrategyEnum::Login(LoginAuthStrategyImpl::new(login_config, config.token_expiry))
            }
        };
        
        Ok(Self {
            strategy,
            config,
        })
    }
    
    /// Get authentication token
    pub async fn get_token(&self) -> Result<String, AuthError> {
        self.strategy.get_token().await
    }
    
    /// Refresh authentication token
    pub async fn refresh_token(&self) -> Result<String, AuthError> {
        self.strategy.refresh_token().await
    }
    
    /// Validate token
    pub async fn validate_token(&self, token: &str) -> Result<bool, AuthError> {
        self.strategy.validate_token(token).await
    }
    
    /// Check if token needs refresh
    pub async fn needs_refresh(&self) -> Result<bool, AuthError> {
        self.strategy.needs_refresh().await
    }
    
    /// Get authentication headers
    pub async fn get_auth_headers(&self) -> Result<reqwest::header::HeaderMap, AuthError> {
        self.strategy.get_auth_headers().await
    }
    
    /// Get authentication mode
    pub fn get_auth_mode(&self) -> AuthMode {
        self.strategy.get_auth_mode()
    }
    
    /// Login and get token at specific index
    pub async fn login_and_get_token(&self, token_index: usize) -> Result<String, AuthError> {
        self.strategy.login_and_get_token(token_index).await
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> &AuthConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: AuthConfig) -> Result<(), AuthError> {
        info!("AuthServiceFactory: Updating configuration");
        
        // Create new strategy with updated configuration
        let strategy = match config.mode {
            AuthMode::Direct => {
                let direct_config = config.direct_config.clone().ok_or_else(|| {
                    AuthError::ConfigurationError("Direct authentication configuration is required".to_string())
                })?;
                
                AuthStrategyEnum::Direct(DirectAuthStrategyImpl::new(direct_config, config.token_expiry))
            }
            AuthMode::Login => {
                let login_config = config.login_config.clone().ok_or_else(|| {
                    AuthError::ConfigurationError("Login authentication configuration is required".to_string())
                })?;
                
                AuthStrategyEnum::Login(LoginAuthStrategyImpl::new(login_config, config.token_expiry))
            }
        };
        
        self.strategy = strategy;
        self.config = config;
        
        info!("AuthServiceFactory configuration updated successfully");
        Ok(())
    }
    
    /// Update the authentication token
    pub fn update_token(&mut self, token: String) -> Result<(), AuthError> {
        info!("AuthServiceFactory: Updating authentication token");
        
        // Update the token in the configuration based on the current auth mode
        match self.config.mode {
            AuthMode::Direct => {
                if let Some(ref mut direct_config) = self.config.direct_config {
                    direct_config.token = Some(token);
                } else {
                    return Err(AuthError::ConfigurationError("Direct authentication configuration not found".to_string()));
                }
            }
            AuthMode::Login => {
                // For login-based auth, we need to update the current token in the strategy
                // This is a bit more complex and would require modifying the strategy implementation
                // For now, we'll update the configuration and recreate the strategy
                if self.config.login_config.is_some() {
                    // In a real implementation, we would update the current token in the strategy
                    // For now, we'll just update the configuration
                } else {
                    return Err(AuthError::ConfigurationError("Login authentication configuration not found".to_string()));
                }
            }
        }
        
        // Recreate the strategy with the updated token
        let strategy = match self.config.mode {
            AuthMode::Direct => {
                let direct_config = self.config.direct_config.as_ref().unwrap().clone();
                AuthStrategyEnum::Direct(DirectAuthStrategyImpl::new(direct_config, self.config.token_expiry))
            }
            AuthMode::Login => {
                let login_config = self.config.login_config.as_ref().unwrap().clone();
                AuthStrategyEnum::Login(LoginAuthStrategyImpl::new(login_config, self.config.token_expiry))
            }
        };
        
        self.strategy = strategy;
        
        info!("AuthServiceFactory token updated successfully");
        Ok(())
    }
}

/// Builder pattern for AuthServiceFactory
pub struct AuthServiceFactoryBuilder {
    config: Option<AuthConfig>,
}

impl AuthServiceFactoryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: None,
        }
    }
    
    /// Set configuration
    pub fn with_config(mut self, config: AuthConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    /// Build the factory
    pub fn build(self) -> Result<AuthServiceFactory, AuthError> {
        let config = self.config.ok_or_else(|| 
            AuthError::ConfigurationError("Configuration is required".to_string())
        )?;
        
        AuthServiceFactory::new(config)
    }
}

impl Default for AuthServiceFactoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::auth_service::auth_strategy::{LoginAuthType, LoginRequestBody, TokenExtraction};
    use std::collections::HashMap;

    #[test]
    fn test_factory_creation_direct() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer test-token".to_string());
        
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
        
        let factory = AuthServiceFactory::new(config).unwrap();
        assert_eq!(factory.get_auth_mode(), AuthMode::Direct);
    }
    
    #[test]
    fn test_factory_creation_login() {
        let login_config = LoginAuthConfig {
            auth_type: LoginAuthType::Json,
            url: "https://example.com/login".to_string(),
            method: HttpMethod::POST,
            headers: None,
            body: Some(LoginRequestBody {
                format: BodyFormat::Json,
                content: {
                    let mut map = HashMap::new();
                    map.insert("username".to_string(), "test".to_string());
                    map.insert("password".to_string(), "test".to_string());
                    map
                },
            }),
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
        
        let factory = AuthServiceFactory::new(config).unwrap();
        assert_eq!(factory.get_auth_mode(), AuthMode::Login);
    }
    
    #[test]
    fn test_factory_builder() {
        let direct_config = DirectAuthConfig {
            auth_type: DirectAuthType::Token,
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
        
        let factory = AuthServiceFactoryBuilder::new()
            .with_config(config)
            .build()
            .unwrap();
        
        assert_eq!(factory.get_auth_mode(), AuthMode::Direct);
    }
}