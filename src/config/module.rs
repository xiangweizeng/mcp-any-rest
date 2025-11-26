//! Module configuration for ZenTao MCP Server
//! This module provides dynamic configuration for modules, methods and resources visibility

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Module visibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    /// Whether the module is enabled
    pub enabled: bool,
    /// Module description
    pub description: Option<String>,
    /// Methods configuration
    pub methods: Option<HashMap<String, MethodConfig>>,
    /// Resources configuration
    pub resources: Option<HashMap<String, ResourceConfig>>,
}

/// Method visibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodConfig {
    /// Whether the method is enabled
    pub enabled: bool,
    /// Method description
    pub description: Option<String>,
    /// Access level (public, internal, private)
    pub access_level: Option<AccessLevel>,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

/// Resource visibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Whether the resource is enabled
    pub enabled: bool,
    /// Resource description
    pub description: Option<String>,
    /// Access level
    pub access_level: Option<AccessLevel>,
    /// Resource type
    pub resource_type: Option<ResourceType>,
}

/// Access level for methods and resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Public access - available to all users
    Public,
    /// Internal access - available to authenticated users
    Internal,
    /// Private access - available to administrators only
    Private,
}

impl AccessLevel {
    /// Convert from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "public" => Some(AccessLevel::Public),
            "internal" => Some(AccessLevel::Internal),
            "private" => Some(AccessLevel::Private),
            _ => None,
        }
    }
}

/// Resource type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    /// API endpoint
    ApiEndpoint,
    /// Data resource
    DataResource,
    /// File resource
    FileResource,
    /// Other resource type
    Other(String),
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    pub requests_per_minute: u32,
    /// Maximum requests per hour
    pub requests_per_hour: u32,
    /// Burst capacity
    pub burst_capacity: u32,
}

/// Global module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalModuleConfig {
    /// Default access level for new modules
    pub default_access_level: AccessLevel,
    /// Default rate limiting configuration
    pub default_rate_limit: Option<RateLimitConfig>,
    /// Module-specific configurations
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub modules: HashMap<String, ModuleConfig>,
}

impl Default for GlobalModuleConfig {
    fn default() -> Self {
        Self {
            default_access_level: AccessLevel::Internal,
            default_rate_limit: Some(RateLimitConfig {
                requests_per_minute: 60,
                requests_per_hour: 1000,
                burst_capacity: 10,
            }),
            modules: HashMap::new(),
        }
    }
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            description: None,
            methods: None,
            resources: None,
        }
    }
}

impl Default for MethodConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            description: None,
            access_level: Some(AccessLevel::Internal),
            rate_limit: None,
        }
    }
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            description: None,
            access_level: Some(AccessLevel::Internal),
            resource_type: Some(ResourceType::ApiEndpoint),
        }
    }
}

impl GlobalModuleConfig {
    /// Check if a module is enabled
    /// Rule: No configuration means disabled
    pub fn is_module_enabled(&self, module_name: &str) -> bool {
        self.modules
            .get(module_name)
            .map(|config| config.enabled)
            .unwrap_or(false) // No configuration means disabled
    }

    /// Check if a method is enabled
    /// Rule: If module is enabled but method not configured, method is enabled
    ///       If module is disabled, method is disabled
    ///       If method is explicitly configured, use its enabled status
    pub fn is_method_enabled(&self, module_name: &str, method_name: &str) -> bool {
        // First check if module is enabled
        if !self.is_module_enabled(module_name) {
            return false;
        }
        
        // If module is enabled but method not configured, method is enabled
        self.modules
            .get(module_name)
            .and_then(|module_config| module_config.methods.as_ref())
            .and_then(|methods| methods.get(method_name))
            .map(|method_config| method_config.enabled)
            .unwrap_or(true) // Module enabled but method not configured means method enabled
    }

    /// Check if a resource is enabled
    /// Rule: If module is enabled but resource not configured, resource is enabled
    ///       If module is disabled, resource is disabled
    ///       If resource is explicitly configured, use its enabled status
    pub fn is_resource_enabled(&self, module_name: &str, resource_name: &str) -> bool {
        // First check if module is enabled
        if !self.is_module_enabled(module_name) {
            return false;
        }
        
        // If module is enabled but resource not configured, resource is enabled
        self.modules
            .get(module_name)
            .and_then(|module_config| module_config.resources.as_ref())
            .and_then(|resources| resources.get(resource_name))
            .map(|resource_config| resource_config.enabled)
            .unwrap_or(true) // Module enabled but resource not configured means resource enabled
    }

    /// Get method configuration
    pub fn get_method_config(&self, module_name: &str, method_name: &str) -> Option<&MethodConfig> {
        self.modules
            .get(module_name)
            .and_then(|module_config| module_config.methods.as_ref())
            .and_then(|methods| methods.get(method_name))
    }

    /// Get resource configuration
    pub fn get_resource_config(&self, module_name: &str, resource_name: &str) -> Option<&ResourceConfig> {
        self.modules
            .get(module_name)
            .and_then(|module_config| module_config.resources.as_ref())
            .and_then(|resources| resources.get(resource_name))
    }

    /// Get module configuration
    pub fn get_module_config(&self, module_name: &str) -> Option<&ModuleConfig> {
        self.modules.get(module_name)
    }
}

impl ModuleConfig {
    /// Create a new module configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a method configuration
    pub fn add_method(&mut self, method_name: String, config: MethodConfig) {
        if self.methods.is_none() {
            self.methods = Some(HashMap::new());
        }
        if let Some(methods) = &mut self.methods {
            methods.insert(method_name, config);
        }
    }

    /// Add a resource configuration
    pub fn add_resource(&mut self, resource_name: String, config: ResourceConfig) {
        if self.resources.is_none() {
            self.resources = Some(HashMap::new());
        }
        if let Some(resources) = &mut self.resources {
            resources.insert(resource_name, config);
        }
    }
}

impl MethodConfig {
    /// Create a new method configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
}

impl ResourceConfig {
    /// Create a new resource configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
}