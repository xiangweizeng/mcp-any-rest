//! Unit tests for authentication service factory pattern

use crate::config::config::Config;
use crate::config::dynamic::DynamicConfigManager;
use crate::services::auth_service::unified_auth_service::UnifiedAuthService;
use crate::services::auth_service::auth_factory::{AuthServiceFactoryBuilder};
use crate::services::auth_service::auth_strategy::{
    AuthConfig, AuthError, AuthMode, DirectAuthConfig, LoginAuthConfig, DirectAuthType, LoginAuthType,
    LoginRequestBody, ResponseFormat, TokenExtraction, TokenExtractionItem, TokenFormat, TokenLocation, TokenTargetLocation, BodyFormat, HttpMethod,
};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod auth_factory_tests {
    use super::*;

    #[test]
    fn test_auth_mode_enum() {
        assert_eq!(AuthMode::Direct.to_string(), "direct");
        assert_eq!(AuthMode::Login.to_string(), "login");
        
        let direct = AuthMode::Direct;
        let login = AuthMode::Login;
        
        assert!(matches!(direct, AuthMode::Direct));
        assert!(matches!(login, AuthMode::Login));
    }

    #[test]
    fn test_direct_auth_config_creation() {
        let config = DirectAuthConfig {
            auth_type: DirectAuthType::Token,
            token: Some("test-token".to_string()),
            api_key_name: None,
            username: None,
            password: None,
            custom_headers: None,
        };
        
        assert_eq!(config.token, Some("test-token".to_string()));
        assert_eq!(config.auth_type, DirectAuthType::Token);
    }

    #[test]
    fn test_login_auth_config_creation() {
        let config = LoginAuthConfig {
            auth_type: LoginAuthType::Json,
            url: "http://localhost/login".to_string(),
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
            token_extraction: TokenExtraction {
                tokens: vec![
                    TokenExtractionItem {
                        source_location: TokenLocation::Body,
                        source_key: "token".to_string(),
                        format: TokenFormat::Bearer,
                        target_location: TokenTargetLocation::Header,
                        target_key: "Authorization".to_string(),
                    }
                ],
            },
            refresh_url: None,
            refresh_method: None,
        };
        
        assert_eq!(config.url, "http://localhost/login");
        assert_eq!(config.method, HttpMethod::POST);
        assert_eq!(config.token_extraction.tokens.len(), 1);
        assert_eq!(config.token_extraction.tokens[0].source_location, TokenLocation::Body);
        assert_eq!(config.token_extraction.tokens[0].source_key, "token");
        assert_eq!(config.token_extraction.tokens[0].format, TokenFormat::Bearer);
        assert_eq!(config.token_extraction.tokens[0].target_location, TokenTargetLocation::Header);
        assert_eq!(config.token_extraction.tokens[0].target_key, "Authorization");
    }

    #[tokio::test]
    async fn test_auth_service_factory_builder_direct() {
        let direct_config = DirectAuthConfig {
            auth_type: DirectAuthType::Token,
            token: Some("test-token".to_string()),
            api_key_name: None,
            username: None,
            password: None,
            custom_headers: None,
        };
        
        let auth_config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(direct_config),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let auth_service = UnifiedAuthService::new(auth_config).unwrap();
            
        // Test that the service was created successfully
        assert_eq!(auth_service.get_auth_mode().await, AuthMode::Direct);
        
        // Test that we can get a token through the service
        let token = auth_service.get_token().await;
        assert!(token.is_ok());
        assert_eq!(token.unwrap(), "test-token");
    }

    #[tokio::test]
    async fn test_unified_auth_service_with_config_manager() {
        let _config = Config::new();
        
        // Create a direct auth config for testing
        let auth_config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(DirectAuthConfig::default()),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let auth_service = UnifiedAuthService::new(auth_config).unwrap();
        
        // Should have Direct as default strategy initially
        assert_eq!(auth_service.get_auth_mode().await, AuthMode::Direct);
    }

    #[tokio::test]
    async fn test_auth_service_factory_impl_direct() {
        let _config = Config::new();
        
        // Create a direct auth config for testing
        let auth_config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(DirectAuthConfig::default()),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let auth_service = UnifiedAuthService::new(auth_config).unwrap();
        
        // Test that the factory was created with the correct default strategy
        assert_eq!(auth_service.get_auth_mode().await, AuthMode::Direct);
    }

    #[tokio::test]
    async fn test_auth_service_factory_impl_login() {
        let _config = Config::new();
        
        // Create a login auth config for testing
        let login_auth_config = LoginAuthConfig {
            auth_type: LoginAuthType::Json,
            url: "http://localhost/login".to_string(),
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
            token_extraction: TokenExtraction {
                tokens: vec![
                    TokenExtractionItem {
                        source_location: TokenLocation::Body,
                        source_key: "token".to_string(),
                        format: TokenFormat::Bearer,
                        target_location: TokenTargetLocation::Header,
                        target_key: "Authorization".to_string(),
                    }
                ],
            },
            refresh_url: None,
            refresh_method: None,
        };
        
        let auth_config = AuthConfig {
            mode: AuthMode::Login,
            direct_config: None,
            login_config: Some(login_auth_config),
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let auth_service = UnifiedAuthService::new(auth_config).unwrap();
        
        // Test that the factory was created with the correct strategy
        assert_eq!(auth_service.get_auth_mode().await, AuthMode::Login);
    }

    #[tokio::test]
    async fn test_auth_service_factory_impl_default() {
        let _config = Config::new();
        
        // Create a simple auth config for testing
        let auth_config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(DirectAuthConfig::default()),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let auth_service = UnifiedAuthService::new(auth_config).unwrap();

        // Should have Direct as default strategy initially
        assert_eq!(auth_service.get_auth_mode().await, AuthMode::Direct);
    }

    #[tokio::test]
    async fn test_direct_auth_config_strategy() {
        let config = DirectAuthConfig {
            auth_type: DirectAuthType::Token,
            token: Some("test-token".to_string()),
            api_key_name: None,
            username: None,
            password: None,
            custom_headers: None,
        };
        
        let auth_config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(config),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let factory = AuthServiceFactoryBuilder::new()
            .with_config(auth_config)
            .build()
            .unwrap();
            
        // Test token retrieval
        let token = factory.get_token().await;
        assert!(token.is_ok());
        assert_eq!(token.unwrap(), "test-token");
        
        // Test token validation
        let validation_result = factory.validate_token("test-token").await;
        assert!(validation_result.is_ok());
    }

    #[tokio::test]
    async fn test_direct_auth_config_strategy_with_custom_headers() {
        let mut custom_headers = HashMap::new();
        custom_headers.insert("Authorization".to_string(), "Bearer test-token".to_string());
        
        let config = DirectAuthConfig {
            auth_type: DirectAuthType::CustomHeaders,
            token: None,
            api_key_name: None,
            username: None,
            password: None,
            custom_headers: Some(custom_headers),
        };
        
        let auth_config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(config),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let factory = AuthServiceFactoryBuilder::new()
            .with_config(auth_config)
            .build()
            .unwrap();
            
        // Test token retrieval
        let token = factory.get_token().await;
        assert!(token.is_ok());
        
        // Test token validation
        let validation_result = factory.validate_token("Bearer test-token").await;
        assert!(validation_result.is_ok());
    }

    #[tokio::test]
    async fn test_login_auth_config_strategy_creation() {
        let login_auth_config = LoginAuthConfig {
            auth_type: LoginAuthType::Json,
            url: "http://localhost/login".to_string(),
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
            token_extraction: TokenExtraction {
                tokens: vec![
                    TokenExtractionItem {
                        source_location: TokenLocation::Body,
                        source_key: "token".to_string(),
                        format: TokenFormat::Bearer,
                        target_location: TokenTargetLocation::Header,
                        target_key: "Authorization".to_string(),
                    }
                ],
            },
            refresh_url: None,
            refresh_method: None,
        };
        
        let auth_config = AuthConfig {
            mode: AuthMode::Login,
            direct_config: None,
            login_config: Some(login_auth_config),
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let factory = AuthServiceFactoryBuilder::new()
            .with_config(auth_config)
            .build()
            .unwrap();
            
        // Test that the factory was created successfully
        assert_eq!(factory.get_auth_mode(), AuthMode::Login);
        
        // Test token retrieval (this will fail due to network, but should not panic)
        let token_result = factory.get_token().await;
        assert!(token_result.is_err()); // Should fail due to network connection
    }

    #[tokio::test]
    async fn test_auth_error_enum() {
        let config_error = AuthError::ConfigurationError("Invalid configuration".to_string());
        let token_error = AuthError::TokenExpired("Token expired".to_string());
        let network_error = AuthError::NetworkError("Request failed".to_string());
        let parse_error = AuthError::ParseError("Parse failed".to_string());

        assert!(matches!(config_error, AuthError::ConfigurationError(_)));
        assert!(matches!(token_error, AuthError::TokenExpired(_)));
        assert!(matches!(network_error, AuthError::NetworkError(_)));
        assert!(matches!(parse_error, AuthError::ParseError(_)));
    }

    #[tokio::test]
    async fn test_auth_strategy_trait_object() {
        let config = DirectAuthConfig {
            auth_type: DirectAuthType::Token,
            token: Some("test-token".to_string()),
            api_key_name: None,
            username: None,
            password: None,
            custom_headers: None,
        };
        
        let auth_config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(config),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let factory = AuthServiceFactoryBuilder::new()
            .with_config(auth_config)
            .build()
            .unwrap();
            
        // Test that we can use the factory through the trait object
        let token = factory.get_token().await;
        assert!(token.is_ok());
        assert_eq!(token.unwrap(), "test-token");
        
        let validation_result = factory.validate_token("test-token").await;
        assert!(validation_result.is_ok());
    }

    #[tokio::test]
    async fn test_auth_service_factory_error_handling() {
        let _config = Config::new();
        
        // Test with invalid DirectAuthConfig (no token provided)
        let invalid_auth_config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(DirectAuthConfig {
                auth_type: DirectAuthType::Token,
                token: None,
                api_key_name: None,
                username: None,
                password: None,
                custom_headers: None,
            }),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let auth_service = UnifiedAuthService::new(invalid_auth_config).unwrap();

        // Test token retrieval with invalid config should fail
        let token_result = auth_service.get_token().await;
        assert!(token_result.is_err());
        
        // Test token validation with invalid token
        let validation_result = auth_service.validate_token("invalid-token").await;
        assert!(validation_result.is_err());
        
        // Test with valid config
        let valid_auth_config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(DirectAuthConfig::default()),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let valid_auth_service = UnifiedAuthService::new(valid_auth_config).unwrap();
        assert_eq!(valid_auth_service.get_auth_mode().await, AuthMode::Direct);
    }

    #[tokio::test]
    async fn test_auth_service_factory_performance() {
        // Test factory creation performance
        let config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(DirectAuthConfig::default()),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let start = std::time::Instant::now();
        let factory = AuthServiceFactoryBuilder::new()
            .with_config(config)
            .build()
            .unwrap();
        let duration = start.elapsed();
        
        assert!(factory.get_auth_mode() == AuthMode::Direct);
        assert!(duration.as_millis() < 1000, "Factory creation should be fast");
    }

    #[tokio::test]
    async fn test_auth_service_factory_thread_safety() {
        // Test that factory can be safely accessed from multiple threads
        let config = AuthConfig {
            mode: AuthMode::Direct,
            direct_config: Some(DirectAuthConfig::default()),
            login_config: None,
            token_expiry: 3600,
            refresh_buffer: 300,
            max_retry_attempts: 3,
        };
        
        let factory = Arc::new(AuthServiceFactoryBuilder::new()
            .with_config(config)
            .build()
            .unwrap());
        
        let mut handles = vec![];
        
        // Spawn multiple tasks to access the factory concurrently
        for i in 0..10 {
            let factory_clone = Arc::clone(&factory);
            
            let handle = tokio::spawn(async move {
                // Access factory methods from different tasks
                let strategy_type = factory_clone.get_auth_mode();
                assert_eq!(strategy_type, AuthMode::Direct);
                
                // Simulate some work
                tokio::time::sleep(std::time::Duration::from_millis(i * 10)).await;
                
                // Access factory again
                let strategy_type = factory_clone.get_auth_mode();
                assert_eq!(strategy_type, AuthMode::Direct);
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }
}