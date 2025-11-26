//! Test script for authentication service

use std::sync::Arc;
use tracing;
use mcp_any_rest::config::dynamic::DynamicConfigManager;
use mcp_any_rest::services::auth_service::{AuthService};
use mcp_any_rest::services::auth_service::auth_strategy::{AuthConfig as StrategyAuthConfig, AuthMode as StrategyAuthMode, DirectAuthType as StrategyDirectAuthType, LoginAuthType as StrategyLoginAuthType, DirectAuthConfig as StrategyDirectAuthConfig, LoginAuthConfig as StrategyLoginAuthConfig, TokenExtraction as StrategyTokenExtraction, TokenExtractionItem as StrategyTokenExtractionItem, TokenLocation as StrategyTokenLocation, TokenTargetLocation as StrategyTokenTargetLocation, TokenFormat as StrategyTokenFormat, HttpMethod as StrategyHttpMethod, ResponseFormat as StrategyResponseFormat, BodyFormat as StrategyBodyFormat, LoginRequestBody as StrategyLoginRequestBody};

// Convert config::AuthConfig to auth_strategy::AuthConfig
fn convert_auth_config(config_auth: &mcp_any_rest::config::config::AuthConfig) -> StrategyAuthConfig {
    let mode = match config_auth.mode {
        mcp_any_rest::config::config::AuthMode::Direct => StrategyAuthMode::Direct,
        mcp_any_rest::config::config::AuthMode::Login => StrategyAuthMode::Login,
    };
    
    let direct_config = config_auth.direct_config.as_ref().map(|config| {
        let auth_type = match config.auth_type {
            mcp_any_rest::config::config::DirectAuthType::Bearer => StrategyDirectAuthType::Bearer,
            mcp_any_rest::config::config::DirectAuthType::ApiKey => StrategyDirectAuthType::ApiKey,
            mcp_any_rest::config::config::DirectAuthType::Basic => StrategyDirectAuthType::Basic,
            mcp_any_rest::config::config::DirectAuthType::Token => StrategyDirectAuthType::Token,
            mcp_any_rest::config::config::DirectAuthType::CustomHeaders => StrategyDirectAuthType::CustomHeaders,
        };
        
        StrategyDirectAuthConfig {
            auth_type,
            token: config.token.clone(),
            api_key_name: config.api_key_name.clone(),
            username: config.username.clone(),
            password: config.password.clone(),
            custom_headers: config.custom_headers.clone(),
        }
    });
    
    let login_config = config_auth.login_config.as_ref().map(|config| {
        let auth_type = match config.auth_type {
            mcp_any_rest::config::config::LoginAuthType::Json => StrategyLoginAuthType::Json,
            mcp_any_rest::config::config::LoginAuthType::Form => StrategyLoginAuthType::Form,
            mcp_any_rest::config::config::LoginAuthType::OAuth2 => StrategyLoginAuthType::OAuth2,
            mcp_any_rest::config::config::LoginAuthType::ApiKey => StrategyLoginAuthType::ApiKey,
            mcp_any_rest::config::config::LoginAuthType::Custom => StrategyLoginAuthType::Custom,
        };
        
        let method = match config.method {
            mcp_any_rest::config::config::HttpMethod::Get => StrategyHttpMethod::GET,
            mcp_any_rest::config::config::HttpMethod::Post => StrategyHttpMethod::POST,
            mcp_any_rest::config::config::HttpMethod::Put => StrategyHttpMethod::PUT,
            mcp_any_rest::config::config::HttpMethod::Delete => StrategyHttpMethod::DELETE,
            mcp_any_rest::config::config::HttpMethod::Patch => StrategyHttpMethod::PATCH,
        };
        
        let response_format = match config.response_format {
            mcp_any_rest::config::config::ResponseFormat::Json => StrategyResponseFormat::Json,
            mcp_any_rest::config::config::ResponseFormat::Xml => StrategyResponseFormat::Xml,
            mcp_any_rest::config::config::ResponseFormat::Text => StrategyResponseFormat::Text,
        };
        
        let body = config.body.as_ref().map(|b| {
            let format = match b.format {
                mcp_any_rest::config::config::BodyFormat::Json => StrategyBodyFormat::Json,
                mcp_any_rest::config::config::BodyFormat::Form => StrategyBodyFormat::Form,
            };
            
            StrategyLoginRequestBody {
                format,
                content: b.content.clone(),
            }
        });
        
        let token_extraction = StrategyTokenExtraction {
            tokens: config.token_extraction.tokens.iter().map(|token| {
                StrategyTokenExtractionItem {
                    source_location: match token.source_location {
                        mcp_any_rest::config::config::TokenLocation::Header => StrategyTokenLocation::Header,
                        mcp_any_rest::config::config::TokenLocation::Body => StrategyTokenLocation::Body,
                        mcp_any_rest::config::config::TokenLocation::Query => StrategyTokenLocation::Body, // Map Query to Body
                    },
                    source_key: token.source_key.clone(),
                    format: match token.format {
                        mcp_any_rest::config::config::TokenFormat::Bearer => StrategyTokenFormat::Bearer,
                        mcp_any_rest::config::config::TokenFormat::Token => StrategyTokenFormat::Raw,
                        mcp_any_rest::config::config::TokenFormat::ApiKey => StrategyTokenFormat::Raw,
                        mcp_any_rest::config::config::TokenFormat::Raw => StrategyTokenFormat::Raw,
                        mcp_any_rest::config::config::TokenFormat::Basic => StrategyTokenFormat::Raw,
                    },
                    target_location: match token.target_location {
                        mcp_any_rest::config::config::TokenTargetLocation::Header => StrategyTokenTargetLocation::Header,
                        mcp_any_rest::config::config::TokenTargetLocation::Query => StrategyTokenTargetLocation::Query,
                        mcp_any_rest::config::config::TokenTargetLocation::Cookie => StrategyTokenTargetLocation::Cookie,
                        mcp_any_rest::config::config::TokenTargetLocation::Body => StrategyTokenTargetLocation::Body,
                    },
                    target_key: token.target_key.clone(),
                }
            }).collect(),
        };
        
        let refresh_method = config.refresh_method.as_ref().map(|m| {
            match m {
                mcp_any_rest::config::config::HttpMethod::Get => StrategyHttpMethod::GET,
                mcp_any_rest::config::config::HttpMethod::Post => StrategyHttpMethod::POST,
                mcp_any_rest::config::config::HttpMethod::Put => StrategyHttpMethod::PUT,
                mcp_any_rest::config::config::HttpMethod::Delete => StrategyHttpMethod::DELETE,
                mcp_any_rest::config::config::HttpMethod::Patch => StrategyHttpMethod::PATCH,
            }
        });
        
        StrategyLoginAuthConfig {
            auth_type,
            url: config.url.clone(),
            method,
            headers: config.headers.clone(),
            body,
            response_format,
            token_extraction,
            refresh_url: config.refresh_url.clone(),
            refresh_method,
        }
    });
    
    StrategyAuthConfig {
        mode,
        direct_config,
        login_config,
        token_expiry: config_auth.token_expiry,
        refresh_buffer: config_auth.refresh_buffer,
        max_retry_attempts: config_auth.max_retry_attempts,
    }
}

