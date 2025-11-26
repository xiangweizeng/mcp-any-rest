//! Schema builder for dynamic module service (ZML-based) and legacy JSON config
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

use crate::config::zml_loader::ZmlModuleLoader;

use crate::zml::ast::{
    EnumDef, FieldDef, MethodDef, Module, TypeDef, TypeExpr, Value as ZmlValue,
};

/// Maximum depth for nested object expansion to prevent infinite recursion
const MAX_NESTING_DEPTH: usize = 10;

/// Build input schema for tool based on ZML method definition
pub fn build_input_schema_zml(method: &MethodDef, module: &Module, loader: Option<&ZmlModuleLoader>) -> Value {
    let (properties, required) = build_schema_properties(method, module, loader);

    let mut schema = Map::new();
    // Declare JSON Schema dialect for MCP clients
    schema.insert(
        "$schema".to_string(),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    schema.insert("type".to_string(), Value::String("object".to_string()));
    schema.insert("properties".to_string(), Value::Object(properties.clone()));
    // Disallow unknown properties to enforce strict MCP parameter validation
    schema.insert(
        "additionalProperties".to_string(),
        Value::Bool(false),
    );

    // Attach method-level description at the schema root if available
    if let Some(desc) = &method.description {
        schema.insert("description".to_string(), Value::String(desc.clone()));
    }

    if !required.is_empty() {
        schema.insert(
            "required".to_string(),
            Value::Array(required.into_iter().map(Value::String).collect()),
        );
    }

    // Provide JSON-RPC 2.0 compliant request envelope as a definition
    // This allows clients or integrations that require JSON-RPC to reference a canonical schema
    let mut defs = Map::new();

    // params object mirrors the tool arguments schema
    let mut params_obj = Map::new();
    params_obj.insert("type".to_string(), Value::String("object".to_string()));
    params_obj.insert("properties".to_string(), Value::Object(properties));
    params_obj.insert("additionalProperties".to_string(), Value::Bool(false));
    let params_required = method
        .params
        .iter()
        .filter_map(|(k, v)| if !v.optional { Some(Value::String(k.clone())) } else { None })
        .collect::<Vec<_>>();
    if !params_required.is_empty() {
        params_obj.insert("required".to_string(), Value::Array(params_required));
    }

    // id can be string, integer, number or null (notifications omit id)
    let id_type = {
        let mut id = Map::new();
        id.insert(
            "type".to_string(),
            Value::Array(vec![
                Value::String("string".to_string()),
                Value::String("integer".to_string()),
                Value::String("number".to_string()),
                Value::String("null".to_string()),
            ]),
        );
        Value::Object(id)
    };

    // jsonrpc const "2.0"
    let jsonrpc_const = {
        let mut jr = Map::new();
        jr.insert("type".to_string(), Value::String("string".to_string()));
        jr.insert("const".to_string(), Value::String("2.0".to_string()));
        Value::Object(jr)
    };

    // method as string (not constraining to a single value to avoid coupling)
    let method_schema = json_type("string");

    let mut rpc_props = Map::new();
    rpc_props.insert("jsonrpc".to_string(), jsonrpc_const);
    rpc_props.insert("method".to_string(), method_schema);
    rpc_props.insert("params".to_string(), Value::Object(params_obj));
    rpc_props.insert("id".to_string(), id_type);

    let mut rpc_request = Map::new();
    rpc_request.insert("type".to_string(), Value::String("object".to_string()));
    rpc_request.insert("properties".to_string(), Value::Object(rpc_props));
    rpc_request.insert("required".to_string(), Value::Array(vec![
        Value::String("jsonrpc".to_string()),
        Value::String("method".to_string()),
        Value::String("params".to_string()),
    ]));
    rpc_request.insert("additionalProperties".to_string(), Value::Bool(false));

    defs.insert("JsonRpcRequest".to_string(), Value::Object(rpc_request));
    schema.insert("$defs".to_string(), Value::Object(defs));

    Value::Object(schema)
}

/// Build schema properties from ZML method definition
fn build_schema_properties(
    method: &MethodDef,
    module: &Module,
    loader: Option<&ZmlModuleLoader>,
) -> (Map<String, Value>, Vec<String>) {
    build_schema_properties_with_depth(method, module, loader, 0)
}

/// Internal function for building schema properties with depth control
fn build_schema_properties_with_depth(
    method: &MethodDef,
    module: &Module,
    loader: Option<&ZmlModuleLoader>,
    depth: usize,
) -> (Map<String, Value>, Vec<String>) {
    // Prevent infinite recursion by checking depth
    if depth > MAX_NESTING_DEPTH {
        let mut simple_schema = Map::new();
        simple_schema.insert("type".to_string(), Value::String("object".to_string()));
        simple_schema.insert("description".to_string(), Value::String("Nested object (depth limit reached)".to_string()));
        return (simple_schema, Vec::new());
    }

    let mut properties = Map::new();
    let mut required = Vec::new();

    for (param_name, param_def) in &method.params {
        let mut param_schema = build_type_schema(&param_def.type_expr, module, loader, depth + 1);

        // Attach description and default if present
        if let Some(desc) = &param_def.description {
            if let Some(obj) = param_schema.as_object_mut() {
                obj.insert("description".to_string(), Value::String(desc.clone()));
            }
        }
        if let Some(default) = &param_def.default_value {
            if let Some(obj) = param_schema.as_object_mut() {
                obj.insert("default".to_string(), zml_value_to_json(default));
            }
        }

        properties.insert(param_name.clone(), param_schema);

        if !param_def.optional {
            required.push(param_name.clone());
        }
    }

    (properties, required)
}
/// Convert ZML Value to serde_json::Value
fn zml_value_to_json(v: &ZmlValue) -> Value {
    match v {
        ZmlValue::String(s) => Value::String(s.clone()),
        ZmlValue::Integer(i) => Value::from(*i),
        ZmlValue::Number(n) => Value::Number(serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0))),
        ZmlValue::Boolean(b) => Value::from(*b),
        ZmlValue::Array(arr) => Value::Array(arr.iter().map(zml_value_to_json).collect()),
        ZmlValue::Object(map) => {
            let mut m = Map::new();
            for (k, v) in map.iter() {
                m.insert(k.clone(), zml_value_to_json(v));
            }
            Value::Object(m)
        }
        ZmlValue::Null => Value::Null,
    }
}

