//! Response validation for dynamic module service

use crate::config::zml_loader::ZmlModuleLoader;
use crate::zml::ast::{EnumDef as ZmlEnumDef, MethodDef as ZmlMethodDef, Module as ZmlModule, TypeDef as ZmlTypeDef, TypeExpr as ZmlTypeExpr, Value as ZmlValue};
use rmcp::ErrorData as McpError;
use serde_json::Value;

/// ===================== ZML Support =====================
/// Validate response against ZML method response type
pub fn validate_response_zml(
    response: &Value,
    method: &ZmlMethodDef,
    module: &ZmlModule,
    loader: Option<&ZmlModuleLoader>,
) -> Result<(), McpError> {
    validate_value_against_typeexpr(response, &method.response, module, loader)
}

/// Validate a JSON value against a ZML TypeExpr (recursive)
fn validate_value_against_typeexpr(
    value: &Value,
    type_expr: &ZmlTypeExpr,
    module: &ZmlModule,
    loader: Option<&ZmlModuleLoader>,
) -> Result<(), McpError> {
    match type_expr {
        ZmlTypeExpr::String => {
            if !value.is_string() { return Err(McpError::internal_error("Response must be string", None)); }
        }
        ZmlTypeExpr::Integer => {
            if !(value.is_number() && (value.as_i64().is_some() || value.as_u64().is_some())) {
                return Err(McpError::internal_error("Response must be integer", None));
            }
        }
        ZmlTypeExpr::Number => {
            if !value.is_number() { return Err(McpError::internal_error("Response must be number", None)); }
        }
        ZmlTypeExpr::Boolean => {
            if !value.is_boolean() { return Err(McpError::internal_error("Response must be boolean", None)); }
        }
        ZmlTypeExpr::Date | ZmlTypeExpr::DateTime => {
            if !value.is_string() { return Err(McpError::internal_error("Response must be string date/datetime", None)); }
        }
        ZmlTypeExpr::Any => { /* always valid */ }
        ZmlTypeExpr::Array(item) => {
            if !value.is_array() { return Err(McpError::internal_error("Response must be array", None)); }
            for v in value.as_array().unwrap() {
                validate_value_against_typeexpr(v, item, module, loader)?;
            }
        }
        ZmlTypeExpr::Object(fields) => {
            if !value.is_object() { return Err(McpError::internal_error("Response must be object", None)); }
            let obj = value.as_object().unwrap();
            for (fname, fdef) in fields.iter() {
                if !fdef.optional && !obj.contains_key(fname) {
                    return Err(McpError::internal_error(format!("Missing required field: {}", fname), None));
                }
                if let Some(v) = obj.get(fname) {
                    validate_value_against_typeexpr(v, &fdef.type_expr, module, loader)?;
                }
            }
        }
        ZmlTypeExpr::Enum(values) => {
            if let Some(s) = value.as_str() {
                if !values.iter().any(|v| v == s) {
                    return Err(McpError::internal_error(format!("Response value '{}' not in enum", s), None));
                }
            } else {
                return Err(McpError::internal_error("Enum response must be string", None));
            }
        }
        ZmlTypeExpr::Ref(name) | ZmlTypeExpr::Alias(name) => {
            let (tdef, edef) = resolve_named(name, module, loader);
            if let Some(td) = tdef {
                // Treat typedef as object
                let as_object = ZmlTypeExpr::Object(td.fields.clone());
                validate_value_against_typeexpr(value, &as_object, module, loader)?;
            } else if let Some(ed) = edef {
                validate_enumdef(value, ed)?;
            } else {
                // Unknown reference; fallback to string
                if !value.is_string() {
                    return Err(McpError::internal_error("Response must be string (unresolved ref)", None));
                }
            }
        }
    }
    Ok(())
}

/// Resolve a named type/enum, supporting qualified module reference: Module.Type
fn resolve_named<'a>(
    name: &str,
    current_module: &'a ZmlModule,
    loader: Option<&'a ZmlModuleLoader>,
) -> (Option<&'a ZmlTypeDef>, Option<&'a ZmlEnumDef>) {
    if let Some(idx) = name.find('.') {
        let (mod_name, type_name) = name.split_at(idx);
        let type_name = &type_name[1..];
        let module_ref = loader.and_then(|l| l.get_module(mod_name));
        let module_to_search = module_ref.unwrap_or(current_module);
        (module_to_search.types.get(type_name), module_to_search.enums.get(type_name))
    } else {
        (current_module.types.get(name), current_module.enums.get(name))
    }
}

/// Validate value against an EnumDef (supports typed enum values)
fn validate_enumdef(value: &Value, ed: &ZmlEnumDef) -> Result<(), McpError> {
    // Accept either explicit typed values or the enum key names as strings
    for (_name, ev) in ed.values.iter() {
        if let Some(v) = &ev.value {
            let expected = zml_value_to_json(v);
            if &expected == value { return Ok(()); }
        } else if let Some(s) = value.as_str() {
            if s == ev.name { return Ok(()); }
        }
    }
    Err(McpError::internal_error("Response value not found in enum", None))
}

/// Convert ZML Value to serde_json::Value (local copy)
fn zml_value_to_json(v: &ZmlValue) -> Value {
    match v {
        ZmlValue::String(s) => Value::String(s.clone()),
        ZmlValue::Integer(i) => Value::from(*i),
        ZmlValue::Number(n) => Value::Number(serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0))),
        ZmlValue::Boolean(b) => Value::from(*b),
        ZmlValue::Array(arr) => Value::Array(arr.iter().map(zml_value_to_json).collect()),
        ZmlValue::Object(map) => {
            let mut m = serde_json::Map::new();
            for (k, v) in map.iter() { m.insert(k.clone(), zml_value_to_json(v)); }
            Value::Object(m)
        }
        ZmlValue::Null => Value::Null,
    }
}