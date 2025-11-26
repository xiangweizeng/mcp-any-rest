//! Parameter validation for dynamic module service

use crate::config::zml_loader::ZmlModuleLoader;
use crate::zml::ast::{MethodDef as ZmlMethodDef, Module as ZmlModule, TypeExpr as ZmlTypeExpr, TypeDef as ZmlTypeDef, Value as ZmlValue};
use log::debug;
use rmcp::ErrorData as McpError;
use serde_json::Value;
use std::collections::HashMap;

/// ===================== ZML Support =====================
/// Validate and normalize parameters against ZML MethodDef
pub fn validate_parameters_zml(
    params: &HashMap<String, Value>,
    module: &ZmlModule,
    method: &ZmlMethodDef,
    loader: Option<&ZmlModuleLoader>,
) -> Result<HashMap<String, Value>, McpError> {
    debug!("Validating ZML parameters: {:?}", params);
    let mut normalized = params.clone();

    for (name, def) in &method.params {
        // Required check and default application
        if !def.optional && !normalized.contains_key(name) {
            if let Some(default) = &def.default_value {
                normalized.insert(name.clone(), zml_value_to_json(default));
            } else {
                return Err(McpError::invalid_params(format!("Missing required parameter: {}", name), None));
            }
        }

        if let Some(value) = normalized.get(name) {
            let converted = maybe_convert_basic(value, &def.type_expr)?;
            validate_value_against_typeexpr(&converted, &def.type_expr, module, loader)?;
            // Save potentially converted value back
            normalized.insert(name.clone(), converted);
        }
    }

    Ok(normalized)
}

/// Attempt basic normalization: parse strings to integer/number/boolean
fn maybe_convert_basic(value: &Value, t: &ZmlTypeExpr) -> Result<Value, McpError> {
    match t {
        ZmlTypeExpr::Integer => {
            if value.is_number() && (value.as_i64().is_some() || value.as_u64().is_some()) {
                Ok(value.clone())
            } else if let Some(s) = value.as_str() {
                match s.parse::<i64>() {
                    Ok(i) => Ok(Value::Number(serde_json::Number::from(i))),
                    Err(_) => Err(McpError::invalid_params("Parameter must be integer", None)),
                }
            } else {
                Err(McpError::invalid_params("Parameter must be integer", None))
            }
        }
        ZmlTypeExpr::Number => {
            if value.is_number() { Ok(value.clone()) }
            else if let Some(s) = value.as_str() {
                match s.parse::<f64>() {
                    Ok(f) => serde_json::Number::from_f64(f)
                        .map(Value::Number)
                        .ok_or_else(|| McpError::invalid_params("Invalid number value", None)),
                    Err(_) => Err(McpError::invalid_params("Parameter must be number", None)),
                }
            } else { Err(McpError::invalid_params("Parameter must be number", None)) }
        }
        ZmlTypeExpr::Boolean => {
            if value.is_boolean() { Ok(value.clone()) }
            else if let Some(s) = value.as_str() {
                match s.to_lowercase().as_str() {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    _ => Err(McpError::invalid_params("Parameter must be boolean", None)),
                }
            } else { Err(McpError::invalid_params("Parameter must be boolean", None)) }
        }
        _ => Ok(value.clone()),
    }
}

/// Recursive validation of a JSON value against ZML TypeExpr (parameters)
fn validate_value_against_typeexpr(
    value: &Value,
    type_expr: &ZmlTypeExpr,
    module: &ZmlModule,
    loader: Option<&ZmlModuleLoader>,
) -> Result<(), McpError> {
    match type_expr {
        ZmlTypeExpr::String => { if !value.is_string() { return Err(McpError::invalid_params("Parameter must be string", None)); } }
        ZmlTypeExpr::Integer => { if !(value.is_number() && (value.as_i64().is_some() || value.as_u64().is_some())) { return Err(McpError::invalid_params("Parameter must be integer", None)); } }
        ZmlTypeExpr::Number => { if !value.is_number() { return Err(McpError::invalid_params("Parameter must be number", None)); } }
        ZmlTypeExpr::Boolean => { if !value.is_boolean() { return Err(McpError::invalid_params("Parameter must be boolean", None)); } }
        ZmlTypeExpr::Date | ZmlTypeExpr::DateTime => { if !value.is_string() { return Err(McpError::invalid_params("Parameter must be string date/datetime", None)); } }
        ZmlTypeExpr::Any => {}
        ZmlTypeExpr::Array(item) => {
            if !value.is_array() { return Err(McpError::invalid_params("Parameter must be array", None)); }
            for v in value.as_array().unwrap() { validate_value_against_typeexpr(v, item, module, loader)?; }
        }
        ZmlTypeExpr::Object(fields) => {
            if !value.is_object() { return Err(McpError::invalid_params("Parameter must be object", None)); }
            let obj = value.as_object().unwrap();
            for (fname, fdef) in fields.iter() {
                if !fdef.optional && !obj.contains_key(fname) { return Err(McpError::invalid_params(format!("Missing required field: {}", fname), None)); }
                if let Some(v) = obj.get(fname) { validate_value_against_typeexpr(v, &fdef.type_expr, module, loader)?; }
            }
        }
        ZmlTypeExpr::Enum(values) => {
            if let Some(s) = value.as_str() {
                if !values.iter().any(|v| v == s) { return Err(McpError::invalid_params(format!("Parameter value '{}' not in enum", s), None)); }
            } else { return Err(McpError::invalid_params("Enum parameter must be string", None)); }
        }
        ZmlTypeExpr::Ref(name) | ZmlTypeExpr::Alias(name) => {
            let (tdef, edef) = resolve_named(name, module, loader);
            if let Some(td) = tdef {
                let as_object = ZmlTypeExpr::Object(td.fields.clone());
                validate_value_against_typeexpr(value, &as_object, module, loader)?;
            } else if let Some(ed) = edef {
                validate_enumdef(value, ed)?;
            } else {
                // Unknown reference; accept string
                if !value.is_string() { return Err(McpError::invalid_params("Parameter must be string (unresolved ref)", None)); }
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
        (
            module_to_search.types.get(type_name),
            module_to_search.enums.get(type_name),
        )
    } else {
        (
            current_module.types.get(name),
            current_module.enums.get(name),
        )
    }
}

use crate::zml::ast::EnumDef as ZmlEnumDef; // local alias for resolve

fn validate_enumdef(value: &Value, ed: &ZmlEnumDef) -> Result<(), McpError> {
    for (_name, ev) in ed.values.iter() {
        if let Some(v) = &ev.value {
            let expected = zml_value_to_json(v);
            if &expected == value { return Ok(()); }
        } else if let Some(s) = value.as_str() {
            if s == ev.name { return Ok(()); }
        }
    }
    Err(McpError::invalid_params("Parameter value not found in enum", None))
}

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