/// Resolve a named type reference to TypeDef or EnumDef
fn resolve_named<'a>(
    name: &str,
    current_module: &'a Module,
    loader: Option<&'a ZmlModuleLoader>,
) -> (Option<&'a TypeDef>, Option<&'a EnumDef>) {
    // Support qualified name: moduleName.typeName
    let (module_to_search, type_name) = if let Some(idx) = name.find('.') {
        let (mod_name, type_name) = name.split_at(idx);
        let type_name = &type_name[1..];
        let module_ref = loader.and_then(|l| l.get_module(mod_name));
        (module_ref.unwrap_or(current_module), type_name)
    } else {
        (current_module, name)
    };

    let t = module_to_search.types.get(type_name);
    let e = module_to_search.enums.get(type_name);
    (t, e)
}

/// Build JSON Schema from a ZML TypeExpr with depth control
fn build_type_schema(
    type_expr: &TypeExpr,
    module: &Module,
    loader: Option<&ZmlModuleLoader>,
    depth: usize,
) -> Value {
    if depth > MAX_NESTING_DEPTH {
        let mut simple = Map::new();
        simple.insert("type".to_string(), Value::String("object".to_string()));
        simple.insert(
            "description".to_string(),
            Value::String("Nested object (depth limit reached)".to_string()),
        );
        return Value::Object(simple);
    }

    match type_expr {
        TypeExpr::String => json_type("string"),
        TypeExpr::Integer => json_type("integer"),
        TypeExpr::Number => json_type("number"),
        TypeExpr::Boolean => json_type("boolean"),
        TypeExpr::Date => json_string_with_format("date"),
        TypeExpr::DateTime => json_string_with_format("date-time"),
        TypeExpr::Any => Value::Object(Map::new()),
        TypeExpr::Array(item) => {
            let mut m = Map::new();
            m.insert("type".to_string(), Value::String("array".to_string()));
            m.insert(
                "items".to_string(),
                build_type_schema(item, module, loader, depth + 1),
            );
            Value::Object(m)
        }
        TypeExpr::Object(fields) => build_object_schema(fields, module, loader, depth + 1),
        TypeExpr::Enum(values) => {
            let mut m = Map::new();
            m.insert("type".to_string(), Value::String("string".to_string()));
            m.insert(
                "enum".to_string(),
                Value::Array(values.iter().map(|s| Value::String(s.clone())).collect()),
            );
            Value::Object(m)
        }
        TypeExpr::Ref(name) | TypeExpr::Alias(name) => {
            let (type_def, enum_def) = resolve_named(name, module, loader);
            if let Some(td) = type_def {
                build_typedef_schema(td, module, loader, depth + 1)
            } else if let Some(ed) = enum_def {
                build_enumdef_schema(ed)
            } else {
                // Fallback to string type if unresolved
                json_type("string")
            }
        }
    }
}

