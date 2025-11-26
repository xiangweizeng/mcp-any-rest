//! Service composer for aggregating multiple MCP services using module registry pattern

use crate::config::web::WebConfigState;
use crate::config::zml_loader::ZmlModuleLoader;
use crate::services::auth_service::UnifiedAuthService;
use crate::services::composer_service::module_registry::ServiceRegistry;
use crate::{
    config::dynamic::DynamicConfigManager,
    services::dynamic_service::zml_module_factory::ZmlModuleFactory,
};
use log::{debug, error, info};

use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer, ServerHandler};

use std::sync::Arc;

/// Service composer that acts as a proxy for multiple MCP services
/// Uses module registry pattern to delegate requests to appropriate services
#[derive(Clone)]
pub struct ServiceComposer {
    _config: Arc<DynamicConfigManager>,
    auth_service: Arc<UnifiedAuthService>,
    service_registry: Arc<ServiceRegistry>,
}

impl ServiceComposer {
    /// Create a new service composer proxy with all services using module registry
    pub fn new(config: Arc<DynamicConfigManager>) -> anyhow::Result<Self> {
        info!("Creating new ServiceComposer with module registry pattern");

        debug!("Creating UnifiedAuthService");
        let config_clone = config.get_config();
        
        // Convert config::AuthConfig to auth_strategy::AuthConfig
        let auth_config = crate::services::auth_service::auth_strategy::AuthConfig {
            mode: match config_clone.auth.mode {
                crate::config::config::AuthMode::Direct => crate::services::auth_service::auth_strategy::AuthMode::Direct,
                crate::config::config::AuthMode::Login => crate::services::auth_service::auth_strategy::AuthMode::Login,
            },
            direct_config: config_clone.auth.direct_config.map(|dc| {
                crate::services::auth_service::auth_strategy::DirectAuthConfig {
                    auth_type: match dc.auth_type {
                        crate::config::config::DirectAuthType::Bearer => crate::services::auth_service::auth_strategy::DirectAuthType::Bearer,
                        crate::config::config::DirectAuthType::ApiKey => crate::services::auth_service::auth_strategy::DirectAuthType::ApiKey,
                        crate::config::config::DirectAuthType::Basic => crate::services::auth_service::auth_strategy::DirectAuthType::Basic,
                        crate::config::config::DirectAuthType::Token => crate::services::auth_service::auth_strategy::DirectAuthType::Token,
                        crate::config::config::DirectAuthType::CustomHeaders => crate::services::auth_service::auth_strategy::DirectAuthType::CustomHeaders,
                    },
                    token: dc.token,
                    api_key_name: dc.api_key_name,
                    username: dc.username,
                    password: dc.password,
                    custom_headers: dc.custom_headers,
                }
            }),
            login_config: config_clone.auth.login_config.map(|lc| {
                crate::services::auth_service::auth_strategy::LoginAuthConfig {
                    auth_type: match lc.auth_type {
                        crate::config::config::LoginAuthType::Json => crate::services::auth_service::auth_strategy::LoginAuthType::Json,
                        crate::config::config::LoginAuthType::Form => crate::services::auth_service::auth_strategy::LoginAuthType::Form,
                        crate::config::config::LoginAuthType::OAuth2 => crate::services::auth_service::auth_strategy::LoginAuthType::OAuth2,
                        crate::config::config::LoginAuthType::ApiKey => crate::services::auth_service::auth_strategy::LoginAuthType::ApiKey,
                        crate::config::config::LoginAuthType::Custom => crate::services::auth_service::auth_strategy::LoginAuthType::Custom,
                    },
                    url: lc.url,
                    method: match lc.method {
                        crate::config::config::HttpMethod::Get => crate::services::auth_service::auth_strategy::HttpMethod::GET,
                        crate::config::config::HttpMethod::Post => crate::services::auth_service::auth_strategy::HttpMethod::POST,
                        crate::config::config::HttpMethod::Put => crate::services::auth_service::auth_strategy::HttpMethod::PUT,
                        crate::config::config::HttpMethod::Delete => crate::services::auth_service::auth_strategy::HttpMethod::DELETE,
                        crate::config::config::HttpMethod::Patch => crate::services::auth_service::auth_strategy::HttpMethod::PATCH,
                    },
                    headers: lc.headers,
                    body: lc.body.map(|b| {
                        crate::services::auth_service::auth_strategy::LoginRequestBody {
                            format: match b.format {
                                crate::config::config::BodyFormat::Json => crate::services::auth_service::auth_strategy::BodyFormat::Json,
                                crate::config::config::BodyFormat::Form => crate::services::auth_service::auth_strategy::BodyFormat::Form,
                            },
                            content: b.content,
                        }
                    }),
                    response_format: match lc.response_format {
                        crate::config::config::ResponseFormat::Json => crate::services::auth_service::auth_strategy::ResponseFormat::Json,
                        crate::config::config::ResponseFormat::Xml => crate::services::auth_service::auth_strategy::ResponseFormat::Xml,
                        crate::config::config::ResponseFormat::Text => crate::services::auth_service::auth_strategy::ResponseFormat::Text,
                    },
                    token_extraction: if !lc.token_extraction.tokens.is_empty() {
                        crate::services::auth_service::auth_strategy::TokenExtraction {
                            tokens: lc.token_extraction.tokens.into_iter().map(|token| {
                                crate::services::auth_service::auth_strategy::TokenExtractionItem {
                                    source_location: match token.source_location {
                                        crate::config::config::TokenLocation::Header => crate::services::auth_service::auth_strategy::TokenLocation::Header,
                                        crate::config::config::TokenLocation::Body => crate::services::auth_service::auth_strategy::TokenLocation::Body,
                                        crate::config::config::TokenLocation::Query => crate::services::auth_service::auth_strategy::TokenLocation::Query,
                                    },
                                    source_key: token.source_key,
                                    format: match token.format {
                                        crate::config::config::TokenFormat::Bearer => crate::services::auth_service::auth_strategy::TokenFormat::Bearer,
                                        crate::config::config::TokenFormat::Token => crate::services::auth_service::auth_strategy::TokenFormat::Raw,
                                        crate::config::config::TokenFormat::ApiKey => crate::services::auth_service::auth_strategy::TokenFormat::Raw,
                                        crate::config::config::TokenFormat::Raw => crate::services::auth_service::auth_strategy::TokenFormat::Raw,
                                        crate::config::config::TokenFormat::Basic => crate::services::auth_service::auth_strategy::TokenFormat::Basic,
                                    },
                                    target_location: match token.target_location {
                                        crate::config::config::TokenTargetLocation::Header => crate::services::auth_service::auth_strategy::TokenTargetLocation::Header,
                                        crate::config::config::TokenTargetLocation::Query => crate::services::auth_service::auth_strategy::TokenTargetLocation::Query,
                                        crate::config::config::TokenTargetLocation::Cookie => crate::services::auth_service::auth_strategy::TokenTargetLocation::Header, // Default to Header for Cookie
                                        crate::config::config::TokenTargetLocation::Body => crate::services::auth_service::auth_strategy::TokenTargetLocation::Body,
                                    },
                                    target_key: token.target_key,
                                }
                            }).collect(),
                        }
                    } else {
                        // Fallback for old format if tokens is empty
                        crate::services::auth_service::auth_strategy::TokenExtraction::default()
                    },
                    refresh_url: lc.refresh_url,
                    refresh_method: lc.refresh_method.map(|m| {
                        match m {
                            crate::config::config::HttpMethod::Get => crate::services::auth_service::auth_strategy::HttpMethod::GET,
                            crate::config::config::HttpMethod::Post => crate::services::auth_service::auth_strategy::HttpMethod::POST,
                            crate::config::config::HttpMethod::Put => crate::services::auth_service::auth_strategy::HttpMethod::PUT,
                            crate::config::config::HttpMethod::Delete => crate::services::auth_service::auth_strategy::HttpMethod::DELETE,
                            crate::config::config::HttpMethod::Patch => crate::services::auth_service::auth_strategy::HttpMethod::PATCH,
                        }
                    }),
                }
            }),
            token_expiry: config_clone.auth.token_expiry,
            refresh_buffer: config_clone.auth.refresh_buffer,
            max_retry_attempts: config_clone.auth.max_retry_attempts,
        };
        
        let auth_service = Arc::new(UnifiedAuthService::new(auth_config)
            .map_err(|e| anyhow::anyhow!("Failed to create auth service: {:?}", e))?);

        debug!("Creating ServiceRegistry");
        let mut service_registry = ServiceRegistry::new(config.clone(), auth_service.clone());

        // Register ZML-based modules
        // Use config directory to determine ZML directory path
        let (config_path, _, _) = config.get_config_paths();
        let config_dir = config_path.parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let zml_dir = config_dir.join("zml");
        
        info!("Loading ZML modules from: {:?}", zml_dir);
        
        let zml_loader = match ZmlModuleLoader::from_dir(&zml_dir) {
            Ok(loader) => Arc::new(loader),
            Err(e) => {
                error!("Failed to load ZML modules from {}: {}", zml_dir.display(), e);
                // Continue without ZML modules if loading fails
                Arc::new(ZmlModuleLoader::default())
            }
        };
        
        let zml_factory = ZmlModuleFactory::new(zml_loader.clone(), config.clone(), auth_service.clone());
        zml_factory.register_modules(&mut service_registry).unwrap();

        let service_registry = Arc::new(service_registry);

        info!(
            "ServiceComposer created successfully with {} modules registered",
            service_registry.get_module_count()
        );

        Ok(Self {
            _config: config,
            auth_service,
            service_registry,
        })
    }

