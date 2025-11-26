// ！ WebServer

use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::{Html, Json},
    routing::{delete, get, patch, post, put},
    Router,
};

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

use rmcp::{
    transport::{
        streamable_http_server::session::local::LocalSessionManager, StreamableHttpService,
    },
};

use crate::config::config::Config;
use crate::config::dynamic::ConfigChangeEvent;
use crate::config::dynamic::DynamicConfigManager;
use crate::config::loader::ConfigLoader;
use crate::config::module::GlobalModuleConfig;
use crate::config::module::ModuleConfig;

/// Web configuration server state (compatible with both old and new config systems)
#[derive(Clone)]
pub enum WebConfigState {
    Dynamic(Arc<DynamicConfigManager>),
    Loader(Arc<ConfigLoader>),
}

impl WebConfigState {
    /// Get the configuration
    pub fn get_config(&self) -> Config {
        match self {
            WebConfigState::Dynamic(manager) => manager.get_config(),
            WebConfigState::Loader(_loader) => {
                // For ConfigLoader, we need to create a default Config
                // since ConfigLoader doesn't have a get_config method
                Config::new()
            }
        }
    }

    /// Get available presets as string values
    pub fn get_available_presets(&self) -> Result<Vec<String>> {
        match self {
            WebConfigState::Dynamic(manager) => {
                let presets = manager.get_available_presets()?;
                let mut result = Vec::new();
                for preset in presets {
                    result.push(preset.id.clone());
                }
                Ok(result)
            }
            WebConfigState::Loader(loader) => {
                let presets = loader.get_available_presets()?;
                let mut result = Vec::new();
                for preset in presets {
                    result.push(preset.id.clone());
                }
                Ok(result)
            }
        }
    }

    /// Get actual preset information (for API responses)
    pub fn get_preset_info(&self) -> Result<Vec<serde_json::Value>> {
        match self {
            WebConfigState::Dynamic(manager) => {
                let presets = manager.get_available_presets()?;
                let mut preset_values = Vec::new();

                for preset in presets {
                    // Load the full preset configuration to get module information
                    let preset_config =
                        manager.load_preset_config(&preset.id).unwrap_or_else(|_| {
                            // If loading fails, create a default preset config
                            crate::config::dynamic::PresetConfig {
                                name: preset.name.clone(),
                                description: preset.description.clone(),
                                modules: std::collections::HashMap::new(),
                                default_access_level: None,
                                default_rate_limit: None,
                            }
                        });

                    let preset_value = serde_json::json!({
                        "id": preset.id,
                        "name": preset.name,
                        "description": preset.description,
                        "enabled": preset.enabled,
                        "priority": preset.priority,
                        "modules": preset_config.modules
                    });
                    preset_values.push(preset_value);
                }

                Ok(preset_values)
            }
            WebConfigState::Loader(loader) => {
                let presets = loader.get_available_presets()?;
                let mut preset_values = Vec::new();

                for preset in presets {
                    // For ConfigLoader, we need to create a new loader with preset path to load the preset configuration
                    let preset_config_path = loader
                        .get_config_path()
                        .parent()
                        .map(|p| p.join("presets"))
                        .unwrap_or_else(|| PathBuf::from("config/presets"));

                    let preset_loader =
                        crate::config::preset_loader::PresetLoader::new(preset_config_path);

                    // Load the preset configuration
                    let preset_config =
                        if let Ok(()) = preset_loader.clone().load_preset(&preset.id) {
                            preset_loader
                                .get_preset(&preset.id)
                                .map(|config| config.clone())
                                .unwrap_or_else(|| crate::config::preset_loader::PresetConfig {
                                    name: preset.name.clone(),
                                    description: preset.description.clone(),
                                    default_access_level: None,
                                    default_rate_limit: None,
                                    modules: std::collections::HashMap::new(),
                                })
                        } else {
                            // If loading fails, create a default preset config
                            crate::config::preset_loader::PresetConfig {
                                name: preset.name.clone(),
                                description: preset.description.clone(),
                                default_access_level: None,
                                default_rate_limit: None,
                                modules: std::collections::HashMap::new(),
                            }
                        };

                    let preset_value = serde_json::json!({
                        "id": preset.id,
                        "name": preset.name,
                        "description": preset.description,
                        "enabled": preset.enabled,
                        "priority": preset.priority,
                        "modules": preset_config.modules
                    });
                    preset_values.push(preset_value);
                }

                Ok(preset_values)
            }
        }
    }

    /// Apply a preset
    pub fn apply_preset(&self, preset: String) -> Result<()> {
        match self {
            WebConfigState::Dynamic(manager) => manager.apply_preset(preset),
            WebConfigState::Loader(loader) => {
                // Load the preset and apply it
                let loader_clone = loader.clone();
                loader_clone
                    .load_config_with_preset(Some(&preset))
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!("Failed to apply preset: {}", e))
            }
        }
    }

    /// Update configuration
    pub fn update_config(&self, config: Config) -> Result<()> {
        match self {
            WebConfigState::Dynamic(manager) => manager.update_config(config),
            WebConfigState::Loader(_loader) => {
                // ConfigLoader doesn't support updating main config directly
                // This is a limitation of the new system
                Ok(())
            }
        }
    }

    /// Update module configuration
    pub fn update_module_config(&self, module_config: GlobalModuleConfig) -> Result<()> {
        match self {
            WebConfigState::Dynamic(manager) => manager.update_module_config(module_config),
            WebConfigState::Loader(loader) => {
                // For ConfigLoader, we need to save the module configuration
                let loader_clone = loader.clone();
                loader_clone
                    .save_config(&module_config)
                    .map_err(|e| anyhow::anyhow!("Failed to update module configuration: {}", e))
            }
        }
    }

    /// Get configuration paths
    pub fn get_config_paths(&self) -> (PathBuf, PathBuf, PathBuf) {
        match self {
            WebConfigState::Dynamic(manager) => manager.get_config_paths(),
            WebConfigState::Loader(loader) => {
                // For ConfigLoader, return the config path and preset path
                (
                    loader.get_config_path().to_path_buf(),
                    PathBuf::from("config/modules.json"),
                    PathBuf::from("config/presets"),
                )
            }
        }
    }

    /// Subscribe to configuration changes
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        match self {
            WebConfigState::Dynamic(manager) => manager.subscribe(),
            WebConfigState::Loader(_loader) => {
                // Create a dummy receiver for ConfigLoader
                let (_, receiver) = broadcast::channel(1);
                receiver
            }
        }
    }

    /// Reload configuration if modified
    pub fn reload_if_modified(&self) -> Result<bool> {
        match self {
            WebConfigState::Dynamic(manager) => manager.reload_if_modified(),
            WebConfigState::Loader(_loader) => {
                // ConfigLoader doesn't support automatic reloading
                Ok(false)
            }
        }
    }

    /// Save a preset configuration
    pub fn save_preset(
        &self,
        preset_id: String,
        preset_config: crate::config::preset_loader::PresetConfig,
    ) -> Result<()> {
        info!("Saving preset: {}", preset_id);
        match self {
            WebConfigState::Dynamic(manager) => {
                // For DynamicConfigManager, we need to implement save_preset functionality
                // Since DynamicConfigManager doesn't have save_preset, we'll create a new PresetLoader
                let preset_path = manager.get_config_paths().2;
                let mut preset_loader =
                    crate::config::preset_loader::PresetLoader::new(preset_path);
                preset_loader.save_preset(&preset_id, &preset_config)
            }
            WebConfigState::Loader(loader) => {
                // For ConfigLoader, use the preset loader
                let preset_path = loader
                    .get_config_path()
                    .parent()
                    .map(|p| p.join("presets"))
                    .unwrap_or_else(|| PathBuf::from("config/presets"));

                let mut preset_loader =
                    crate::config::preset_loader::PresetLoader::new(preset_path);
                preset_loader.save_preset(&preset_id, &preset_config)
            }
        }
    }

    /// Delete a preset
    pub fn delete_preset(&self, preset_id: String) -> Result<()> {
        match self {
            WebConfigState::Dynamic(manager) => {
                // For DynamicConfigManager, we need to implement delete_preset functionality
                let preset_path = manager.get_config_paths().2;
                let mut preset_loader =
                    crate::config::preset_loader::PresetLoader::new(preset_path);
                preset_loader.delete_preset(&preset_id)
            }
            WebConfigState::Loader(loader) => {
                // For ConfigLoader, use the preset loader
                let preset_path = loader
                    .get_config_path()
                    .parent()
                    .map(|p| p.join("presets"))
                    .unwrap_or_else(|| PathBuf::from("config/presets"));

                let mut preset_loader =
                    crate::config::preset_loader::PresetLoader::new(preset_path);
                preset_loader.delete_preset(&preset_id)
            }
        }
    }
}

