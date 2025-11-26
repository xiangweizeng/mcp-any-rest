//! ZML Compiler - Convert ZML AST to JSON Configuration

use serde_json::{Map, Value as JsonValue};
use std::collections::HashMap;

use crate::zml::ast::*;

/// Compiler Error Type
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("Compilation error: {message}")]
    CompilationError { message: String },
    #[error("Type conversion error: {message}")]
    TypeConversionError { message: String },
    #[error("Reference resolution error: {message}")]
    ReferenceError { message: String },
}

/// ZML Compiler
pub struct Compiler {
    /// Type resolution cache
    type_cache: HashMap<String, JsonValue>,
    /// Module resolution cache
    module_cache: HashMap<String, JsonValue>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            type_cache: HashMap::new(),
            module_cache: HashMap::new(),
        }
    }

    /// Compile module to JSON configuration
    pub fn compile_module(&mut self, module: &Module) -> Result<JsonValue, CompileError> {
        let mut module_json = Map::new();

        // Compile basic information
        self.compile_module_info(module, &mut module_json)?;

        // Compile type definitions
        self.compile_types(module, &mut module_json)?;

        // Compile method definitions
        self.compile_methods(module, &mut module_json)?;

        // Compile enum definitions
        self.compile_enums(module, &mut module_json)?;

        // Compile resource definitions
        self.compile_resources(module, &mut module_json)?;

        // Cache module
        let module_json_value = JsonValue::Object(module_json);
        self.module_cache
            .insert(module.name.clone(), module_json_value.clone());

        Ok(module_json_value)
    }

    /// Compile module basic information
    fn compile_module_info(
        &self,
        module: &Module,
        module_json: &mut Map<String, JsonValue>,
    ) -> Result<(), CompileError> {
        // Basic information
        module_json.insert("name".to_string(), JsonValue::String(module.name.clone()));

        if let Some(version) = &module.version {
            module_json.insert("version".to_string(), JsonValue::String(version.clone()));
        }

        if let Some(description) = &module.description {
            module_json.insert("description".to_string(), JsonValue::String(description.clone()));
        }

        if let Some(enabled) = module.enabled {
            module_json.insert("enabled".to_string(), JsonValue::Bool(enabled));
        }

        if let Some(access_level) = &module.access_level {
            let level_str = match access_level {
                AccessLevel::Public => "public",
                AccessLevel::Private => "private",
                AccessLevel::Internal => "internal",
            };
            module_json.insert("access_level".to_string(), JsonValue::String(level_str.to_string()));
        }

        if let Some(category) = &module.category {
            module_json.insert("category".to_string(), JsonValue::String(category.clone()));
        }

        Ok(())
    }

    /// Compile enum definitions
    fn compile_enums(
        &mut self,
        module: &Module,
        module_json: &mut Map<String, JsonValue>,
    ) -> Result<(), CompileError> {
        let mut enums_json = Map::new();

        for (enum_name, enum_def) in &module.enums {
            let enum_json = self.compile_enum_def(enum_def)?;
            enums_json.insert(enum_name.clone(), enum_json);
        }

        if !enums_json.is_empty() {
            module_json.insert("enums".to_string(), JsonValue::Object(enums_json));
        }

        Ok(())
    }

    /// Compile single enum definition
    fn compile_enum_def(&self, enum_def: &EnumDef) -> Result<JsonValue, CompileError> {
        let mut enum_json = Map::new();

        // Enum basic information
        enum_json.insert("name".to_string(), JsonValue::String(enum_def.name.clone()));

        if let Some(description) = &enum_def.description {
            enum_json.insert("description".to_string(), JsonValue::String(description.clone()));
        }

        // Compile enum values
        let mut values_json = Map::new();
        for (value_name, enum_value) in &enum_def.values {
            let value_json = self.compile_enum_value(enum_value)?;
            values_json.insert(value_name.clone(), value_json);
        }

        enum_json.insert("values".to_string(), JsonValue::Object(values_json));

        Ok(JsonValue::Object(enum_json))
    }

    /// Compile enum values
    fn compile_enum_value(&self, enum_value: &EnumValueDef) -> Result<JsonValue, CompileError> {
        let mut value_json = Map::new();

        value_json.insert("name".to_string(), JsonValue::String(enum_value.name.clone()));

        if let Some(value) = &enum_value.value {
            let value_json_value = self.compile_value(value)?;
            value_json.insert("value".to_string(), value_json_value);
        }

        if let Some(description) = &enum_value.description {
            value_json.insert("description".to_string(), JsonValue::String(description.clone()));
        }

        Ok(JsonValue::Object(value_json))
    }

    /// Compile type definitions
    fn compile_types(
        &mut self,
        module: &Module,
        module_json: &mut Map<String, JsonValue>,
    ) -> Result<(), CompileError> {
        let mut types_json = Map::new();

        for (type_name, type_def) in &module.types {
            let type_json = self.compile_type_def(type_def, module)?;
            types_json.insert(type_name.clone(), type_json);
        }

        if !types_json.is_empty() {
            module_json.insert("types".to_string(), JsonValue::Object(types_json));
        }

        Ok(())
    }

    /// Compile single type definition
    fn compile_type_def(
        &mut self,
        type_def: &TypeDef,
        module: &Module,
    ) -> Result<JsonValue, CompileError> {
        let mut type_json = Map::new();

        // Type basic information
        type_json.insert("name".to_string(), JsonValue::String(type_def.name.clone()));

        if let Some(description) = &type_def.description {
            type_json.insert("description".to_string(), JsonValue::String(description.clone()));
        }

        // Compile fields
        let mut properties = Map::new();
        let mut required = Vec::new();

        for (field_name, field_def) in &type_def.fields {
            let field_json = self.compile_field_def(field_def, module)?;
            properties.insert(field_name.clone(), field_json);

            if !field_def.optional {
                required.push(field_name.clone());
            }
        }

        type_json.insert("properties".to_string(), JsonValue::Object(properties));

        if !required.is_empty() {
            type_json.insert("required".to_string(), JsonValue::Array(
                required.into_iter().map(JsonValue::String).collect()
            ));
        }

        // Cache type definition
        let type_json_value = JsonValue::Object(type_json);
        self.type_cache
            .insert(type_def.name.clone(), type_json_value.clone());

        Ok(type_json_value)
    }

    /// Compile field definition
    fn compile_field_def(
        &mut self,
        field_def: &FieldDef,
        module: &Module,
    ) -> Result<JsonValue, CompileError> {
        let mut field_json = Map::new();

        // Compile type expression
        let type_json = self.compile_type_expr(&field_def.type_expr, module)?;
        field_json.insert("type".to_string(), type_json);

        // Optional field
        if field_def.optional {
            field_json.insert("optional".to_string(), JsonValue::Bool(true));
        }

        // Default value
        if let Some(default_value) = &field_def.default_value {
            let default_json = self.compile_value(default_value)?;
            field_json.insert("default".to_string(), default_json);
        }

        // Description
        if let Some(description) = &field_def.description {
            field_json.insert("description".to_string(), JsonValue::String(description.clone()));
        }

        Ok(JsonValue::Object(field_json))
    }

    /// Compile type expression
    fn compile_type_expr(
        &mut self,
        type_expr: &TypeExpr,
        module: &Module,
    ) -> Result<JsonValue, CompileError> {
        match type_expr {
            TypeExpr::String => Ok(JsonValue::String("string".to_string())),
            TypeExpr::Integer => Ok(JsonValue::String("integer".to_string())),
            TypeExpr::Number => Ok(JsonValue::String("number".to_string())),
            TypeExpr::Boolean => Ok(JsonValue::String("boolean".to_string())),
            TypeExpr::Date => Ok(JsonValue::String("date".to_string())),
            TypeExpr::DateTime => Ok(JsonValue::String("datetime".to_string())),
            TypeExpr::Any => Ok(JsonValue::String("any".to_string())),
            
            TypeExpr::Array(element_type) => {
                let mut array_json = Map::new();
                array_json.insert("type".to_string(), JsonValue::String("array".to_string()));
                
                let items_json = self.compile_type_expr(element_type, module)?;
                array_json.insert("items".to_string(), items_json);
                
                Ok(JsonValue::Object(array_json))
            }
            
            TypeExpr::Object(fields) => {
                let mut object_json = Map::new();
                object_json.insert("type".to_string(), JsonValue::String("object".to_string()));
                
                let mut properties = Map::new();
                let mut required = Vec::new();
                
                for (field_name, field_def) in fields {
                    let field_json = self.compile_field_def(field_def, module)?;
                    properties.insert(field_name.clone(), field_json);
                    
                    if !field_def.optional {
                        required.push(field_name.clone());
                    }
                }
                
                object_json.insert("properties".to_string(), JsonValue::Object(properties));
                
                if !required.is_empty() {
                    object_json.insert("required".to_string(), JsonValue::Array(
                        required.into_iter().map(JsonValue::String).collect()
                    ));
                }
                
                Ok(JsonValue::Object(object_json))
            }
            
            TypeExpr::Enum(values) => {
                let mut enum_json = Map::new();
                enum_json.insert("type".to_string(), JsonValue::String("enum".to_string()));
                
                let values_json: Vec<JsonValue> = values
                    .iter()
                    .map(|v| JsonValue::String(v.clone()))
                    .collect();
                
                enum_json.insert("values".to_string(), JsonValue::Array(values_json));
                Ok(JsonValue::Object(enum_json))
            }
            
            TypeExpr::Ref(ref_type_name) => {
                // Check if reference type exists
                if !module.types.contains_key(ref_type_name) {
                    return Err(CompileError::ReferenceError {
                        message: format!("Referenced type '{}' does not exist", ref_type_name),
                    });
                }
                
                // Return reference format
                let mut ref_json = Map::new();
                ref_json.insert("$ref".to_string(), JsonValue::String(format!("#/types/{}", ref_type_name)));
                Ok(JsonValue::Object(ref_json))
            }
            
            TypeExpr::Alias(alias_name) => {
                // Check if type alias exists (including type definitions and enum definitions)
                if !module.types.contains_key(alias_name) && !module.enums.contains_key(alias_name) {
                    return Err(CompileError::ReferenceError {
                        message: format!("Type alias '{}' does not exist", alias_name),
                    });
                }
                
                // Directly return type name
                Ok(JsonValue::String(alias_name.clone()))
            }
        }
    }

    /// Compile value
    fn compile_value(&self, value: &Value) -> Result<JsonValue, CompileError> {
        match value {
            Value::String(s) => Ok(JsonValue::String(s.clone())),
            Value::Integer(i) => Ok(JsonValue::Number((*i).into())),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    Ok(JsonValue::Number((*n as i64).into()))
                } else {
                    Ok(JsonValue::Number(serde_json::Number::from_f64(*n).unwrap()))
                }
            }
            Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
            Value::Array(arr) => {
                let items: Result<Vec<JsonValue>, CompileError> = 
                    arr.iter().map(|v| self.compile_value(v)).collect();
                Ok(JsonValue::Array(items?))
            }
            Value::Object(obj) => {
                let mut map = Map::new();
                for (k, v) in obj {
                    let compiled_value = self.compile_value(v)?;
                    map.insert(k.clone(), compiled_value);
                }
                Ok(JsonValue::Object(map))
            }
            Value::Null => Ok(JsonValue::Null),
        }
    }

    /// Compile method definitions
    fn compile_methods(
        &mut self,
        module: &Module,
        module_json: &mut Map<String, JsonValue>,
    ) -> Result<(), CompileError> {
        let mut methods_json = Map::new();

        for (method_name, method_def) in &module.methods {
            let method_json = self.compile_method_def(method_def, module)?;
            methods_json.insert(method_name.clone(), method_json);
        }

        if !methods_json.is_empty() {
            module_json.insert("methods".to_string(), JsonValue::Object(methods_json));
        }

        Ok(())
    }

    /// Compile single method definition
    fn compile_method_def(
        &mut self,
        method_def: &MethodDef,
        module: &Module,
    ) -> Result<JsonValue, CompileError> {
        let mut method_json = Map::new();

        // Basic information
        method_json.insert("name".to_string(), JsonValue::String(method_def.name.clone()));

        if let Some(description) = &method_def.description {
            method_json.insert("description".to_string(), JsonValue::String(description.clone()));
        }

        // HTTP method
        let http_method = match method_def.http_method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
        };
        method_json.insert("http_method".to_string(), JsonValue::String(http_method.to_string()));

        // URI
        method_json.insert("uri".to_string(), JsonValue::String(method_def.uri.clone()));

        // Access level
        let access_level = match method_def.access_level {
            AccessLevel::Public => "public",
            AccessLevel::Private => "private",
            AccessLevel::Internal => "internal",
        };
        method_json.insert("access_level".to_string(), JsonValue::String(access_level.to_string()));

        // Rate limit configuration
        if let Some(rate_limit) = &method_def.rate_limit {
            let mut rate_limit_json = Map::new();
            rate_limit_json.insert("requests".to_string(), JsonValue::Number(rate_limit.requests.into()));
            rate_limit_json.insert("per_seconds".to_string(), JsonValue::Number(rate_limit.per_seconds.into()));
            method_json.insert("rate_limit".to_string(), JsonValue::Object(rate_limit_json));
        }

        // Parameter definitions
        if !method_def.params.is_empty() {
            let mut params_json = Map::new();
            for (param_name, param_def) in &method_def.params {
                let param_json = self.compile_param_def(param_def, module)?;
                params_json.insert(param_name.clone(), param_json);
            }
            method_json.insert("params".to_string(), JsonValue::Object(params_json));
        }

        // Response definition
        let response_json = self.compile_type_expr(&method_def.response, module)?;
        method_json.insert("response".to_string(), response_json);

        Ok(JsonValue::Object(method_json))
    }

    /// Compile parameter definition
    fn compile_param_def(
        &mut self,
        param_def: &ParamDef,
        module: &Module,
    ) -> Result<JsonValue, CompileError> {
        let mut param_json = Map::new();

        param_json.insert("name".to_string(), JsonValue::String(param_def.name.clone()));

        let type_json = self.compile_type_expr(&param_def.type_expr, module)?;
        param_json.insert("type".to_string(), type_json);

        if param_def.optional {
            param_json.insert("optional".to_string(), JsonValue::Bool(true));
        }

        if let Some(default_value) = &param_def.default_value {
            let default_json = self.compile_value(default_value)?;
            param_json.insert("default".to_string(), default_json);
        }

        if let Some(description) = &param_def.description {
            param_json.insert("description".to_string(), JsonValue::String(description.clone()));
        }

        Ok(JsonValue::Object(param_json))
    }

    /// Compile resource definition
    fn compile_resources(
        &mut self,
        module: &Module,
        module_json: &mut Map<String, JsonValue>,
    ) -> Result<(), CompileError> {
        let mut resources_json = Map::new();

        for (resource_name, resource_def) in &module.resources {
            let resource_json = self.compile_resource_def(resource_def)?;
            resources_json.insert(resource_name.clone(), resource_json);
        }

        if !resources_json.is_empty() {
            module_json.insert("resources".to_string(), JsonValue::Object(resources_json));
        }

        Ok(())
    }

    /// Compile single resource definition
    fn compile_resource_def(&self, resource_def: &ResourceDef) -> Result<JsonValue, CompileError> {
        let mut resource_json = Map::new();

        resource_json.insert("name".to_string(), JsonValue::String(resource_def.name.clone()));

        let resource_type = match resource_def.resource_type {
            ResourceType::Collection => "collection",
            ResourceType::Entity => "entity",
        };
        resource_json.insert("type".to_string(), JsonValue::String(resource_type.to_string()));

        resource_json.insert("uri".to_string(), JsonValue::String(resource_def.uri.clone()));

        if let Some(description) = &resource_def.description {
            resource_json.insert("description".to_string(), JsonValue::String(description.clone()));
        }

        Ok(JsonValue::Object(resource_json))
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.type_cache.clear();
        self.module_cache.clear();
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_basic_type() {
        let mut compiler = Compiler::new();
        let module = Module {
            name: "TestModule".to_string(),
            extends: None,
            version: Some("1.0.0".to_string()),
            description: Some("Test module".to_string()),
            enabled: Some(true),
            access_level: Some(AccessLevel::Public),
            category: Some("test".to_string()),
            types: HashMap::new(),
            enums: HashMap::new(),
            methods: HashMap::new(),
            resources: HashMap::new(),
            templates: HashMap::new(),
        };

        let result = compiler.compile_module(&module);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert_eq!(json["name"], "TestModule");
        assert_eq!(json["version"], "1.0.0");
        assert_eq!(json["description"], "Test module");
        assert_eq!(json["enabled"], true);
        assert_eq!(json["access_level"], "public");
    }
}