    /// Create a new service composer from WebConfigState
    pub fn from_web_state(state: WebConfigState) -> anyhow::Result<Self> {
        info!("Creating new ServiceComposer from WebConfigState");

        // Extract DynamicConfigManager from WebConfigState
        let config = match state {
            WebConfigState::Dynamic(manager) => manager,
            WebConfigState::Loader(_) => {
                // For ConfigLoader, we need to create a default DynamicConfigManager
                // This is a limitation of the current implementation
                info!("Creating default DynamicConfigManager for ConfigLoader state");

                // Use default paths for the DynamicConfigManager
                let config_path = std::path::PathBuf::from("config/config.toml");
                let module_config_path = std::path::PathBuf::from("config/modules.toml");
                let preset_config_path = std::path::PathBuf::from("config/presets");

                // Create the DynamicConfigManager with default paths
                match DynamicConfigManager::new(config_path, module_config_path, preset_config_path)
                {
                    Ok(manager) => Arc::new(manager),
                    Err(e) => {
                        error!("Failed to create default DynamicConfigManager: {}", e);
                        // Fallback to a simple default if creation fails
                        Arc::new(
                            DynamicConfigManager::new(
                                std::path::PathBuf::from("config.toml"),
                                std::path::PathBuf::from("modules.toml"),
                                std::path::PathBuf::from("presets"),
                            )
                            .unwrap_or_else(|_| {
                                panic!("Failed to create fallback DynamicConfigManager")
                            }),
                        )
                    }
                }
            }
        };

        Ok(Self::new(config)?)
    }