/// Configuration update request
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigUpdateRequest {
    pub preset: Option<String>,
    pub config: Option<Config>,
    pub module_config: Option<GlobalModuleConfig>,
    pub changes: Option<Vec<String>>,
}

/// Configuration response
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: String,
    pub config: Option<serde_json::Value>,
    pub module_config: Option<serde_json::Value>,
}

/// Preset list response
#[derive(Debug, Deserialize, Serialize)]
pub struct PresetListResponse {
    pub success: bool,
    pub message: String,
    pub presets: Vec<serde_json::Value>,
}

/// History response
#[derive(Debug, Deserialize, Serialize)]
pub struct HistoryResponse {
    pub success: bool,
    pub message: String,
    pub history: Vec<serde_json::Value>,
}

use crate::services::composer_service::ServiceComposer;

// Use port from configuration
fn get_bind_address(config: &Config) -> String {
    format!("127.0.0.1:{}", config.server.port)
}

/// Web configuration server
pub struct WebServer {
    _state: WebConfigState,
    _router: Router,
    _change_receiver: Option<broadcast::Receiver<ConfigChangeEvent>>,
    _service_composer: Option<crate::services::composer_service::ServiceComposer>,
}

impl WebServer {
    /// Create a new web configuration server with DynamicConfigManager
    pub fn new_dynamic(config_manager: Arc<DynamicConfigManager>) -> Self {
        let state = WebConfigState::Dynamic(config_manager.clone());
        let change_receiver = Some(config_manager.subscribe());

        let router = Router::new()
            .route("/", get(Self::index))
            .route("/config", get(Self::get_config))
            .route("/config", post(Self::update_config))
            .route("/config/presets", get(Self::get_presets))
            .route("/config/presets", post(Self::save_preset))
            .route("/config/presets/:preset_id", delete(Self::delete_preset))
            .route("/config/preset/:preset", post(Self::apply_preset))
            .route("/config/reload", post(Self::reload_config))
            .route("/config/save", post(Self::save_config))
            .route("/config/status", get(Self::get_status))
            .route("/config/modules", get(Self::get_modules))
            .route(
                "/config/modules/:module_name",
                get(Self::get_module)
                    .put(Self::update_module)
                    .patch(Self::update_module_field),
            )
            .route(
                "/config/modules/:module_name/reset",
                post(Self::reset_module),
            )
            .route(
                "/config/modules/:module_name/methods/:method_name",
                patch(Self::update_method),
            )
            .route("/config/server", get(Self::get_server_config))
            .route("/config/server", put(Self::update_server_config))
            .with_state(state.clone());
        Self {
            _state: state,
            _router: router,
            _change_receiver: change_receiver,
            _service_composer: None,
        }
    }

    /// Get the router
    pub fn register_service_composer(mut self, service_composer: ServiceComposer) -> Self {
        // Store composer for runtime updates
        self._service_composer = Some(service_composer.clone());
        let service: StreamableHttpService<ServiceComposer, LocalSessionManager> =
            StreamableHttpService::new(
                move || Ok(service_composer.clone()),
                Default::default(),
                Default::default(),
            );

        let config = self._state.get_config();

        // Start MCP server using HTTP transport
        let address = get_bind_address(&config);
        info!("  - Web configuration: http://{}", address);
        self._router = self._router.nest_service("/mcp", service);
        self
    }