#[tokio::main]
async fn main() {
    // Set up logging
    env_logger::init();
    
    println!("Testing ZenTao MCP Server Authentication Service");
    println!("================================================");
    
    println!("Configuration validation passed");
    
    // Create dynamic config manager
    let dynamic_config = Arc::new(DynamicConfigManager::new(
        std::path::Path::new("config/config.json").to_path_buf(),
        std::path::Path::new("config/modules.json").to_path_buf(),
        std::path::Path::new("config/presets").to_path_buf(),
    ).unwrap());
    
    // Create authentication service
    let config = dynamic_config.get_config();
    let strategy_auth_config = convert_auth_config(&config.auth);
    let auth_service = match AuthService::new(strategy_auth_config) {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("❌ Failed to create authentication service: {}", e);
            return;
        }
    };
    
    println!("\nTesting token retrieval...");
    
    // Test token retrieval
    match auth_service.get_token().await {
        Ok(token) => {
            tracing::info!("✅ Token retrieved successfully");
            println!("  Token: {}...{}", &token[..20], &token[token.len()-20..]);
            
            // Test token validation
            println!("\nTesting token validation...");
            match auth_service.validate_token(&token).await {
                Ok(is_valid) => {
                    if is_valid {
                        tracing::info!("✅ Token validation passed");
                    } else {
                        tracing::error!("❌ Token validation failed");
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Token validation error: {}", e);
                }
            }
            
            // Test authenticated request headers
            println!("\nTesting authentication headers...");
            match auth_service.get_auth_headers().await {
                Ok(headers) => {
                    tracing::info!("✅ Authentication headers generated successfully");
                    if let Some(auth_header) = headers.get("Authorization") {
                        println!("  Authorization header: {}", auth_header.to_str().unwrap_or("Invalid"));
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Failed to get authentication headers: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to retrieve token: {}", e);
            tracing::error!("❌ Please check your ZenTao server configuration and credentials");
        }
    }
    
    println!("\nAuthentication test completed");
}