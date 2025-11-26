//! ZML Abstract Syntax Tree (AST) Definition

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ZML Module Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub extends: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub access_level: Option<AccessLevel>,
    pub category: Option<String>,
    pub types: HashMap<String, TypeDef>,
    pub enums: HashMap<String, EnumDef>,
    pub methods: HashMap<String, MethodDef>,
    pub resources: HashMap<String, ResourceDef>,
    pub templates: HashMap<String, TemplateDef>,
}

/// Access Level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessLevel {
    Public,
    Private,
    Internal,
}

/// Type Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
    pub fields: HashMap<String, FieldDef>,
    pub description: Option<String>,
}

/// Enum Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDef {
    pub name: String,
    pub values: HashMap<String, EnumValueDef>,
    pub description: Option<String>,
}

/// Enum Value Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumValueDef {
    pub name: String,
    pub value: Option<Value>,
    pub description: Option<String>,
}

/// Field Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub type_expr: TypeExpr,
    pub optional: bool,
    pub default_value: Option<Value>,
    pub description: Option<String>,
}

/// Type Expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeExpr {
    /// Basic types
    String,
    Integer,
    Number,
    Boolean,
    Date,
    DateTime,
    Any,
    
    /// Array type
    Array(Box<TypeExpr>),
    
    /// Object type
    Object(HashMap<String, FieldDef>),
    
    /// Enum Type
    Enum(Vec<String>),
    
    /// Reference Type
    Ref(String),
    
    /// Type Alias
    Alias(String),
}

/// Method Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDef {
    pub name: String,
    pub description: Option<String>,
    pub http_method: HttpMethod,
    pub uri: String,
    pub access_level: AccessLevel,
    pub rate_limit: Option<RateLimit>,
    pub params: HashMap<String, ParamDef>,
    pub response: TypeExpr,
}

/// HTTP Method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// Rate Limit Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests: u32,
    pub per_seconds: u32,
}

/// Parameter Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    pub name: String,
    pub type_expr: TypeExpr,
    pub optional: bool,
    pub default_value: Option<Value>,
    pub description: Option<String>,
}

/// Resource Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    pub name: String,
    pub resource_type: ResourceType,
    pub uri: String,
    pub description: Option<String>,
}

/// Resource Type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Collection,
    Entity,
}

/// Template Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDef {
    pub name: String,
    pub content: HashMap<String, Value>,
}

/// Value Type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}

/// Location Information (for error reporting)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

/// AST Node with Location Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Node<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

// Implement convenience methods
impl TypeExpr {
    /// Check if type is basic
    pub fn is_basic(&self) -> bool {
        matches!(
            self,
            TypeExpr::String
                | TypeExpr::Integer
                | TypeExpr::Number
                | TypeExpr::Boolean
                | TypeExpr::Date
                | TypeExpr::DateTime
                | TypeExpr::Any
        )
    }
    
    /// Check if type is composite
    pub fn is_composite(&self) -> bool {
        matches!(self, TypeExpr::Array(_) | TypeExpr::Object(_) | TypeExpr::Enum(_))
    }
    
    /// Check if type is reference
    pub fn is_reference(&self) -> bool {
        matches!(self, TypeExpr::Ref(_) | TypeExpr::Alias(_))
    }

    /// Convert type expression to a concise string representation
    pub fn to_string_repr(&self) -> String {
        match self {
            TypeExpr::String => "string".to_string(),
            TypeExpr::Integer => "integer".to_string(),
            TypeExpr::Number => "number".to_string(),
            TypeExpr::Boolean => "boolean".to_string(),
            TypeExpr::Date => "date".to_string(),
            TypeExpr::DateTime => "datetime".to_string(),
            TypeExpr::Any => "any".to_string(),
            TypeExpr::Array(inner) => format!("array<{}>", inner.to_string_repr()),
            TypeExpr::Object(_fields) => {
                // Keep object representation concise for template usage
                // Detailed field representation can be added if necessary
                "object".to_string()
            }
            TypeExpr::Enum(values) => format!("enum[{}]", values.join(", ")),
            TypeExpr::Ref(name) => format!("ref:{}", name),
            TypeExpr::Alias(name) => format!("alias:{}", name),
        }
    }
}

impl Value {
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Integer(i) => i.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                let pairs: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Value::Null => "null".to_string(),
        }
    }
}

// Implement From trait for common types
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Number(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(vec: Vec<T>) -> Self {
        Value::Array(vec.into_iter().map(|v| v.into()).collect())
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(map: HashMap<String, Value>) -> Self {
        Value::Object(map)
    }
}