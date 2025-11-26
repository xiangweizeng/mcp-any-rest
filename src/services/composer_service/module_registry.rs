//! Module registry for dynamic service registration and management

use crate::config::dynamic::DynamicConfigManager;
use crate::services::auth_service::UnifiedAuthService;
use anyhow::Result;
use log::{debug, error, info};
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

/// Trait for all ZenTao MCP service modules
pub trait DynamicModule: Send + Sync {
    /// Get the module name
    fn module_name(&self) -> &'static str;

    /// Get the module description
    fn module_description(&self) -> &'static str;

    /// Get the module version
    fn module_version(&self) -> &'static str;

    /// Get tools from this module (as a ServerHandler)
    fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListToolsResult, McpError>> + Send + '_>>;

    /// Get prompts from this module (as a ServerHandler)
    fn list_prompts(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListPromptsResult, McpError>> + Send + '_>>;

    /// List resources provided by this module (as a ServerHandler)
    fn list_resources(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListResourcesResult, McpError>> + Send + '_>>;

    /// Call a tool on this module (as a ServerHandler)
    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send + '_>>;

    /// Call get_prompt on this module (as a ServerHandler)
    fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<GetPromptResult, McpError>> + Send + '_>>;

    /// Call read_resource on this module (as a ServerHandler)
    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ReadResourceResult, McpError>> + Send + '_>>;
}

/// Service registry for managing all ZenTao MCP service modules
#[derive(Clone)]
pub struct ServiceRegistry {
    #[allow(dead_code)]
    pub config: Arc<DynamicConfigManager>,
    #[allow(dead_code)]
    pub auth_service: Arc<UnifiedAuthService>,
    modules: Arc<RwLock<HashMap<String, Arc<dyn DynamicModule>>>>,
    tool_module_map: Arc<RwLock<HashMap<String, String>>>,
    prompt_module_map: Arc<RwLock<HashMap<String, String>>>,
    resource_module_map: Arc<RwLock<HashMap<String, String>>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new(config: Arc<DynamicConfigManager>, auth_service: Arc<UnifiedAuthService>) -> Self {
        info!("Creating new ServiceRegistry");


        Self {
            config: config.clone(),
            auth_service: auth_service.clone(),
            modules: Arc::new(RwLock::new(HashMap::new())),
            tool_module_map: Arc::new(RwLock::new(HashMap::new())),
            prompt_module_map: Arc::new(RwLock::new(HashMap::new())),
            resource_module_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service module
    pub fn register_module<T>(&self, module: T) -> Result<()>
    where
        T: DynamicModule + 'static,
    {
        let module_name = module.module_name().to_string();
        let module_arc = Arc::new(module);

        info!(
            "Registering module: {} - {}",
            module_name,
            module_arc.module_description()
        );

        // Add module to registry
        {
            let mut modules = self.modules.write().unwrap();
            modules.insert(module_name.clone(), module_arc.clone());
        }

        debug!("Module {} registered successfully", module_name);

        Ok(())
    }

    /// Register a module dynamically using the module factory
    pub fn register_dynamic_module(&self, module_name: &str) -> Result<(), McpError> {
        // Check if module is enabled in configuration
        let config = self.config.get_config();
        if let Some(module_config) = config.get_module_config(module_name) {
            if !module_config.enabled {
                debug!(
                    "Module '{}' is disabled in configuration, skipping registration",
                    module_name
                );
                return Ok(());
            }
        }

        debug!("Registering dynamic module: {}", module_name);

        Ok(())
    }

    /// Get a module by name
    pub fn get_module(&self, module_name: &str) -> Option<Arc<dyn DynamicModule>> {
        let modules = self.modules.read().unwrap();
        modules.get(module_name).cloned()
    }

    /// Get all registered modules
    pub fn get_all_modules(&self) -> Vec<Arc<dyn DynamicModule>> {
        let modules = self.modules.read().unwrap();
        modules.values().cloned().collect()
    }

    /// Get module name for a specific tool by parsing the prefix
    pub async fn get_module_for_tool(
        &self,
        tool_name: &str,
        _context: RequestContext<RoleServer>,
    ) -> Option<String> {
        // Parse the tool name to extract module name from prefix format: module_name_tool_name
        let config = self.config.get_config();
        if let Some(prefix_end) = tool_name.find('_') {
            if prefix_end > 0 && prefix_end < tool_name.len() - 1 {
                let module_name = &tool_name[0..prefix_end]; // Extract module name before _

                // Check if the module exists and is enabled
                if self.has_module(module_name) && config.is_module_enabled(module_name) {
                    // Extract the original tool name
                    let original_tool_name = &tool_name[prefix_end + 1..]; // Extract tool name after _

                    // Check if the specific tool is enabled
                    if config.is_method_enabled(module_name, original_tool_name) {
                        debug!(
                            "ServiceRegistry: Found module '{}' for tool '{}' via prefix (enabled)",
                            module_name, tool_name
                        );
                        return Some(module_name.to_string());
                    } else {
                        debug!(
                            "ServiceRegistry: Tool '{}_{}' is disabled in configuration, skipping",
                            module_name, original_tool_name
                        );
                        return None;
                    }
                }
            }
        }

        // Fallback to old behavior if prefix parsing fails
        debug!(
            "ServiceRegistry: No valid prefix found in tool name '{}', falling back to search",
            tool_name
        );

        // Collect module references before entering async context to avoid holding lock across await
        let module_refs: Vec<(String, Arc<dyn DynamicModule>)> = {
            let modules = self.modules.read().unwrap();
            modules
                .iter()
                .filter(|(module_name, _)| config.is_module_enabled(module_name))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        };

        for (module_name, module) in &module_refs {
            match module.list_tools(None, _context.clone()).await {
                Ok(result) => {
                    if result.tools.iter().any(|tool| {
                        tool.name == tool_name && config.is_method_enabled(&module_name, &tool.name)
                    }) {
                        return Some(module_name.clone());
                    }
                }
                Err(_) => continue,
            }
        }
        None
    }

    /// Get module name for a specific prompt by parsing the prefix
    pub async fn get_module_for_prompt(
        &self,
        prompt_name: &str,
        _context: RequestContext<RoleServer>,
    ) -> Option<String> {
        // Parse the prompt name to extract module name from prefix format: module_name/prompt_name
        let config = self.config.get_config();
        if let Some(prefix_end) = prompt_name.find('/') {
            if prefix_end > 0 && prefix_end < prompt_name.len() - 1 {
                let module_name = &prompt_name[0..prefix_end]; // Extract module name before /

                // Check if the module exists and is enabled
                if self.has_module(module_name) && config.is_module_enabled(module_name) {
                    // Extract the original prompt name
                    let original_prompt_name = &prompt_name[prefix_end + 1..]; // Extract prompt name after /

                    // Check if the specific prompt is enabled
                    if config.is_method_enabled(module_name, original_prompt_name) {
                        debug!(
                            "ServiceRegistry: Found module '{}' for prompt '{}' via prefix (enabled)",
                            module_name, prompt_name
                        );
                        return Some(module_name.to_string());
                    } else {
                        debug!(
                            "ServiceRegistry: Prompt '{}/{}' is disabled in configuration, skipping",
                            module_name, original_prompt_name
                        );
                        return None;
                    }
                }
            }
        }

        // Fallback to old behavior if prefix parsing fails
        debug!(
            "ServiceRegistry: No valid prefix found in prompt name '{}', falling back to search",
            prompt_name
        );

        // Collect module references before entering async context to avoid holding lock across await
        let module_refs: Vec<(String, Arc<dyn DynamicModule>)> = {
            let modules = self.modules.read().unwrap();
            modules
                .iter()
                .filter(|(module_name, _)| config.is_module_enabled(module_name))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        };

        for (module_name, module) in &module_refs {
            match module.list_prompts(None, _context.clone()).await {
                Ok(result) => {
                    if result.prompts.iter().any(|prompt| {
                        prompt.name == prompt_name
                            && config.is_method_enabled(&module_name, &prompt.name)
                    }) {
                        return Some(module_name.clone());
                    }
                }
                Err(_) => continue,
            }
        }
        None
    }

    /// Get module name for a specific resource URI by parsing the prefix
    pub async fn get_module_for_resource(
        &self,
        resource_uri: &str,
        _context: RequestContext<RoleServer>,
    ) -> Option<String> {
        // Parse the resource URI to extract module name from prefix format: module_name/resource_uri
        let config = self.config.get_config();
        if let Some(prefix_end) = resource_uri.find('/') {
            if prefix_end > 0 && prefix_end < resource_uri.len() - 1 {
                let module_name = &resource_uri[0..prefix_end]; // Extract module name before /

                // Check if the module exists and is enabled
                if self.has_module(module_name) && config.is_module_enabled(module_name) {
                    // Extract the original resource URI
                    let original_resource_uri = &resource_uri[prefix_end + 1..]; // Extract resource URI after /

                    // Check if the specific resource is enabled
                    if config.is_resource_enabled(module_name, original_resource_uri) {
                        debug!(
                            "ServiceRegistry: Found module '{}' for resource '{}' via prefix (enabled)",
                            module_name, resource_uri
                        );
                        return Some(module_name.to_string());
                    } else {
                        debug!(
                            "ServiceRegistry: Resource '{}/{}' is disabled in configuration, skipping",
                            module_name, original_resource_uri
                        );
                        return None;
                    }
                }
            }
        }

        // Fallback to old behavior if prefix parsing fails
        debug!(
            "ServiceRegistry: No valid prefix found in resource URI '{}', falling back to search",
            resource_uri
        );

        // Collect module references before entering async context to avoid holding lock across await
        let module_refs: Vec<(String, Arc<dyn DynamicModule>)> = {
            let modules = self.modules.read().unwrap();
            modules
                .iter()
                .filter(|(module_name, _)| config.is_module_enabled(module_name))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        };

        for (module_name, module) in &module_refs {
            match module.list_resources(None, _context.clone()).await {
                Ok(result) => {
                    if result.resources.iter().any(|resource| {
                        resource.uri == resource_uri
                            && config.is_resource_enabled(&module_name, &resource.uri)
                    }) {
                        return Some(module_name.clone());
                    }
                }
                Err(_) => continue,
            }
        }
        None
    }

    /// Get all registered module names
    pub fn get_module_names(&self) -> Vec<String> {
        let modules = self.modules.read().unwrap();
        modules.keys().cloned().collect()
    }

    /// Get the number of registered modules
    pub fn get_module_count(&self) -> usize {
        let modules = self.modules.read().unwrap();
        modules.len()
    }

    /// Get module statistics
    pub fn get_module_stats(&self) -> ModuleStats {
        let modules = self.modules.read().unwrap();
        let tool_map = self.tool_module_map.read().unwrap();

        ModuleStats {
            total_modules: modules.len(),
            total_tools: tool_map.len(),
            module_names: modules.keys().cloned().collect(),
        }
    }

    /// Check if a module is registered
    pub fn has_module(&self, module_name: &str) -> bool {
        let modules = self.modules.read().unwrap();
        modules.contains_key(module_name)
    }

    /// Unregister a module
    pub fn unregister_module(&self, module_name: &str) -> Result<()> {
        info!("Unregistering module: {}", module_name);

        // Remove module from registry
        let module_arc = {
            let mut modules = self.modules.write().unwrap();
            modules.remove(module_name)
        };

        if let Some(_module) = module_arc {
            // Since we can't get tool/prompt/resource names synchronously anymore,
            // we'll remove the module from the mappings by filtering out entries
            // that belong to the unregistered module

            // Remove tools from tool-module mapping
            let mut tool_map = self.tool_module_map.write().unwrap();
            tool_map.retain(|_, module| module != module_name);

            // Remove prompts from prompt-module mapping
            let mut prompt_map = self.prompt_module_map.write().unwrap();
            prompt_map.retain(|_, module| module != module_name);

            // Remove resources from resource-module mapping
            let mut resource_map = self.resource_module_map.write().unwrap();
            resource_map.retain(|_, module| module != module_name);

            debug!("Module {} unregistered successfully", module_name);
        }

        Ok(())
    }

    /// Aggregate all tools from all modules
    pub async fn aggregate_tools(
        &self,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        debug!("ServiceRegistry: Aggregating tools from all modules");

        // Collect module references before entering async context to avoid holding lock across await
        let module_refs: Vec<(String, Arc<dyn DynamicModule>)> = {
            let modules = self.modules.read().unwrap();
            modules
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        };

        let mut all_tools = Vec::new();

        let config = self.config.get_config();
        for (module_name, module) in &module_refs {
            // Check if module is enabled in configuration
            if !config.is_module_enabled(module_name) {
                debug!(
                    "ServiceRegistry: Module '{}' is disabled in configuration, skipping tools",
                    module_name
                );
                continue;
            }

            match module.list_tools(None, context.clone()).await {
                Ok(result) => {
                    debug!(
                        "ServiceRegistry: Module '{}' provides {} tools",
                        module_name,
                        result.tools.len()
                    );

                    // Add module name prefix to each tool, but only include enabled tools
                    let prefixed_tools: Vec<Tool> = result
                        .tools
                        .into_iter()
                        .filter(|tool| {
                            let tool_name = tool.name.to_string();
                            let is_enabled = config.is_method_enabled(module_name, &tool_name);
                            if !is_enabled {
                                debug!("ServiceRegistry: Tool '{}_{}' is disabled in configuration, skipping", module_name, tool_name);
                            }
                            is_enabled
                        })
                        .map(|mut tool| {
                            tool.name = format!("{}_{}", module_name, tool.name).into();
                            tool
                        })
                        .collect();

                    all_tools.extend(prefixed_tools);
                }
                Err(e) => {
                    error!(
                        "ServiceRegistry: Failed to get tools from module '{}': {}",
                        module_name, e
                    );
                    return Err(e);
                }
            }
        }

        debug!(
            "ServiceRegistry: Aggregated {} tools from {} modules",
            all_tools.len(),
            module_refs.len()
        );
        Ok(ListToolsResult {
            tools: all_tools,
            next_cursor: None,
        })
    }

    /// Aggregate prompts from all registered modules
    pub async fn aggregate_prompts(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<Vec<Prompt>, McpError> {
        // Collect module references before entering async context to avoid holding lock across await
        let module_refs: Vec<Arc<dyn DynamicModule>> = {
            let modules = self.modules.read().unwrap();
            modules.values().cloned().collect()
        };

        let mut all_prompts = Vec::new();
        let config = self.config.get_config();
        for module in &module_refs {
            let module_name = module.module_name();

            // Check if module is enabled in configuration
            if !config.is_module_enabled(module_name) {
                debug!(
                    "ServiceRegistry: Module '{}' is disabled in configuration, skipping prompts",
                    module_name
                );
                continue;
            }

            let module_prompts = module.list_prompts(None, _context.clone()).await?.prompts;

            // Add module name prefix to each prompt, but only include enabled prompts
            let prefixed_prompts: Vec<Prompt> = module_prompts
                .into_iter()
                .filter(|prompt| {
                    let prompt_name = prompt.name.to_string();
                    let is_enabled = config.is_method_enabled(module_name, &prompt_name);
                    if !is_enabled {
                        debug!("ServiceRegistry: Prompt '{}_{}' is disabled in configuration, skipping", module_name, prompt_name);
                    }
                    is_enabled
                })
                .map(|mut prompt| {
                    prompt.name = format!("{}_{}", module_name, prompt.name).into();
                    prompt
                })
                .collect();

            all_prompts.extend(prefixed_prompts);
        }

        debug!(
            "Aggregated {} prompts from {} modules",
            all_prompts.len(),
            module_refs.len()
        );
        Ok(all_prompts)
    }

    /// Aggregate resources from all registered modules
    pub async fn aggregate_resources(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<Vec<Resource>, McpError> {
        // Collect module references before entering async context to avoid holding lock across await
        let module_refs: Vec<Arc<dyn DynamicModule>> = {
            let modules = self.modules.read().unwrap();
            modules.values().cloned().collect()
        };

        let mut all_resources = Vec::new();
        let config = self.config.get_config();
        for module in &module_refs {
            let module_name = module.module_name();

            // Check if module is enabled in configuration
            if !config.is_module_enabled(module_name) {
                debug!(
                    "ServiceRegistry: Module '{}' is disabled in configuration, skipping resources",
                    module_name
                );
                continue;
            }

            let module_resources = module
                .list_resources(None, _context.clone())
                .await?
                .resources;

            // Add module name prefix to each resource, but only include enabled resources
            let prefixed_resources: Vec<Resource> = module_resources
                .into_iter()
                .filter(|resource| {
                    let resource_uri = resource.uri.to_string();
                    let is_enabled = config.is_resource_enabled(module_name, &resource_uri);
                    if !is_enabled {
                        debug!("ServiceRegistry: Resource '{}_{}' is disabled in configuration, skipping", module_name, resource_uri);
                    }
                    is_enabled
                })
                .map(|mut resource| {
                    resource.uri = format!("{}_{}", module_name, resource.uri).into();
                    resource
                })
                .collect();

            all_resources.extend(prefixed_resources);
        }

        debug!(
            "Aggregated {} resources from {} modules",
            all_resources.len(),
            module_refs.len()
        );
        Ok(all_resources)
    }

    /// Route a tool call to the appropriate module
    pub async fn route_tool_call(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let mut modified_request = request.clone();
        let tool_name = request.name.to_string();
        debug!("ServiceRegistry: Routing tool call '{}'", tool_name);

        let config = self.config.get_config();
        // Find which module handles this tool
        if let Some(module_name) = self
            .get_module_for_tool(tool_name.as_ref(), context.clone())
            .await
        {
            if let Some(module) = self.get_module(&module_name) {
                // Check if module is enabled in configuration
                if !config.is_module_enabled(&module_name) {
                    error!(
                        "ServiceRegistry: Module '{}' is disabled in configuration, cannot route tool '{}'",
                        module_name, tool_name
                    );
                    return Err(McpError::internal_error(
                        format!("Module '{}' is disabled", module_name),
                        None,
                    ));
                }

                // Remove prefix from tool name before delegating to the module
                let original_tool_name = if let Some(prefix_end) = tool_name.find('_') {
                    if prefix_end > 0 && prefix_end < tool_name.len() - 1 {
                        // Extract the original tool name after the prefix
                        tool_name[prefix_end + 1..].to_string() // Skip "_"
                    } else {
                        tool_name.clone()
                    }
                } else {
                    tool_name.clone()
                };

                // Check if the specific tool is enabled in configuration
                if !config.is_method_enabled(&module_name, &original_tool_name) {
                    error!(
                        "ServiceRegistry: Tool '{}_{}' is disabled in configuration",
                        module_name, original_tool_name
                    );
                    return Err(McpError::internal_error(
                        format!("Tool '{}_{}' is disabled", module_name, original_tool_name),
                        None,
                    ));
                }

                debug!(
                    "ServiceRegistry: Routing tool '{}' to module '{}'",
                    tool_name, module_name
                );

                // Create new request with original tool name
                let modified_tool_name = original_tool_name.clone();
                modified_request.name = original_tool_name.into();
                debug!(
                    "ServiceRegistry: Modified tool name from '{}' to '{}' for module '{}'",
                    tool_name, modified_tool_name, module_name
                );

                // Delegate the tool call to the appropriate module with original name
                return module.call_tool(modified_request, context).await;
            } else {
                error!(
                    "ServiceRegistry: Module '{}' not found for tool '{}'",
                    module_name, tool_name
                );
                return Err(McpError::internal_error(
                    format!("Module '{}' not found", module_name),
                    None,
                ));
            }
        }

        // Tool not found in any module
        error!(
            "ServiceRegistry: Unknown tool '{}' - not found in any module",
            tool_name
        );
        Err(McpError::internal_error(
            format!("Unknown tool '{}'", tool_name),
            None,
        ))
    }

    /// Route a prompt request to the appropriate module
    pub async fn route_prompt_request(
        &self,
        request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let prompt_name = &request.name;
        debug!("ServiceRegistry: Routing prompt request '{}'", prompt_name);

        let config = self.config.get_config();
        // Find which module handles this prompt
        if let Some(module_name) = self
            .get_module_for_prompt(prompt_name, context.clone())
            .await
        {
            if let Some(module) = self.get_module(&module_name) {
                // Check if module is enabled in configuration
                if !config.is_module_enabled(&module_name) {
                    error!(
                        "ServiceRegistry: Module '{}' is disabled in configuration, cannot route prompt '{}'",
                        module_name, prompt_name
                    );
                    return Err(McpError::internal_error(
                        format!("Module '{}' is disabled", module_name),
                        None,
                    ));
                }

                // Remove prefix from prompt name before delegating to the module
                let original_prompt_name = if let Some(prefix_end) = prompt_name.find('/') {
                    if prefix_end > 0 && prefix_end < prompt_name.len() - 1 {
                        // Extract the original prompt name after the prefix
                        prompt_name[prefix_end + 1..].to_string() // Skip "/"
                    } else {
                        prompt_name.clone()
                    }
                } else {
                    prompt_name.clone()
                };

                // Check if the specific prompt is enabled in configuration
                if !config.is_method_enabled(&module_name, &original_prompt_name) {
                    error!(
                        "ServiceRegistry: Prompt '{}/{}' is disabled in configuration",
                        module_name, original_prompt_name
                    );
                    return Err(McpError::internal_error(
                        format!(
                            "Prompt '{}/{}' is disabled",
                            module_name, original_prompt_name
                        ),
                        None,
                    ));
                }

                debug!(
                    "ServiceRegistry: Routing prompt '{}' to module '{}'",
                    prompt_name, module_name
                );

                // Create new request with original prompt name
                let mut modified_request = request.clone();
                modified_request.name = original_prompt_name.into();

                // Delegate the prompt request to the appropriate module with original name
                return module.get_prompt(modified_request, context).await;
            } else {
                error!(
                    "ServiceRegistry: Module '{}' not found for prompt '{}'",
                    module_name, prompt_name
                );
                return Err(McpError::internal_error(
                    format!("Module '{}' not found", module_name),
                    None,
                ));
            }
        }

        // Prompt not found in any module
        error!(
            "ServiceRegistry: Unknown prompt '{}' - not found in any module",
            prompt_name
        );
        Err(McpError::internal_error(
            format!("Unknown prompt '{}'", prompt_name),
            None,
        ))
    }

    /// Route a resource request to the appropriate module
    pub async fn route_resource_request(
        &self,
        request: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let resource_uri = &request.uri;
        debug!(
            "ServiceRegistry: Routing resource request '{}'",
            resource_uri
        );

        let config = self.config.get_config();
        // Find which module handles this resource
        if let Some(module_name) = self
            .get_module_for_resource(resource_uri, context.clone())
            .await
        {
            if let Some(module) = self.get_module(&module_name) {
                // Check if module is enabled in configuration
                if !config.is_module_enabled(&module_name) {
                    error!(
                        "ServiceRegistry: Module '{}' is disabled in configuration, cannot route resource '{}'",
                        module_name, resource_uri
                    );
                    return Err(McpError::internal_error(
                        format!("Module '{}' is disabled", module_name),
                        None,
                    ));
                }

                // Remove prefix from resource URI before delegating to the module
                let original_resource_uri = if let Some(prefix_end) = resource_uri.find('/') {
                    if prefix_end > 0 && prefix_end < resource_uri.len() - 1 {
                        // Extract the original resource URI after the prefix
                        resource_uri[prefix_end + 1..].to_string() // Skip "/"
                    } else {
                        resource_uri.clone()
                    }
                } else {
                    resource_uri.clone()
                };

                // Check if the specific resource is enabled in configuration
                if !config.is_resource_enabled(&module_name, &original_resource_uri) {
                    error!(
                        "ServiceRegistry: Resource '{}/{}' is disabled in configuration",
                        module_name, original_resource_uri
                    );
                    return Err(McpError::internal_error(
                        format!(
                            "Resource '{}/{}' is disabled",
                            module_name, original_resource_uri
                        ),
                        None,
                    ));
                }

                debug!(
                    "ServiceRegistry: Routing resource '{}' to module '{}'",
                    resource_uri, module_name
                );

                // Create new request with original resource URI
                let mut modified_request = request.clone();
                modified_request.uri = original_resource_uri.into();

                // Delegate the resource request to the appropriate module with original URI
                return module.read_resource(modified_request, context).await;
            } else {
                error!(
                    "ServiceRegistry: Module '{}' not found for resource '{}'",
                    module_name, resource_uri
                );
                return Err(McpError::internal_error(
                    format!("Module '{}' not found", module_name),
                    None,
                ));
            }
        }

        // Resource not found in any module
        error!(
            "ServiceRegistry: Unknown resource '{}' - not found in any module",
            resource_uri
        );
        Err(McpError::internal_error(
            format!("Unknown resource '{}'", resource_uri),
            None,
        ))
    }
}

/// Statistics about registered modules
#[derive(Debug, Clone)]
pub struct ModuleStats {
    pub total_modules: usize,
    pub total_tools: usize,
    pub module_names: Vec<String>,
}

impl ModuleStats {
    /// Create a string representation of the stats
    pub fn to_string(&self) -> String {
        format!(
            "Modules: {}, Tools: {}, Module Names: {:?}",
            self.total_modules, self.total_tools, self.module_names
        )
    }
}

/// Macro for easily implementing DynamicModule trait
#[macro_export]
macro_rules! impl_zentao_service_module {
    ($struct_name:ty, $module_name:expr, $description:expr, $version:expr) => {
        impl $crate::services::module_registry::DynamicModule for $struct_name {
            fn module_name(&self) -> &'static str {
                $module_name
            }

            fn module_description(&self) -> &'static str {
                $description
            }

            fn module_version(&self) -> &'static str {
                $version
            }

            fn list_tools(
                &self,
                _request: Option<PaginatedRequestParam>,
                _context: RequestContext<RoleServer>,
            ) -> Pin<
                Box<
                    dyn std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_,
                >,
            > {
                Box::pin(async move {
                    // Delegate to the ServerHandler implementation
                    ServerHandler::list_tools(self, _request, _context).await
                })
            }

            fn list_prompts(
                &self,
                _request: Option<PaginatedRequestParam>,
                _context: RequestContext<RoleServer>,
            ) -> Pin<Box<dyn Future<Output = Result<ListPromptsResult, McpError>> + Send + '_>>
            {
                Box::pin(async move {
                    // Delegate to the ServerHandler implementation
                    ServerHandler::list_prompts(self, _request, _context).await
                })
            }

            fn list_resources(
                &self,
                request: Option<PaginatedRequestParam>,
                context: RequestContext<RoleServer>,
            ) -> Pin<Box<dyn Future<Output = Result<ListResourcesResult, ErrorData>> + Send + '_>>
            {
                Box::pin(async move {
                    // Delegate to the ServerHandler implementation
                    ServerHandler::list_resources(self, request, context).await
                })
            }

            fn call_tool(
                &self,
                request: CallToolRequestParam,
                context: RequestContext<RoleServer>,
            ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send + '_>> {
                Box::pin(async move {
                    // Delegate to the ServerHandler implementation
                    ServerHandler::call_tool(self, request, context).await
                })
            }

            fn get_prompt(
                &self,
                request: GetPromptRequestParam>,
                context: RequestContext<RoleServer>,
            ) -> Pin<Box<dyn Future<Output = Result<GetPromptResult, ErrorData>> + Send + '_>> {
                Box::pin(async move {
                    // Delegate to the ServerHandler implementation
                    ServerHandler::get_prompt(self, request, context).await
                })
            }

            fn read_resource(
                &self,
                request: ReadResourceRequestParam>,
                context: RequestContext<RoleServer>,
            ) -> Pin<Box<dyn Future<Output = Result<ReadResourceResult, ErrorData>> + Send + '_>>
            {
                Box::pin(async move {
                    // Delegate to the ServerHandler implementation
                    ServerHandler::read_resource(self, request, context).await
                })
            }
        }
    };
}