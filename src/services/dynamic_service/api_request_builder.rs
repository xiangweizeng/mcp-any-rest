//! API request builder for dynamic module service

use crate::zml::ast::{MethodDef as ZmlMethodDef, Module as ZmlModule, HttpMethod as ZmlHttpMethod};
use anyhow::Result;
use log::debug;
use reqwest::Method;
use serde_json::Value;
use std::collections::HashMap;

/// ===================== ZML Support =====================
/// Build API request for ZML MethodDef
pub fn build_api_request_zml(
    params: &HashMap<String, Value>,
    _module: &ZmlModule,
    method: &ZmlMethodDef,
) -> Result<(String, Method, Option<Value>)> {
    debug!("Building API request for ZML method: {:?}", method);
    let http_method = determine_http_method_zml(&method.http_method);
    let endpoint = build_endpoint_zml(method, params)?;
    let request_body = build_request_body_for_method_zml(&http_method, params, method)?;

    debug!("ZML Request endpoint: {}", endpoint);
    debug!("ZML Request body: {:?}", request_body);
    Ok((endpoint, http_method, request_body))
}

/// Determine HTTP method from ZML HttpMethod enum
fn determine_http_method_zml(http_method: &ZmlHttpMethod) -> Method {
    match http_method {
        ZmlHttpMethod::Get => Method::GET,
        ZmlHttpMethod::Post => Method::POST,
        ZmlHttpMethod::Put => Method::PUT,
        ZmlHttpMethod::Delete => Method::DELETE,
        ZmlHttpMethod::Patch => Method::PATCH,
    }
}

/// Build endpoint for ZML method by replacing `{param}` placeholders
pub fn build_endpoint_zml(method: &ZmlMethodDef, params: &HashMap<String, Value>) -> Result<String> {
    let mut endpoint = method.uri.clone();
    debug!("ZML URI template: {}", endpoint);

    // Replace path parameters enclosed in {param}
    // Collect used path params to separate query/body later
    let mut used_path_params: Vec<String> = Vec::new();

    // Simple scan for `{name}` patterns
    let mut idx = 0usize;
    while let Some(start) = endpoint[idx..].find('{') {
        let real_start = idx + start;
        if let Some(end) = endpoint[real_start..].find('}') {
            let real_end = real_start + end;
            let key = endpoint[real_start + 1..real_end].to_string();
            used_path_params.push(key.clone());
            if let Some(value) = params.get(&key) {
                endpoint.replace_range(real_start..=real_end, &json_value_to_string(value));
                idx = real_start + json_value_to_string(value).len();
            } else {
                // Leave placeholder as-is if missing; validation will catch required params
                idx = real_end + 1;
            }
        } else {
            break;
        }
    }

    // Add query parameters to endpoint if any non-path params are left
    let has_non_path_params = params.keys().any(|k| !used_path_params.contains(k));
    if has_non_path_params {
        let query_params = params
            .iter()
            .filter(|(k, _)| !used_path_params.contains(k))
            .map(|(k, v)| format!("{}={}", k, json_value_to_string(v)))
            .collect::<Vec<_>>()
            .join("&");
        endpoint.push_str(&format!("?{}", query_params));
    }

    debug!("ZML formatted URI: {}", endpoint);
    Ok(endpoint)
}

/// Build request body for ZML based on HTTP method and params
fn build_request_body_for_method_zml(
    http_method: &Method,
    params: &HashMap<String, Value>,
    method: &ZmlMethodDef,
) -> Result<Option<Value>> {
    match *http_method {
        Method::POST | Method::PUT | Method::PATCH => {
            // Use non-path params as JSON body
            let body = build_request_body_zml(params, method)?;
            Ok(Some(body))
        }
        _ => Ok(None),
    }
}

/// Build JSON body for ZML method: include params not present in path
pub fn build_request_body_zml(
    params: &HashMap<String, Value>,
    method: &ZmlMethodDef,
) -> Result<Value> {
    let mut body = serde_json::Map::new();

    // Determine path params used in `uri`
    let mut path_params: HashMap<String, bool> = HashMap::new();
    let mut idx = 0usize;
    let template = &method.uri;
    while let Some(start) = template[idx..].find('{') {
        let real_start = idx + start;
        if let Some(end) = template[real_start..].find('}') {
            let real_end = real_start + end;
            let key = template[real_start + 1..real_end].to_string();
            path_params.insert(key, true);
            idx = real_end + 1;
        } else {
            break;
        }
    }

    for (name, _def) in &method.params {
        if path_params.get(name).copied().unwrap_or(false) {
            continue; // skip path params
        }
        if let Some(value) = params.get(name) {
            body.insert(name.clone(), value.clone());
        }
    }

    Ok(Value::Object(body))
}

/// Helper to convert serde_json::Value to string for path substitution
fn json_value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        // For arrays/objects, use compact JSON
        Value::Array(_) | Value::Object(_) => v.to_string(),
    }
}