    /// Start the web configuration server
    pub async fn start(self) -> Result<()> {
        info!("Configuration loaded successfully");

        let config = self._state.get_config();
        info!("Base URL: {}", config.api.base_url);
        info!("Server Port: {}", config.server.port);
        info!("Log Level: {}", config.server.log_level);

        // Start MCP server using HTTP transport
        let address = get_bind_address(&config);
        info!("Starting MCP-ANY-REST server (HTTP) on {}...", address);

        let tcp_listener = tokio::net::TcpListener::bind(&address).await?;
        info!("Available endpoints:");
        info!("  - MCP-ANY-REST web configuration: http://{}", address);

        // Listen for dynamic configuration changes and apply to runtime services
        if let (Some(mut receiver), Some(composer)) = (self._change_receiver, self._service_composer.clone()) {
            let state = self._state.clone();
            tokio::spawn(async move {
                loop {
                    match receiver.recv().await {
                        Ok(_evt) => {
                            // Rebuild auth configuration from latest state
                            let cfg = state.get_config();
                            let auth_cfg = crate::services::auth_service::auth_strategy::AuthConfig {
                                mode: match cfg.auth.mode {
                                    crate::config::config::AuthMode::Direct => crate::services::auth_service::auth_strategy::AuthMode::Direct,
                                    crate::config::config::AuthMode::Login => crate::services::auth_service::auth_strategy::AuthMode::Login,
                                },
                                direct_config: cfg.auth.direct_config.map(|dc| {
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
                                login_config: cfg.auth.login_config.map(|lc| {
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
                                                            crate::config::config::TokenTargetLocation::Cookie => crate::services::auth_service::auth_strategy::TokenTargetLocation::Header,
                                                            crate::config::config::TokenTargetLocation::Body => crate::services::auth_service::auth_strategy::TokenTargetLocation::Body,
                                                        },
                                                        target_key: token.target_key,
                                                    }
                                                }).collect(),
                                            }
                                        } else {
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
                                token_expiry: cfg.auth.token_expiry,
                                refresh_buffer: cfg.auth.refresh_buffer,
                                max_retry_attempts: cfg.auth.max_retry_attempts,
                            };

                            match composer.auth_service().update_config(auth_cfg).await {
                                Ok(()) => info!("Applied dynamic auth configuration update"),
                                Err(e) => error!("Failed to update auth configuration dynamically: {:?}", e),
                            }
                        }
                        Err(e) => {
                            error!("Configuration change receiver error: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        // Wait for either server to stop
        tokio::select! {
            result = axum::serve(tcp_listener, self._router.into_make_service())
                .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() }) => {
                if let Err(e) = result {
                    error!("MCP-ANY-REST server error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Index page handler
    async fn index(State(_state): State<WebConfigState>) -> Html<String> {
        // include to bin "config/config.html"
        let html_content = include_str!("config.html");
  
        Html(html_content.to_string())
    }

    /// Get current configuration
    async fn get_config(State(state): State<WebConfigState>) -> Json<ConfigResponse> {
        let config = state.get_config();
        let module_config = config.module_config.clone();

        Json(ConfigResponse {
            success: true,
            message: "Configuration retrieved successfully".to_string(),
            config: Some(serde_json::to_value(config).unwrap_or_default()),
            module_config: Some(serde_json::to_value(module_config).unwrap_or_default()),
        })
    }

    /// Update configuration
    async fn update_config(
        State(state): State<WebConfigState>,
        Json(update_request): Json<ConfigUpdateRequest>,
    ) -> Json<ConfigResponse> {
        match update_request {
            ConfigUpdateRequest {
                config: Some(new_config),
                module_config: Some(new_module_config),
                ..
            } => {
                // Update both configurations
                match state.update_config(new_config) {
                    Ok(()) => match state.update_module_config(new_module_config) {
                        Ok(()) => Json(ConfigResponse {
                            success: true,
                            message: "Configuration updated successfully".to_string(),
                            config: None,
                            module_config: None,
                        }),
                        Err(e) => Json(ConfigResponse {
                            success: false,
                            message: format!("Failed to update module configuration: {}", e),
                            config: None,
                            module_config: None,
                        }),
                    },
                    Err(e) => Json(ConfigResponse {
                        success: false,
                        message: format!("Failed to update configuration: {}", e),
                        config: None,
                        module_config: None,
                    }),
                }
            }
            ConfigUpdateRequest {
                config: Some(new_config),
                module_config: None,
                ..
            } => {
                // Update only main configuration
                match state.update_config(new_config) {
                    Ok(()) => Json(ConfigResponse {
                        success: true,
                        message: "Configuration updated successfully".to_string(),
                        config: None,
                        module_config: None,
                    }),
                    Err(e) => Json(ConfigResponse {
                        success: false,
                        message: format!("Failed to update configuration: {}", e),
                        config: None,
                        module_config: None,
                    }),
                }
            }
            ConfigUpdateRequest {
                config: None,
                module_config: Some(new_module_config),
                ..
            } => {
                // Update only module configuration
                match state.update_module_config(new_module_config) {
                    Ok(()) => Json(ConfigResponse {
                        success: true,
                        message: "Module configuration updated successfully".to_string(),
                        config: None,
                        module_config: None,
                    }),
                    Err(e) => Json(ConfigResponse {
                        success: false,
                        message: format!("Failed to update module configuration: {}", e),
                        config: None,
                        module_config: None,
                    }),
                }
            }
            _ => Json(ConfigResponse {
                success: false,
                message: "No valid configuration data provided".to_string(),
                config: None,
                module_config: None,
            }),
        }
    }

    /// Get available preset configurations
    async fn get_presets(State(state): State<WebConfigState>) -> Json<PresetListResponse> {
        match state.get_preset_info() {
            Ok(presets) => Json(PresetListResponse {
                success: true,
                message: "Presets loaded successfully".to_string(),
                presets: presets,
            }),
            Err(e) => Json(PresetListResponse {
                success: false,
                message: format!("Failed to load presets: {}", e),
                presets: Vec::new(),
            }),
        }
    }

    /// Apply configuration preset
    async fn apply_preset(
        State(state): State<WebConfigState>,
        Path(preset_name): Path<String>,
    ) -> Json<ConfigResponse> {
        // Check if preset exists in available presets first
        let available_presets = match state.get_available_presets() {
            Ok(presets) => presets,
            Err(_) => Vec::new(),
        };

        let preset_exists = available_presets
            .iter()
            .any(|p| p.to_string() == preset_name);

        if !preset_exists
            && !["full", "product", "project", "execution", "test", "custom"]
                .contains(&preset_name.as_str())
        {
            return Json(ConfigResponse {
                success: false,
                message: format!("Invalid preset: {}", preset_name),
                config: None,
                module_config: None,
            });
        }

        match state.apply_preset(preset_name.clone()) {
            Ok(()) => {
                // After applying preset, automatically reload configuration to ensure changes take effect
                match state.reload_if_modified() {
                    Ok(true) => Json(ConfigResponse {
                        success: true,
                        message: format!(
                            "Configuration preset '{}' applied successfully and configuration reloaded",
                            preset_name
                        ),
                        config: None,
                        module_config: None,
                    }),
                    Ok(false) => Json(ConfigResponse {
                        success: true,
                        message: format!(
                            "Configuration preset '{}' applied successfully (no reload needed)",
                            preset_name
                        ),
                        config: None,
                        module_config: None,
                    }),
                    Err(e) => Json(ConfigResponse {
                        success: false,
                        message: format!("Preset applied but failed to reload configuration: {}", e),
                        config: None,
                        module_config: None,
                    }),
                }
            }
            Err(e) => Json(ConfigResponse {
                success: false,
                message: format!("Failed to apply preset: {}", e),
                config: None,
                module_config: None,
            }),
        }
    }

    /// Reload configuration from file
    async fn reload_config(State(state): State<WebConfigState>) -> Json<ConfigResponse> {
        match state.reload_if_modified() {
            Ok(true) => Json(ConfigResponse {
                success: true,
                message: "Configuration reloaded from file".to_string(),
                config: None,
                module_config: None,
            }),
            Ok(false) => Json(ConfigResponse {
                success: true,
                message: "Configuration not modified, no reload needed".to_string(),
                config: None,
                module_config: None,
            }),
            Err(e) => Json(ConfigResponse {
                success: false,
                message: format!("Failed to reload configuration: {}", e),
                config: None,
                module_config: None,
            }),
        }
    }

    /// Get server status
    async fn get_status(State(state): State<WebConfigState>) -> Json<HashMap<String, String>> {
        let mut status = HashMap::new();
        status.insert("status".to_string(), "running".to_string());

        let (config_path, module_config_path, _preset_config_path) = state.get_config_paths();
        status.insert("config_path".to_string(), config_path.display().to_string());
        status.insert(
            "module_config_path".to_string(),
            module_config_path.display().to_string(),
        );

        Json(status)
    }

    /// Get all modules configuration
    async fn get_modules(State(state): State<WebConfigState>) -> Json<Vec<serde_json::Value>> {
        let module_config = state.get_config().module_config.clone();
        let mut modules = Vec::new();

        for (module_name, module_config) in &module_config.modules {
            let mut methods = Vec::new();

            if let Some(method_configs) = &module_config.methods {
                for (method_name, method_config) in method_configs {
                    methods.push(serde_json::json!({
                        "name": method_name,
                        "enabled": method_config.enabled
                    }));
                }
            }

            modules.push(serde_json::json!({
                "name": module_name,
                "enabled": module_config.enabled,
                "accessLevel": "Public", // This should be derived from actual config
                "rateLimit": 60, // This should be derived from actual config
                "methods": methods
            }));
        }

        Json(modules)
    }

    /// Get specific module configuration
    async fn get_module(
        Path(module_name): Path<String>,
        State(_state): State<WebConfigState>,
    ) -> Json<serde_json::Value> {
        // Mock implementation - should return specific module data
        Json(serde_json::json!({
            "name": module_name,
            "enabled": true,
            "accessLevel": "Public",
            "rateLimit": 60
        }))
    }

    /// Update module configuration
    async fn update_module(
        Path(module_name): Path<String>,
        State(state): State<WebConfigState>,
        Json(update): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        // Get current module configuration
        let mut module_config = state.get_config().module_config.clone();

        // Find the module to update
        if let Some(module) = module_config.modules.get_mut(&module_name) {
            // Process each field in the update object
            for (field, value) in update.as_object().unwrap_or(&serde_json::Map::new()) {
                match field.as_str() {
                    "enabled" => {
                        if let Some(enabled) = value.as_bool() {
                            module.enabled = enabled;
                        }
                    }
                    "accessLevel" => {
                        if let Some(level) = value.as_str() {
                            // Map string to AccessLevel enum
                            match level.to_lowercase().as_str() {
                                "public" => {
                                    // For module-level access level, we need to update all methods
                                    if let Some(methods) = &mut module.methods {
                                        for method in methods.values_mut() {
                                            method.access_level =
                                                Some(crate::config::module::AccessLevel::Public);
                                        }
                                    }
                                }
                                "internal" => {
                                    if let Some(methods) = &mut module.methods {
                                        for method in methods.values_mut() {
                                            method.access_level =
                                                Some(crate::config::module::AccessLevel::Internal);
                                        }
                                    }
                                }
                                "private" => {
                                    if let Some(methods) = &mut module.methods {
                                        for method in methods.values_mut() {
                                            method.access_level =
                                                Some(crate::config::module::AccessLevel::Private);
                                        }
                                    }
                                }
                                _ => {
                                    return Json(serde_json::json!({
                                        "success": false,
                                        "message": format!("Invalid access level: {}", level)
                                    }));
                                }
                            }
                        }
                    }
                    "rateLimit" => {
                        if let Some(limit_obj) = value.as_object() {
                            // Parse rate limit configuration
                            let requests_per_minute = limit_obj
                                .get("requests_per_minute")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(60)
                                as u32;
                            let requests_per_hour = limit_obj
                                .get("requests_per_hour")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(1000)
                                as u32;
                            let burst_capacity = limit_obj
                                .get("burst_capacity")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(10)
                                as u32;

                            // Apply to all methods in the module
                            if let Some(methods) = &mut module.methods {
                                for method in methods.values_mut() {
                                    method.rate_limit =
                                        Some(crate::config::module::RateLimitConfig {
                                            requests_per_minute,
                                            requests_per_hour,
                                            burst_capacity,
                                        });
                                }
                            }
                        }
                    }
                    _ => {
                        info!(
                            "Unknown field {} in update for module {}",
                            field, module_name
                        );
                    }
                }
            }

            // Save the updated configuration
            match state.update_module_config(module_config) {
                Ok(()) => Json(serde_json::json!({
                    "success": true,
                    "message": format!("Module {} updated successfully", module_name),
                    "data": update
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "message": format!("Failed to update module configuration: {}", e)
                })),
            }
        } else {
            Json(serde_json::json!({
                "success": false,
                "message": format!("Module {} not found", module_name)
            }))
        }
    }

    /// Update specific module field
    async fn update_module_field(
        Path(module_name): Path<String>,
        State(state): State<WebConfigState>,
        Json(update): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        // Extract field name and value from the update object
        // Support both formats: { "field": "field_name", "value": field_value } and { "field_name": field_value }
        let (field, value) = if let Some(field_name) = update.get("field").and_then(|f| f.as_str())
        {
            // Format 1: { "field": "field_name", "value": field_value }
            (field_name.to_string(), update.get("value").cloned())
        } else {
            // Format 2: { "field_name": field_value }
            // Find the first key that is not a reserved field
            let mut field_name = String::new();
            let mut field_value = None;

            for (key, value) in update.as_object().unwrap_or(&serde_json::Map::new()) {
                if key != "field" && key != "value" && key != "success" && key != "message" {
                    field_name = key.clone();
                    field_value = Some(value.clone());
                    break;
                }
            }

            if field_name.is_empty() || field_value.is_none() {
                return Json(serde_json::json!({
                    "success": false,
                    "message": "Missing field or value in update request"
                }));
            }

            (field_name, field_value)
        };

        if field.is_empty() || value.is_none() {
            return Json(serde_json::json!({
                "success": false,
                "message": "Missing field or value in update request"
            }));
        }

        // Get current module configuration
        let mut module_config = state.get_config().module_config.clone();

        // Find the module to update
        if let Some(module) = module_config.modules.get_mut(&module_name) {
            match field.as_str() {
                "enabled" => {
                    if let Some(enabled) = value.and_then(|v| v.as_bool()) {
                        module.enabled = enabled;
                    } else {
                        return Json(serde_json::json!({
                            "success": false,
                            "message": "Invalid value for enabled field"
                        }));
                    }
                }
                "accessLevel" => {
                    if let Some(level_value) = value {
                        if let Some(level) = level_value.as_str() {
                            // This would need to be mapped to the actual AccessLevel enum
                            // For now, we'll just log it
                            info!("Access level update for {}: {}", module_name, level);
                        }
                    }
                }
                "rateLimit" => {
                    if let Some(limit_value) = value {
                        if let Some(limit) = limit_value.as_u64() {
                            // This would need to update the actual rate limit configuration
                            // For now, we'll just log it
                            info!("Rate limit update for {}: {}", module_name, limit);
                        }
                    }
                }
                _ => {
                    return Json(serde_json::json!({
                        "success": false,
                        "message": format!("Unknown field: {}", field)
                    }));
                }
            }

            // Save the updated configuration
            match state.update_module_config(module_config) {
                Ok(()) => Json(serde_json::json!({
                    "success": true,
                    "message": format!("Module {} field {} updated successfully", module_name, field),
                    "data": update
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "message": format!("Failed to update module configuration: {}", e)
                })),
            }
        } else {
            Json(serde_json::json!({
                "success": false,
                "message": format!("Module {} not found", module_name)
            }))
        }
    }

    /// Reset module to defaults
    async fn reset_module(
        Path(module_name): Path<String>,
        State(state): State<WebConfigState>,
    ) -> Json<serde_json::Value> {
        // Get current module configuration
        let mut module_config = state.get_config().module_config.clone();

        // Find the module to reset
        if let Some(module) = module_config.modules.get_mut(&module_name) {
            // Reset module to default configuration
            *module = ModuleConfig::default();

            // Save the updated configuration
            match state.update_module_config(module_config) {
                Ok(()) => Json(serde_json::json!({
                    "success": true,
                    "message": format!("Module {} reset to defaults successfully", module_name)
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "message": format!("Failed to reset module configuration: {}", e)
                })),
            }
        } else {
            Json(serde_json::json!({
                "success": false,
                "message": format!("Module {} not found", module_name)
            }))
        }
    }

    /// Update method configuration
    async fn update_method(
        Path((module_name, method_name)): Path<(String, String)>,
        State(state): State<WebConfigState>,
        Json(update): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        // Get current module configuration
        let mut module_config = state.get_config().module_config.clone();

        // Find the module and method to update
        if let Some(module) = module_config.modules.get_mut(&module_name) {
            if let Some(methods) = &mut module.methods {
                if let Some(method) = methods.get_mut(&method_name) {
                    // Process each field in the update object
                    for (field, value) in update.as_object().unwrap_or(&serde_json::Map::new()) {
                        match field.as_str() {
                            "enabled" => {
                                if let Some(enabled) = value.as_bool() {
                                    method.enabled = enabled;
                                }
                            }
                            "accessLevel" => {
                                if let Some(level) = value.as_str() {
                                    // Map string to AccessLevel enum
                                    match level.to_lowercase().as_str() {
                                        "public" => {
                                            method.access_level =
                                                Some(crate::config::module::AccessLevel::Public);
                                        }
                                        "internal" => {
                                            method.access_level =
                                                Some(crate::config::module::AccessLevel::Internal);
                                        }
                                        "private" => {
                                            method.access_level =
                                                Some(crate::config::module::AccessLevel::Private);
                                        }
                                        _ => {
                                            return Json(serde_json::json!({
                                                "success": false,
                                                "message": format!("Invalid access level: {}", level)
                                            }));
                                        }
                                    }
                                }
                            }
                            "rateLimit" => {
                                if let Some(limit_obj) = value.as_object() {
                                    // Parse rate limit configuration
                                    let requests_per_minute = limit_obj
                                        .get("requests_per_minute")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(60)
                                        as u32;
                                    let requests_per_hour = limit_obj
                                        .get("requests_per_hour")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(1000)
                                        as u32;
                                    let burst_capacity = limit_obj
                                        .get("burst_capacity")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(10)
                                        as u32;

                                    method.rate_limit =
                                        Some(crate::config::module::RateLimitConfig {
                                            requests_per_minute,
                                            requests_per_hour,
                                            burst_capacity,
                                        });
                                }
                            }
                            _ => {
                                info!(
                                    "Unknown field {} in update for method {}.{}",
                                    field, module_name, method_name
                                );
                            }
                        }
                    }

                    // Save the updated configuration
                    match state.update_module_config(module_config) {
                        Ok(()) => Json(serde_json::json!({
                            "success": true,
                            "message": format!("Method {}.{} updated successfully", module_name, method_name),
                            "data": update
                        })),
                        Err(e) => Json(serde_json::json!({
                            "success": false,
                            "message": format!("Failed to update method configuration: {}", e)
                        })),
                    }
                } else {
                    Json(serde_json::json!({
                        "success": false,
                        "message": format!("Method {} not found in module {}", method_name, module_name)
                    }))
                }
            } else {
                Json(serde_json::json!({
                    "success": false,
                    "message": format!("No methods found for module {}", module_name)
                }))
            }
        } else {
            Json(serde_json::json!({
                "success": false,
                "message": format!("Module {} not found", module_name)
            }))
        }
    }

    /// Get server-specific configuration
    async fn get_server_config(State(state): State<WebConfigState>) -> Json<serde_json::Value> {
        let config = state.get_config();

        // Extract server-specific settings
        let server_config = serde_json::json!({
            "base_url": config.api.base_url,
            "server_port": config.server.port,
            "log_level": config.server.log_level,
            "auth_mode": config.auth.mode,
            "direct_config": config.auth.direct_config,
            "login_config": config.auth.login_config,
            "token_refresh_buffer": config.auth.refresh_buffer,
            "token_expiry_time": config.auth.token_expiry,
            "max_retry_attempts": config.auth.max_retry_attempts,
        });

        Json(serde_json::json!({
            "success": true,
            "message": "Server configuration retrieved successfully",
            "config": server_config
        }))
    }

    /// Update server-specific configuration
    async fn update_server_config(
        State(state): State<WebConfigState>,
        Json(server_config): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        // Log incoming update request
        info!("Received server configuration update request");
        // Try to parse the entire configuration object first
        match serde_json::from_value::<crate::config::config::Config>(server_config.clone()) {
            Ok(new_config) => {
                // Full configuration object provided, update everything
                info!("Applying full configuration update");
                match state.update_config(new_config) {
                    Ok(()) => Json(serde_json::json!({
                        "success": true,
                        "message": "Server configuration updated successfully"
                    })),
                    Err(e) => Json(serde_json::json!({
                        "success": false,
                        "message": format!("Failed to update server configuration: {}", e)
                    })),
                }
            }
            Err(_) => {
                // Fall back to partial updates for backward compatibility
                info!("Applying partial configuration update");
                let mut config = state.get_config();
                let mut has_changes = false;

                // Update server-specific settings
                if let Some(config_obj) = server_config.as_object() {
                    // Handle base_url
                    if let Some(base_url) = config_obj.get("base_url").and_then(|v| v.as_str()) {
                        config.api.base_url = base_url.to_string();
                        info!("Updated base_url: {}", config.api.base_url);
                        has_changes = true;
                    }
                    
                    // Handle auth configuration
                    if let Some(auth_config) = config_obj.get("auth") {
                        if let Some(auth_obj) = auth_config.as_object() {
                            // Update auth mode
                            if let Some(mode) = auth_obj.get("mode").and_then(|v| v.as_str()) {
                                let normalized = match mode.to_lowercase().as_str() {
                                    "direct" => crate::config::config::AuthMode::Direct,
                                    "login" => crate::config::config::AuthMode::Login,
                                    _ => return Json(serde_json::json!({
                                        "success": false,
                                        "message": format!("Invalid auth mode: {}", mode)
                                    })),
                                };
                                config.auth.mode = normalized;
                                info!("Updated auth mode: {}", mode);
                            }
                            
                            // Update direct auth config (allow null to clear)
                            if let Some(direct_config_val) = auth_obj.get("direct_config") {
                                if direct_config_val.is_null() {
                                    config.auth.direct_config = None;
                                    info!("Cleared direct auth configuration");
                                } else {
                                    match serde_json::from_value::<crate::config::config::DirectAuthConfig>(direct_config_val.clone()) {
                                        Ok(direct_auth) => {
                                            config.auth.direct_config = Some(direct_auth);
                                            info!("Updated direct auth configuration");
                                        }
                                        Err(e) => {
                                            return Json(serde_json::json!({
                                                "success": false,
                                                "message": format!("Invalid direct auth configuration: {}", e)
                                            }));
                                        }
                                    }
                                }
                            }
                            
                            // Update login auth config (allow null to clear, sanitize method strings)
                            if let Some(login_config_val) = auth_obj.get("login_config") {
                                if login_config_val.is_null() {
                                    config.auth.login_config = None;
                                    info!("Cleared login auth configuration");
                                } else if let Some(mut lc_obj) = login_config_val.as_object().cloned() {
                                    // Sanitize required method field: default to POST on empty, uppercase strings
                                    if let Some(method_val) = lc_obj.get_mut("method") {
                                        if let Some(method_str) = method_val.as_str() {
                                            let trimmed = method_str.trim();
                                            if trimmed.is_empty() {
                                                *method_val = serde_json::Value::String("POST".to_string());
                                                info!("Login method empty; defaulted to POST");
                                            } else {
                                                *method_val = serde_json::Value::String(trimmed.to_ascii_uppercase());
                                            }
                                        }
                                    }

                                    // Sanitize optional refresh_method: null on empty, uppercase strings
                                    if let Some(refresh_val) = lc_obj.get_mut("refresh_method") {
                                        if let Some(refresh_str) = refresh_val.as_str() {
                                            let trimmed = refresh_str.trim();
                                            if trimmed.is_empty() {
                                                *refresh_val = serde_json::Value::Null;
                                                info!("Refresh method empty; cleared to null");
                                            } else {
                                                *refresh_val = serde_json::Value::String(trimmed.to_ascii_uppercase());
                                            }
                                        }
                                    }

                                    // Attempt deserialization with sanitized object
                                    match serde_json::from_value::<crate::config::config::LoginAuthConfig>(serde_json::Value::Object(lc_obj)) {
                                        Ok(login_auth) => {
                                            config.auth.login_config = Some(login_auth);
                                            info!("Updated login auth configuration");
                                        }
                                        Err(e) => {
                                            return Json(serde_json::json!({
                                                "success": false,
                                                "message": format!("Invalid login auth configuration: {}", e)
                                            }));
                                        }
                                    }
                                } else {
                                    // Fallback strict parsing for non-object values
                                    match serde_json::from_value::<crate::config::config::LoginAuthConfig>(login_config_val.clone()) {
                                        Ok(login_auth) => {
                                            config.auth.login_config = Some(login_auth);
                                            info!("Updated login auth configuration");
                                        }
                                        Err(e) => {
                                            return Json(serde_json::json!({
                                                "success": false,
                                                "message": format!("Invalid login auth configuration: {}", e)
                                            }));
                                        }
                                    }
                                }
                            }
                            
                            // Update auth token settings (support legacy names)
                            if let Some(token_expiry) = auth_obj
                                .get("token_expiry")
                                .and_then(|v| v.as_u64())
                                .or_else(|| auth_obj.get("token_expiry_time").and_then(|v| v.as_u64()))
                            {
                                config.auth.token_expiry = token_expiry;
                                info!("Updated token_expiry: {}", token_expiry);
                            }
                            
                            if let Some(refresh_buffer) = auth_obj
                                .get("refresh_buffer")
                                .and_then(|v| v.as_u64())
                                .or_else(|| auth_obj.get("token_refresh_buffer").and_then(|v| v.as_u64()))
                            {
                                config.auth.refresh_buffer = refresh_buffer;
                                info!("Updated refresh_buffer: {}", refresh_buffer);
                            }
                            
                            if let Some(max_retry) = auth_obj.get("max_retry_attempts").and_then(|v| v.as_u64()) {
                                config.auth.max_retry_attempts = max_retry as u32;
                                info!("Updated max_retry_attempts: {}", max_retry);
                            }
                            
                            has_changes = true;
                        }
                    }
                    
                    // Handle server configuration
                    if let Some(server_config) = config_obj.get("server") {
                        if let Some(server_obj) = server_config.as_object() {
                            if let Some(port) = server_obj.get("port").and_then(|v| v.as_u64()) {
                                config.server.port = port as u16;
                                info!("Updated server port: {}", config.server.port);
                            }
                            
                            if let Some(log_level) = server_obj.get("log_level").and_then(|v| v.as_str()) {
                                config.server.log_level = log_level.to_string();
                                info!("Updated server log_level: {}", config.server.log_level);
                            }
                            
                            has_changes = true;
                        }
                    }
                    
                    // Legacy field support for backward compatibility
                    if let Some(port) = config_obj.get("server_port").and_then(|v| v.as_u64()) {
                        config.server.port = port as u16;
                        info!("Updated legacy server_port: {}", config.server.port);
                        has_changes = true;
                    }
                    
                    if let Some(log_level) = config_obj.get("log_level").and_then(|v| v.as_str()) {
                        config.server.log_level = log_level.to_string();
                        info!("Updated legacy log_level: {}", config.server.log_level);
                        has_changes = true;
                    }

                    // Top-level auth fields support for backward compatibility
                    if let Some(mode) = config_obj.get("auth_mode").and_then(|v| v.as_str()) {
                        let normalized = match mode.to_lowercase().as_str() {
                            "direct" => crate::config::config::AuthMode::Direct,
                            "login" => crate::config::config::AuthMode::Login,
                            _ => {
                                return Json(serde_json::json!({
                                    "success": false,
                                    "message": format!("Invalid auth mode: {}", mode)
                                }))
                            }
                        };
                        config.auth.mode = normalized;
                        info!("Updated auth mode (top-level): {}", mode);
                        has_changes = true;
                    }

                    if let Some(direct_top) = config_obj.get("direct_config") {
                        if direct_top.is_null() {
                            config.auth.direct_config = None;
                            info!("Cleared direct auth configuration (top-level)");
                        } else {
                            match serde_json::from_value::<crate::config::config::DirectAuthConfig>(direct_top.clone()) {
                                Ok(dc) => {
                                    config.auth.direct_config = Some(dc);
                                    info!("Updated direct auth configuration (top-level)");
                                }
                                Err(e) => {
                                    return Json(serde_json::json!({
                                        "success": false,
                                        "message": format!("Invalid direct auth configuration: {}", e)
                                    }));
                                }
                            }
                        }
                        has_changes = true;
                    }

                    if let Some(login_top) = config_obj.get("login_config") {
                        if login_top.is_null() {
                            config.auth.login_config = None;
                            info!("Cleared login auth configuration (top-level)");
                        } else if let Some(mut lc_obj) = login_top.as_object().cloned() {
                            // Sanitize required method field: default to POST on empty, uppercase strings
                            if let Some(method_val) = lc_obj.get_mut("method") {
                                if let Some(method_str) = method_val.as_str() {
                                    let trimmed = method_str.trim();
                                    if trimmed.is_empty() {
                                        *method_val = serde_json::Value::String("POST".to_string());
                                        info!("Login method empty (top-level); defaulted to POST");
                                    } else {
                                        *method_val = serde_json::Value::String(trimmed.to_ascii_uppercase());
                                    }
                                }
                            }

                            // Sanitize optional refresh_method: null on empty, uppercase strings
                            if let Some(refresh_val) = lc_obj.get_mut("refresh_method") {
                                if let Some(refresh_str) = refresh_val.as_str() {
                                    let trimmed = refresh_str.trim();
                                    if trimmed.is_empty() {
                                        *refresh_val = serde_json::Value::Null;
                                        info!("Refresh method empty (top-level); cleared to null");
                                    } else {
                                        *refresh_val = serde_json::Value::String(trimmed.to_ascii_uppercase());
                                    }
                                }
                            }

                            match serde_json::from_value::<crate::config::config::LoginAuthConfig>(serde_json::Value::Object(lc_obj)) {
                                Ok(lc) => {
                                    config.auth.login_config = Some(lc);
                                    info!("Updated login auth configuration (top-level)");
                                }
                                Err(e) => {
                                    return Json(serde_json::json!({
                                        "success": false,
                                        "message": format!("Invalid login auth configuration: {}", e)
                                    }));
                                }
                            }
                        } else {
                            match serde_json::from_value::<crate::config::config::LoginAuthConfig>(login_top.clone()) {
                                Ok(lc) => {
                                    config.auth.login_config = Some(lc);
                                    info!("Updated login auth configuration (top-level)");
                                }
                                Err(e) => {
                                    return Json(serde_json::json!({
                                        "success": false,
                                        "message": format!("Invalid login auth configuration: {}", e)
                                    }));
                                }
                            }
                        }
                        has_changes = true;
                    }

                    if let Some(token_expiry_top) = config_obj.get("token_expiry_time").and_then(|v| v.as_u64()) {
                        config.auth.token_expiry = token_expiry_top;
                        info!("Updated token_expiry_time (top-level): {}", token_expiry_top);
                        has_changes = true;
                    }

                    if let Some(refresh_top) = config_obj.get("token_refresh_buffer").and_then(|v| v.as_u64()) {
                        config.auth.refresh_buffer = refresh_top;
                        info!("Updated token_refresh_buffer (top-level): {}", refresh_top);
                        has_changes = true;
                    }
                }

                if has_changes {
                    info!("Applying configuration changes to state");
                    match state.update_config(config) {
                        Ok(()) => Json(serde_json::json!({
                            "success": true,
                            "message": "Server configuration updated successfully"
                        })),
                        Err(e) => Json(serde_json::json!({
                            "success": false,
                            "message": format!("Failed to update server configuration: {}", e)
                        })),
                    }
                } else {
                    info!("No valid server configuration changes provided");
                    Json(serde_json::json!({
                        "success": false,
                        "message": "No valid server configuration changes provided"
                    }))
                }
            }
        }
    }

    /// Save configuration to file
    async fn save_config(
        State(state): State<WebConfigState>,
        Json(config_data): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        // Parse the configuration data
        if let Some(config_obj) = config_data.as_object() {
            let mut processed_any = false;

            // Extract main configuration if present (native shape)
            if let Some(config_value) = config_obj.get("config") {
                match serde_json::from_value::<Config>(config_value.clone()) {
                    Ok(config) => {
                        if let Err(e) = state.update_config(config) {
                            return Json(serde_json::json!({
                                "success": false,
                                "message": format!("Failed to save configuration: {}", e)
                            }));
                        }
                        processed_any = true;
                    }
                    Err(e) => {
                        return Json(serde_json::json!({
                            "success": false,
                            "message": format!("Invalid configuration format: {}", e)
                        }));
                    }
                }
            }

            // Extract module configuration if present (native shape)
            if let Some(module_config_value) = config_obj.get("module_config") {
                match serde_json::from_value::<GlobalModuleConfig>(module_config_value.clone()) {
                    Ok(module_config) => {
                        if let Err(e) = state.update_module_config(module_config) {
                            return Json(serde_json::json!({
                                "success": false,
                                "message": format!("Failed to save module configuration: {}", e)
                            }));
                        }
                        processed_any = true;
                    }
                    Err(e) => {
                        return Json(serde_json::json!({
                            "success": false,
                            "message": format!("Invalid module configuration format: {}", e)
                        }));
                    }
                }
            }

            // Backward compatibility: accept UI payload with { server: {config: {...}}, modules: [...] }
            // Persist server fields when provided
            if let Some(server_value) = config_obj.get("server") {
                // Allow either wrapper {config: {...}} or direct object
                let server_config_value = server_value
                    .as_object()
                    .and_then(|o| o.get("config").cloned())
                    .unwrap_or_else(|| server_value.clone());

                if let Some(sc_obj) = server_config_value.as_object() {
                    let mut config = state.get_config();
                    let mut has_changes = false;

                    // base_url
                    if let Some(base_url) = sc_obj.get("base_url").and_then(|v| v.as_str()) {
                        config.api.base_url = base_url.to_string();
                        has_changes = true;
                    }
                    // server_port
                    if let Some(port) = sc_obj.get("server_port").and_then(|v| v.as_u64()) {
                        config.server.port = port as u16;
                        has_changes = true;
                    }
                    // log_level
                    if let Some(log_level) = sc_obj.get("log_level").and_then(|v| v.as_str()) {
                        config.server.log_level = log_level.to_string();
                        has_changes = true;
                    }

                    // auth mode via top-level field
                    if let Some(mode) = sc_obj.get("auth_mode").and_then(|v| v.as_str()) {
                        let normalized = match mode.to_lowercase().as_str() {
                            "direct" => crate::config::config::AuthMode::Direct,
                            "login" => crate::config::config::AuthMode::Login,
                            _ => {
                                return Json(serde_json::json!({
                                    "success": false,
                                    "message": format!("Invalid auth mode: {}", mode)
                                }))
                            }
                        };
                        config.auth.mode = normalized;
                        has_changes = true;
                    }

                    // direct_config (nullable to clear)
                    if sc_obj.contains_key("direct_config") {
                        match sc_obj.get("direct_config") {
                            Some(v) if !v.is_null() => {
                                match serde_json::from_value::<crate::config::config::DirectAuthConfig>(v.clone()) {
                                    Ok(dc) => {
                                        config.auth.direct_config = Some(dc);
                                        has_changes = true;
                                    }
                                    Err(e) => {
                                        return Json(serde_json::json!({
                                            "success": false,
                                            "message": format!("Invalid direct auth configuration: {}", e)
                                        }))
                                    }
                                }
                            }
                            _ => {
                                config.auth.direct_config = None;
                                has_changes = true;
                            }
                        }
                    }

                    // login_config (nullable to clear) with method sanitization
                    if sc_obj.contains_key("login_config") {
                        match sc_obj.get("login_config") {
                            Some(v) if !v.is_null() => {
                                let mut sanitized = v.clone();
                                if let Some(lc_obj) = sanitized.as_object_mut() {
                                    // method: uppercase, default POST if empty
                                    if let Some(method_val) = lc_obj.get_mut("method") {
                                        if method_val.is_string() {
                                            let m = method_val.as_str().unwrap_or("").trim();
                                            if m.is_empty() {
                                                *method_val = serde_json::Value::String("POST".to_string());
                                            } else {
                                                *method_val = serde_json::Value::String(m.to_uppercase());
                                            }
                                        }
                                    }
                                    // refresh_method: set null if empty string
                                    if let Some(refresh_method_val) = lc_obj.get_mut("refresh_method") {
                                        if refresh_method_val.is_string() {
                                            let rm = refresh_method_val.as_str().unwrap_or("").trim();
                                            if rm.is_empty() {
                                                *refresh_method_val = serde_json::Value::Null;
                                            } else {
                                                *refresh_method_val = serde_json::Value::String(rm.to_uppercase());
                                            }
                                        }
                                    }
                                }

                                match serde_json::from_value::<crate::config::config::LoginAuthConfig>(sanitized) {
                                    Ok(lc) => {
                                        config.auth.login_config = Some(lc);
                                        has_changes = true;
                                    }
                                    Err(e) => {
                                        return Json(serde_json::json!({
                                            "success": false,
                                            "message": format!("Invalid login auth configuration: {}", e)
                                        }))
                                    }
                                }
                            }
                            _ => {
                                config.auth.login_config = None;
                                has_changes = true;
                            }
                        }
                    }

                    // token settings (legacy names)
                    if let Some(expiry) = sc_obj.get("token_expiry_time").and_then(|v| v.as_u64()) {
                        config.auth.token_expiry = expiry;
                        has_changes = true;
                    }
                    if let Some(buffer) = sc_obj.get("token_refresh_buffer").and_then(|v| v.as_u64()) {
                        config.auth.refresh_buffer = buffer as u64;
                        has_changes = true;
                    }
                    if let Some(max_retry) = sc_obj.get("max_retry_attempts").and_then(|v| v.as_u64()) {
                        config.auth.max_retry_attempts = max_retry as u32;
                        has_changes = true;
                    }

                    if has_changes {
                        if let Err(e) = state.update_config(config) {
                            return Json(serde_json::json!({
                                "success": false,
                                "message": format!("Failed to save server configuration: {}", e)
                            }));
                        }
                        processed_any = true;
                    }
                }
            }

            // Backward compatibility: accept simplified modules array from UI
            if let Some(modules_value) = config_obj.get("modules") {
                if let Some(modules_array) = modules_value.as_array() {
                    let mut module_config = state.get_config().module_config.clone();

                    for module in modules_array {
                        if let Some(mo) = module.as_object() {
                            if let Some(name) = mo.get("name").and_then(|v| v.as_str()) {
                                let entry = module_config.modules.entry(name.to_string()).or_default();

                                if let Some(enabled) = mo.get("enabled").and_then(|v| v.as_bool()) {
                                    entry.enabled = enabled;
                                }

                                if let Some(methods) = mo.get("methods").and_then(|v| v.as_array()) {
                                    for m in methods {
                                        if let Some(mo2) = m.as_object() {
                                            if let Some(mname) = mo2.get("name").and_then(|v| v.as_str()) {
                                                let menabled = mo2.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                                                if entry.methods.is_none() {
                                                    entry.methods = Some(std::collections::HashMap::new());
                                                }
                                                if let Some(map) = &mut entry.methods {
                                                    let method_entry = map.entry(mname.to_string()).or_insert(crate::config::module::MethodConfig::default());
                                                    method_entry.enabled = menabled;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Err(e) = state.update_module_config(module_config) {
                        return Json(serde_json::json!({
                            "success": false,
                            "message": format!("Failed to save modules configuration: {}", e)
                        }));
                    }
                    processed_any = true;
                } else {
                    return Json(serde_json::json!({
                        "success": false,
                        "message": "Invalid modules format: expected array"
                    }));
                }
            }

            if processed_any {
                Json(serde_json::json!({
                    "success": true,
                    "message": "Configuration saved successfully"
                }))
            } else {
                Json(serde_json::json!({
                    "success": false,
                    "message": "No valid configuration fields provided"
                }))
            }
        } else {
            Json(serde_json::json!({
                "success": false,
                "message": "Invalid configuration data format"
            }))
        }
    }

    /// Save preset configuration
    async fn save_preset(
        State(state): State<WebConfigState>,
        Json(preset_data): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        // Parse preset data
        if let Some(preset_obj) = preset_data.as_object() {
            // Extract preset information
            let preset_id = preset_obj
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let preset_name = preset_obj
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unnamed Preset".to_string());

            let preset_description = preset_obj
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Custom preset configuration".to_string());

            let module_config = state.get_config().module_config.clone();
            if let Some(id) = preset_id {
                // Build PresetConfig from the data - now directly using the module configs
                let preset_config = crate::config::preset_loader::PresetConfig {
                    name: preset_name,
                    description: preset_description,
                    default_access_level: Some(module_config.default_access_level.clone()),
                    default_rate_limit: module_config.default_rate_limit.clone(),
                    modules: module_config.modules.clone(),
                };

                let id_clone = id.clone();
                match state.save_preset(id, preset_config) {
                    Ok(()) => Json(serde_json::json!({
                        "success": true,
                        "message": format!("Preset '{}' saved successfully", id_clone)
                    })),
                    Err(e) => Json(serde_json::json!({
                        "success": false,
                        "message": format!("Failed to save preset: {}", e)
                    })),
                }
            } else {
                Json(serde_json::json!({
                    "success": false,
                    "message": "Missing preset ID or module configuration data"
                }))
            }
        } else {
            Json(serde_json::json!({
                "success": false,
                "message": "Invalid preset data format"
            }))
        }
    }

    /// Delete preset configuration
    async fn delete_preset(
        Path(preset_id): Path<String>,
        State(state): State<WebConfigState>,
    ) -> Json<serde_json::Value> {
        let preset_id_clone = preset_id.clone();
        match state.delete_preset(preset_id) {
            Ok(()) => Json(serde_json::json!({
                "success": true,
                "message": format!("Preset '{}' deleted successfully", preset_id_clone)
            })),
            Err(e) => Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to delete preset: {}", e)
            })),
        }
    }
}

/// Create default configuration presets
pub fn create_default_presets() -> HashMap<String, GlobalModuleConfig> {
    let mut presets = HashMap::new();

    // Product preset
    let mut product_config = GlobalModuleConfig::default();
    product_config
        .modules
        .insert("product".to_string(), ModuleConfig::default());
    product_config
        .modules
        .insert("plan".to_string(), ModuleConfig::default());
    product_config
        .modules
        .insert("story".to_string(), ModuleConfig::default());
    presets.insert("product".to_string(), product_config);

    // Project preset
    let mut project_config = GlobalModuleConfig::default();
    project_config
        .modules
        .insert("project".to_string(), ModuleConfig::default());
    project_config
        .modules
        .insert("task".to_string(), ModuleConfig::default());
    project_config
        .modules
        .insert("execution".to_string(), ModuleConfig::default());
    presets.insert("project".to_string(), project_config);

    // Execution preset
    let mut execution_config = GlobalModuleConfig::default();
    execution_config
        .modules
        .insert("execution".to_string(), ModuleConfig::default());
    execution_config
        .modules
        .insert("task".to_string(), ModuleConfig::default());
    execution_config
        .modules
        .insert("build".to_string(), ModuleConfig::default());
    presets.insert("execution".to_string(), execution_config);

    // Test preset
    let mut test_config = GlobalModuleConfig::default();
    test_config
        .modules
        .insert("test".to_string(), ModuleConfig::default());
    test_config
        .modules
        .insert("testcase".to_string(), ModuleConfig::default());
    test_config
        .modules
        .insert("testtask".to_string(), ModuleConfig::default());
    presets.insert("test".to_string(), test_config);

    presets
}
