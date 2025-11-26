//! ZML-based dynamic service for MCP-ANY-REST

use crate::config::dynamic::DynamicConfigManager;
use crate::config::zml_loader::ZmlModuleLoader;
use crate::services::auth_service::UnifiedAuthService;
use crate::services::composer_service::module_registry::DynamicModule;

use crate::services::dynamic_service::api_request_builder::build_api_request_zml;
use crate::services::dynamic_service::schema_builder::{build_input_schema_zml, build_output_schema_zml};
use crate::zml::ast::{MethodDef, Module};

use log::info;
use rmcp::{
    handler::server::wrapper::Parameters, model::*, service::RequestContext, ErrorData as McpError,
    Json, RoleServer,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// ZML dynamic service that reads methods from ZML AST modules
#[derive(Clone)]
pub struct ZmlDynamicService {
    module_name: String,
    module: Arc<Module>,
    loader: Arc<ZmlModuleLoader>,
    config: Arc<DynamicConfigManager>,
    auth_service: Arc<UnifiedAuthService>,
}

impl ZmlDynamicService {
    pub fn new(
        module: Arc<Module>,
        loader: Arc<ZmlModuleLoader>,
        config: Arc<DynamicConfigManager>,
        auth_service: Arc<UnifiedAuthService>,
    ) -> Self {
        info!("Creating ZML dynamic service for: {}", module.name);
        Self {
            module_name: module.name.clone(),
            module,
            loader,
            config,
            auth_service,
        }
    }

    /// Generate dynamic tool method from ZML method definition
    fn generate_dynamic_tool_method(
        &self,
        method_name: String,
        method_def: MethodDef,
    ) -> impl Fn(
        &Self,
        Parameters<HashMap<String, Value>>, 
    ) -> Pin<Box<dyn Future<Output = Result<Json<Value>, McpError>> + Send + '_>> + '_ {
        let module = self.module.clone();
        let loader = self.loader.clone();
        let auth_service = self.auth_service.clone();
        let config = self.config.clone();
        let method_name_owned = method_name.clone();
        let method_def_owned = method_def.clone();

        move |_self, params: Parameters<HashMap<String, Value>>| {
            let module = module.clone();
            let _loader = loader.clone();
            let auth_service = auth_service.clone();
            let config = config.clone();
            let method_def = method_def_owned.clone();
            let method_name = method_name_owned.clone();

            Box::pin(async move {
                info!(
                    "Executing ZML method: {}::{}({:?})",
                    module.name, method_name, params.0
                );

                // Validate and normalize parameters against ZML
                // let normalized = validate_parameters_zml(&params.0, &module, &method_def, Some(&loader))?;

                // Build API request
                let (endpoint, http_method, request_body) = 
                    build_api_request_zml(&params.0, &module, &method_def).map_err(|e| {
                        McpError::internal_error(format!("Failed to build API request: {}", e), None)
                    })?;
                
                // Make authenticated request
                // Convert reqwest::Method to auth_service::HttpMethod
                let auth_http_method = match http_method {
                    reqwest::Method::GET => crate::services::auth_service::auth_strategy::HttpMethod::GET,
                    reqwest::Method::POST => crate::services::auth_service::auth_strategy::HttpMethod::POST,
                    reqwest::Method::PUT => crate::services::auth_service::auth_strategy::HttpMethod::PUT,
                    reqwest::Method::DELETE => crate::services::auth_service::auth_strategy::HttpMethod::DELETE,
                    reqwest::Method::PATCH => crate::services::auth_service::auth_strategy::HttpMethod::PATCH,
                    _ => crate::services::auth_service::auth_strategy::HttpMethod::GET, // Default to GET
                };
                
                let config_data = config.get_config();
                let full_url = format!("{}/{}", config_data.api.base_url, endpoint);
                let response_json: Value = auth_service
                    .make_authenticated_request(auth_http_method, &full_url, None, request_body)
                    .await
                    .map_err(|e| McpError::internal_error(format!("API request failed: {}", e), None))?;

                // Validate response against ZML method response type
                // validate_response_zml(&response_json, &method_def, &module, Some(&loader))?;

                Ok(Json(response_json))
            })
        }
    }
}

impl DynamicModule for ZmlDynamicService {
    fn module_name(&self) -> &'static str {
        Box::leak(self.module_name.clone().into_boxed_str())
    }

    fn module_description(&self) -> &'static str {
        Box::leak(self.module.description.clone().unwrap_or_default().into_boxed_str())
    }

    fn module_version(&self) -> &'static str {
        Box::leak(
            self.module
                .version
                .clone()
                .unwrap_or_else(|| "1.0.0".to_string())
                .into_boxed_str(),
        )
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListToolsResult, McpError>> + Send + '_>> {
        Box::pin(async move {
            let mut tools = Vec::new();

            for (method_name, method_def) in &self.module.methods {
                // Build input/output schemas using ZML
                let input_schema = build_input_schema_zml(method_def, &self.module, Some(&self.loader));
                let output_schema = build_output_schema_zml(method_def, &self.module, Some(&self.loader));

                let tool = Tool {
                    name: method_name.clone().into(),
                    title: None,
                    description: method_def.description.clone().map(|d| d.into()),
                    input_schema: Arc::new(input_schema.as_object().unwrap().clone()),
                    output_schema: Some(Arc::new(output_schema.as_object().unwrap().clone())),
                    annotations: None,
                    icons: None,
                };
                tools.push(tool);
            }

            Ok(ListToolsResult { tools, next_cursor: None })
        })
    }

    fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListPromptsResult, McpError>> + Send + '_>> {
        Box::pin(async move { Ok(ListPromptsResult { prompts: Vec::new(), next_cursor: None }) })
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListResourcesResult, McpError>> + Send + '_>> {
        Box::pin(async move { Ok(ListResourcesResult { resources: Vec::new(), next_cursor: None }) })
    }

    fn get_prompt(
        &self,
        _request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<GetPromptResult, McpError>> + Send + '_>> {
        Box::pin(async move {
            Err(McpError::invalid_params(
                "Prompts not supported by ZML dynamic services",
                None,
            ))
        })
    }

    fn read_resource(
        &self,
        _request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ReadResourceResult, McpError>> + Send + '_>> {
        Box::pin(async move {
            Err(McpError::invalid_params(
                "Resources not supported by ZML dynamic services",
                None,
            ))
        })
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send + '_>> {
        Box::pin(async move {
            let tool_name = request.name.to_string();

            // Check module/method enablement via GlobalModuleConfig
            let config = self.config.get_config();
            if !config.is_method_enabled(&self.module_name, &tool_name) {
                return Err(McpError::invalid_params(
                    format!("Method '{}/{}' is disabled", self.module_name, tool_name),
                    None,
                ));
            }

            // Get method definition
            let method_def = self.module.methods.get(&tool_name).ok_or_else(|| {
                McpError::invalid_params(
                    format!("Method '{}' not found in ZML module '{}'", tool_name, self.module_name),
                    None,
                )
            })?;

            // Parse parameters (robust against null and non-object inputs)
            let args_value: Value = request.arguments.into();
            let params: HashMap<String, Value> = match args_value {
                Value::Null => HashMap::new(),
                Value::Object(map) => map.into_iter().collect(),
                _ => {
                    return Err(McpError::invalid_params(
                        "Arguments must be a JSON object",
                        None,
                    ));
                }
            };

            // Strictly reject unknown parameters to align with MCP schema additionalProperties: false
            let allowed_keys: HashSet<String> = method_def.params.keys().cloned().collect();
            let unknown_keys: Vec<String> = params
                .keys()
                .filter(|k| !allowed_keys.contains(*k))
                .cloned()
                .collect();
            if !unknown_keys.is_empty() {
                return Err(McpError::invalid_params(
                    format!("Unknown parameter(s): {}", unknown_keys.join(", ")),
                    None,
                ));
            }

            // Execute dynamic ZML method
            let dynamic_method = self.generate_dynamic_tool_method(tool_name.clone(), method_def.clone());
            let result = dynamic_method(self, Parameters(params)).await?;
    
            info!("Dynamic method '{}' executed successfully with result: {}", tool_name, serde_json::to_string(&result.0).unwrap_or_else(|_| "<unprintable>".to_string()));
            // Serialize JSON result to string
            let result_str = serde_json::to_string(&result.0)
                .map_err(|e| McpError::internal_error(format!("Failed to serialize result: {}", e), None))?;

            Ok(CallToolResult::success(vec![Content::text(result_str)]))
        })
    }
}