    /// Get auth service reference
    pub fn auth_service(&self) -> &UnifiedAuthService {
        &self.auth_service
    }

    /// Get service registry reference
    pub fn service_registry(&self) -> &ServiceRegistry {
        &self.service_registry
    }
}

impl ServerHandler for ServiceComposer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(format!(
                "MCP-ANY-REST. Use 'list_all_tools' to see available tools."
            )),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, McpError> {
        debug!("ServiceComposer: Delegating tool listing to service registry");

        // Use service registry to aggregate tools from all modules
        let all_tools = self
            .service_registry
            .aggregate_tools(_context.clone())
            .await?;

        debug!(
            "ServiceComposer: Total tools available: {}",
            all_tools.tools.len()
        );
        info!(
            "ServiceComposer successfully aggregated {} tools from all modules",
            all_tools.tools.len()
        );

        Ok(ListToolsResult {
            tools: all_tools.tools,
            next_cursor: None,
        })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListPromptsResult, McpError> {
        debug!("ServiceComposer: Delegating prompt listing to service registry");

        // Use service registry to aggregate prompts from all modules
        let all_prompts = self
            .service_registry
            .aggregate_prompts(_context.clone())
            .await?;

        debug!(
            "ServiceComposer: Total prompts available: {}",
            all_prompts.len()
        );
        info!(
            "ServiceComposer successfully aggregated {} prompts from all modules",
            all_prompts.len()
        );

        Ok(ListPromptsResult {
            prompts: all_prompts,
            next_cursor: None,
        })
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListResourcesResult, McpError> {
        debug!("ServiceComposer: Delegating resource listing to service registry");

        // Use service registry to aggregate resources from all modules
        let all_resources = self
            .service_registry
            .aggregate_resources(_context.clone())
            .await?;

        info!(
            "ServiceComposer successfully aggregated {} resources from all modules",
            all_resources.len()
        );

        Ok(ListResourcesResult {
            resources: all_resources,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResult, McpError> {
        debug!(
            "ServiceComposer: Routing tool call '{}' through service registry",
            request.name
        );

        // Use service registry to route the tool call to the appropriate module
        self.service_registry
            .route_tool_call(request, context)
            .await
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> std::result::Result<GetPromptResult, McpError> {
        debug!(
            "ServiceComposer: Routing prompt request '{}' through service registry",
            request.name
        );

        // Use service registry to route the prompt request to the appropriate module
        self.service_registry
            .route_prompt_request(request, context)
            .await
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> std::result::Result<ReadResourceResult, McpError> {
        debug!(
            "ServiceComposer: Routing resource read request '{}' through service registry",
            request.uri
        );

        // Use service registry to route the resource read request to the appropriate module
        self.service_registry
            .route_resource_request(request, context)
            .await
    }
}