fn json_type(t: &str) -> Value {
    let mut m = Map::new();
    m.insert("type".to_string(), Value::String(t.to_string()));
    Value::Object(m)
}

fn json_string_with_format(fmt: &str) -> Value {
    let mut m = Map::new();
    m.insert("type".to_string(), Value::String("string".to_string()));
    m.insert("format".to_string(), Value::String(fmt.to_string()));
    Value::Object(m)
}

fn build_object_schema(
    fields: &HashMap<String, FieldDef>,
    module: &Module,
    loader: Option<&ZmlModuleLoader>,
    depth: usize,
) -> Value {
    let mut props = Map::new();
    let mut req: Vec<String> = Vec::new();

    for (name, field) in fields.iter() {
        let mut field_schema = build_type_schema(&field.type_expr, module, loader, depth + 1);

        // Attach description and default
        if let Some(desc) = &field.description {
            if let Some(obj) = field_schema.as_object_mut() {
                obj.insert("description".to_string(), Value::String(desc.clone()));
            }
        }
        if let Some(default) = &field.default_value {
            if let Some(obj) = field_schema.as_object_mut() {
                obj.insert("default".to_string(), zml_value_to_json(default));
            }
        }

        props.insert(name.clone(), field_schema);
        if !field.optional {
            req.push(name.clone());
        }
    }

    let mut m = Map::new();
    m.insert("type".to_string(), Value::String("object".to_string()));
    m.insert("properties".to_string(), Value::Object(props));
    // Disallow extra fields inside nested objects
    m.insert("additionalProperties".to_string(), Value::Bool(false));
    if !req.is_empty() {
        m.insert(
            "required".to_string(),
            Value::Array(req.into_iter().map(Value::String).collect()),
        );
    }
    Value::Object(m)
}

fn build_typedef_schema(
    td: &TypeDef,
    module: &Module,
    loader: Option<&ZmlModuleLoader>,
    depth: usize,
) -> Value {
    build_object_schema(&td.fields, module, loader, depth + 1)
}

fn build_enumdef_schema(ed: &EnumDef) -> Value {
    // Collect enum values, using explicit value when present; otherwise use name (string)
    let mut enum_values: Vec<Value> = Vec::new();
    let mut type_set: HashSet<&'static str> = HashSet::new();

    for (_name, ev) in ed.values.iter() {
        if let Some(v) = &ev.value {
            let json = zml_value_to_json(v);
            // Track type without moving from `json`
            match &json {
                Value::String(_) => { type_set.insert("string"); }
                Value::Number(n) => {
                    // Distinguish integer vs number
                    if n.is_i64() || n.is_u64() {
                        type_set.insert("integer");
                    } else {
                        type_set.insert("number");
                    }
                }
                Value::Bool(_) => { type_set.insert("boolean"); }
                Value::Array(_) => { type_set.insert("array"); }
                Value::Object(_) => { type_set.insert("object"); }
                Value::Null => { type_set.insert("null"); }
            }
            enum_values.push(json);
        } else {
            enum_values.push(Value::String(ev.name.clone()));
            type_set.insert("string");
        }
    }

    let mut m = Map::new();
    if type_set.len() == 1 {
        let t = type_set.iter().next().unwrap();
        // Prefer integer if all values are integral numbers (not reliably detectable here)
        m.insert("type".to_string(), Value::String((*t).to_string()));
    }
    m.insert("enum".to_string(), Value::Array(enum_values));
    Value::Object(m)
}

