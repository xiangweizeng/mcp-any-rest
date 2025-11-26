//! ZML module factory for dynamically generating modules from ZML AST

use std::sync::Arc;

use log::info;
use rmcp::ErrorData as McpError;

use crate::config::dynamic::DynamicConfigManager;
use crate::config::zml_loader::ZmlModuleLoader;
use crate::services::auth_service::UnifiedAuthService;
use crate::services::composer_service::module_registry::ServiceRegistry;
use crate::services::dynamic_service::zml_dynamic_service::ZmlDynamicService;

/// Factory that creates service modules from ZML loader
#[derive(Clone)]
pub struct ZmlModuleFactory {
    loader: Arc<ZmlModuleLoader>,
    config: Arc<DynamicConfigManager>,
    auth_service: Arc<UnifiedAuthService>,
}

impl ZmlModuleFactory {
    /// Create a new ZML module factory
    pub fn new(
        loader: Arc<ZmlModuleLoader>,
        config: Arc<DynamicConfigManager>,
        auth_service: Arc<UnifiedAuthService>,
    ) -> Self {
        info!("Creating ZML module factory");
        Self { loader, config, auth_service }
    }

    /// Get all enabled ZML modules based on GlobalModuleConfig
    pub fn get_enabled_modules(&self) -> Vec<String> {
        let cfg = self.config.get_config();
        self.loader.get_enabled_modules(&cfg.module_config)
    }

    /// Create a ZML dynamic service for the specified module
    pub fn create_module(&self, module_name: &str) -> Result<ZmlDynamicService, McpError> {
        let cfg = self.config.get_config();
        if !cfg.is_module_enabled(module_name) {
            return Err(McpError::invalid_params(
                format!("Module '{}' is not enabled", module_name),
                None,
            ));
        }

        let module = self
            .loader
            .get_module(module_name)
            .ok_or_else(|| McpError::invalid_params(format!("ZML module '{}' not found", module_name), None))?;

        Ok(ZmlDynamicService::new(
            Arc::new(module.clone()),
            self.loader.clone(),
            self.config.clone(),
            self.auth_service.clone(),
        ))
    }

    /// Register all enabled ZML modules into the service registry
    pub fn register_modules(&self, service_registry: &mut ServiceRegistry) -> Result<(), McpError> {
        let enabled_modules = self.get_enabled_modules();
        for module_name in enabled_modules {
            let module = self.create_module(&module_name)?;
            let _ = service_registry.register_module(module);
        }
        Ok(())
    }
}