/// Build output schema for tool based on ZML method response
/// For dynamic modules, all sub-objects are directly expanded without references
pub fn build_output_schema_zml(method: &MethodDef, module: &Module, loader: Option<&ZmlModuleLoader>) -> Value {
    // Build the core type schema
    let mut schema = build_type_schema(&method.response, module, loader, 0);

    // Attach JSON Schema dialect and optional description
    if let Some(obj) = schema.as_object_mut() {
        obj.insert(
            "$schema".to_string(),
            Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
        );

        if let Some(desc) = &method.description {
            obj.insert("description".to_string(), Value::String(desc.clone()));
        }

        // If this is an object schema, also disallow unknown properties
        let is_object = obj
            .get("type")
            .and_then(|v| v.as_str())
            .map(|t| t == "object")
            .unwrap_or(false)
            || obj.contains_key("properties");

        if is_object {
            obj.insert("additionalProperties".to_string(), Value::Bool(false));
        }
    }

    // Provide JSON-RPC 2.0 compliant response envelope as a definition
    let mut defs = Map::new();

    // jsonrpc const "2.0"
    let jsonrpc_const = {
        let mut jr = Map::new();
        jr.insert("type".to_string(), Value::String("string".to_string()));
        jr.insert("const".to_string(), Value::String("2.0".to_string()));
        Value::Object(jr)
    };

    // id type union
    let id_type = {
        let mut id = Map::new();
        id.insert(
            "type".to_string(),
            Value::Array(vec![
                Value::String("string".to_string()),
                Value::String("integer".to_string()),
                Value::String("number".to_string()),
                Value::String("null".to_string()),
            ]),
        );
        Value::Object(id)
    };

    // Success response
    let mut success_props = Map::new();
    success_props.insert("jsonrpc".to_string(), jsonrpc_const.clone());
    success_props.insert("id".to_string(), id_type.clone());
    success_props.insert("result".to_string(), schema.clone());
    let mut success_obj = Map::new();
    success_obj.insert("type".to_string(), Value::String("object".to_string()));
    success_obj.insert("properties".to_string(), Value::Object(success_props));
    success_obj.insert("required".to_string(), Value::Array(vec![
        Value::String("jsonrpc".to_string()),
        Value::String("id".to_string()),
        Value::String("result".to_string()),
    ]));
    success_obj.insert("additionalProperties".to_string(), Value::Bool(false));

    // Error response
    let mut error_obj_schema = Map::new();
    let mut error_props = Map::new();
    error_props.insert("code".to_string(), json_type("integer"));
    error_props.insert("message".to_string(), json_type("string"));
    // data can be any
    error_props.insert("data".to_string(), Value::Object(Map::new()));
    error_obj_schema.insert("type".to_string(), Value::String("object".to_string()));
    error_obj_schema.insert("properties".to_string(), Value::Object(error_props));
    error_obj_schema.insert("required".to_string(), Value::Array(vec![
        Value::String("code".to_string()),
        Value::String("message".to_string()),
    ]));
    error_obj_schema.insert("additionalProperties".to_string(), Value::Bool(false));

    let mut error_props_envelope = Map::new();
    error_props_envelope.insert("jsonrpc".to_string(), jsonrpc_const);
    error_props_envelope.insert("id".to_string(), id_type);
    error_props_envelope.insert("error".to_string(), Value::Object(error_obj_schema));
    let mut error_envelope = Map::new();
    error_envelope.insert("type".to_string(), Value::String("object".to_string()));
    error_envelope.insert("properties".to_string(), Value::Object(error_props_envelope));
    error_envelope.insert("required".to_string(), Value::Array(vec![
        Value::String("jsonrpc".to_string()),
        Value::String("id".to_string()),
        Value::String("error".to_string()),
    ]));
    error_envelope.insert("additionalProperties".to_string(), Value::Bool(false));

    let mut rpc_response = Map::new();
    rpc_response.insert("oneOf".to_string(), Value::Array(vec![
        Value::Object(success_obj),
        Value::Object(error_envelope),
    ]));

    defs.insert("JsonRpcResponse".to_string(), Value::Object(rpc_response));

    if let Some(obj) = schema.as_object_mut() {
        obj.insert("$defs".to_string(), Value::Object(defs));
    }

    schema
}