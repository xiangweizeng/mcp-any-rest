//! ZML parser implementation

use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

use crate::zml::ast::*;

// Import Pest-generated parser
#[derive(Parser)]
#[grammar = "zml/grammar.pest"]
pub struct ZMLParser;

/// ZML parser error types
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Syntax error: {message} at line {line}, column {column}")]
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
    },
    #[error("Semantic error: {message}")]
    SemanticError { message: String },
    #[error("Type error: {message}")]
    TypeError { message: String },
    #[error("Reference error: {message}")]
    ReferenceError { message: String },
    #[error("IO error: {source}")]
    IoError { source: std::io::Error },
}

/// Parse context for tracking parsing state and error information
#[derive(Debug)]
struct ParseContext {
    current_line: usize,
    current_column: usize,
}

impl ParseContext {
    fn new(_source: &str) -> Self {
        Self {
            current_line: 1,
            current_column: 1,
        }
    }
    
    fn update_position(&mut self, pair: &pest::iterators::Pair<Rule>) {
        let (line, column) = pair.as_span().start_pos().line_col();
        self.current_line = line;
        self.current_column = column;
    }
    
    fn syntax_error(&self, message: String) -> ParseError {
        ParseError::SyntaxError {
            message,
            line: self.current_line,
            column: self.current_column,
        }
    }
    
    fn semantic_error(&self, message: String) -> ParseError {
        ParseError::SemanticError { message }
    }
    
    fn type_error(&self, message: String) -> ParseError {
        ParseError::TypeError { message }
    }
    
    fn reference_error(&self, message: String) -> ParseError {
        ParseError::ReferenceError { message }
    }
}

/// ZML parser wrapper
pub struct ZMLParserWrapper {
    modules: HashMap<String, Module>,
}

impl ZMLParserWrapper {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Parse ZML source code
    pub fn parse(&mut self, source: &str) -> Result<Module, ParseError> {
        let mut context = ParseContext::new(source);
        
        let pairs = ZMLParser::parse(Rule::file, source)
            .map_err(|e| self.convert_pest_error(e, source))?;

        let mut module = Module {
            name: String::new(),
            extends: None,
            version: None,
            description: None,
            enabled: None,
            access_level: None,
            category: None,
            types: HashMap::new(),
            enums: HashMap::new(),
            methods: HashMap::new(),
            resources: HashMap::new(),
            templates: HashMap::new(),
        };

        for pair in pairs {
            match pair.as_rule() {
                Rule::file => {
                    // file rule can now contain multiple module_def and template_def
                    for inner_pair in pair.into_inner() {
                        match inner_pair.as_rule() {
                            Rule::module_def => {
                                context.update_position(&inner_pair);
                                self.parse_module_def(inner_pair, &mut module, &context)?;
                            }
                            Rule::template_def => {
                                // Parse file-level template definition
                                let template_def = self.parse_template_def(inner_pair, &context)?;
                                module.templates.insert(template_def.name.clone(), template_def);
                            }
                            Rule::EOI => break,
                            Rule::WHITESPACE => continue,
                            _ => {
                                // Unknown file-level rule: ignored
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        // Validate module
        self.validate_module(&module, &context)?;

        // Cache module
        if !module.name.is_empty() {
            self.modules.insert(module.name.clone(), module.clone());
        }

        Ok(module)
    }

    /// Parse module definition
    fn parse_module_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        module: &mut Module,
        context: &ParseContext,
    ) -> Result<(), ParseError> {
        let mut inner_pairs = pair.into_inner();

        // Parse module name
        if let Some(name_pair) = inner_pairs.next() {
            module.name = name_pair.as_str().to_string();
        }

        // Parse inheritance relationship and other content
        for pair in inner_pairs {
            match pair.as_rule() {
                Rule::extends_clause => {
                    let mut extends_pairs = pair.into_inner();
                    if let Some(extends_name) = extends_pairs.next() {
                        module.extends = Some(extends_name.as_str().to_string());
                    }
                }
                Rule::module_content => {
                    self.parse_module_content(pair, module, context)?;
                }
                Rule::WHITESPACE => {
                    // Ignore whitespace characters
                    continue;
                }
                _ => {
                    // Debug output for unknown rules
                    println!("Unknown rule: {:?}", pair.as_rule());
                }
            }
        }

        Ok(())
    }

    /// Parse module content
    fn parse_module_content(
        &self,
        pair: pest::iterators::Pair<Rule>,
        module: &mut Module,
        context: &ParseContext,
    ) -> Result<(), ParseError> {
        let inner_pairs = pair.into_inner();

        // Process all content pairs
        for content_pair in inner_pairs {
            // Parsing module content entry
            match content_pair.as_rule() {
                Rule::type_def => {
                    let type_def = self.parse_type_def(content_pair, context)?;
                    module.types.insert(type_def.name.clone(), type_def);
                }
                Rule::enum_def => {
                    println!("Found enum definition, starting parsing...");
                    let enum_def = self.parse_enum_def(content_pair, context)?;
                    println!("Enum parsing completed: name={}, values_count={}", enum_def.name, enum_def.values.len());
                    module.enums.insert(enum_def.name.clone(), enum_def);
                }
                Rule::method_def => {
                    let method_def = self.parse_method_def(content_pair, context)?;
                    module.methods.insert(method_def.name.clone(), method_def);
                }
                Rule::resource_def => {
                    let resource_def = self.parse_resource_def(content_pair, context)?;
                    module
                        .resources
                        .insert(resource_def.name.clone(), resource_def);
                }
                Rule::template_def => {
                    let template_def = self.parse_template_def(content_pair, context)?;
                    module
                        .templates
                        .insert(template_def.name.clone(), template_def);
                }
                Rule::property_def => {
                    let (key, value) = self.parse_property_def(content_pair, context)?;
                    self.set_module_property(module, &key, value);
                }
                _ => {
                    // Unknown module content rule - ignored
                }
            }
        }

        Ok(())
    }

    /// Parse enum definition
    fn parse_enum_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<EnumDef, ParseError> {
        let mut inner_pairs = pair.into_inner();
        let mut enum_def = EnumDef {
            name: String::new(),
            values: HashMap::new(),
            description: None,
        };

        // Parse enum name
        if let Some(name_pair) = inner_pairs.next() {
            enum_def.name = name_pair.as_str().to_string();
        }

        // Parse enum values
        for pair in inner_pairs {
            match pair.as_rule() {
                Rule::enum_value_def => {
                    let enum_value_def = self.parse_enum_value_def(pair, context)?;
                    enum_def.values.insert(enum_value_def.name.clone(), enum_value_def);
                }
                _ => {
                    // Ignore other unknown rules (including comments, etc.)
                    continue;
                }
            }
        }

        Ok(enum_def)
    }

    /// Parse enum value definition
    fn parse_enum_value_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<EnumValueDef, ParseError> {
        let mut inner_pairs = pair.into_inner();
        let mut enum_value_def = EnumValueDef {
            name: String::new(),
            value: None,
            description: None,
        };

        // Parse enum value name
        if let Some(name_pair) = inner_pairs.next() {
            enum_value_def.name = name_pair.as_str().to_string();
        }

        // Parse enum value content and comments
        for pair in inner_pairs {
            match pair.as_rule() {
                Rule::value => {
                    enum_value_def.value = Some(self.parse_value(pair, context)?);
                }
                Rule::comment => {
                    // Parse comment as description
                    let comment = pair.as_str().trim();
                    if comment.starts_with("//") {
                        enum_value_def.description = Some(comment[2..].trim().to_string());
                    }
                }
                _ => {}
            }
        }

        Ok(enum_value_def)
    }

    /// Parse type definition
    fn parse_type_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<TypeDef, ParseError> {
        let mut inner_pairs = pair.into_inner();
        let mut type_def = TypeDef {
            name: String::new(),
            fields: HashMap::new(),
            description: None,
        };

        // Parse type name
        if let Some(name_pair) = inner_pairs.next() {
            type_def.name = name_pair.as_str().to_string();
        }

        // Parse fields
        for pair in inner_pairs {
            if pair.as_rule() == Rule::field_def {
                let field_def = self.parse_field_def(pair, context)?;
                type_def.fields.insert(field_def.name.clone(), field_def);
            }
        }

        Ok(type_def)
    }

    /// Parse field definition
    fn parse_field_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<FieldDef, ParseError> {
        let mut inner_pairs = pair.into_inner();
        let mut field_def = FieldDef {
            name: String::new(),
            type_expr: TypeExpr::Any,
            optional: false,
            default_value: None,
            description: None,
        };

        // Parse field name
        if let Some(name_pair) = inner_pairs.next() {
            field_def.name = name_pair.as_str().to_string();
        }

        // Parse optional marker, type expression and default value
        for pair in inner_pairs {
            match pair.as_rule() {
                Rule::optional_marker => {
                    field_def.optional = true;
                }
                Rule::type_expr => {
                    field_def.type_expr = self.parse_type_expr(pair, context)?;
                }
                Rule::default_value => {
                    let value_pairs = pair.into_inner();
                    for value_pair in value_pairs {
                        match value_pair.as_rule() {
                            Rule::value => {
                                field_def.default_value = Some(self.parse_value(value_pair, context)?);
                            }
                            Rule::comment => {
                                // Comments in default values are ignored but parsing won't fail
                    // If needed, comment processing logic can be added here
                            }
                            _ => {}
                        }
                    }
                }
                Rule::field_comment => {
                    // Parse comment as description
                    let comment = pair.as_str().trim();
                    if comment.starts_with("//") {
                        field_def.description = Some(comment[2..].trim().to_string());
                    }
                }
                _ => {}
            }
        }

        Ok(field_def)
    }

    /// Parse type expression
    fn parse_type_expr(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<TypeExpr, ParseError> {
        match pair.as_rule() {
            Rule::type_expr => {
                // Recursively parse sub-rules inside type_expr
                let mut inner_pairs = pair.into_inner();
                if let Some(inner_pair) = inner_pairs.next() {
                    self.parse_type_expr(inner_pair, context)
                } else {
                    Err(context.type_error("Empty type expression".to_string()))
                }
            }
            Rule::basic_type => match pair.as_str() {
                "string" => Ok(TypeExpr::String),
                "integer" => Ok(TypeExpr::Integer),
                "number" => Ok(TypeExpr::Number),
                "boolean" => Ok(TypeExpr::Boolean),
                "date" => Ok(TypeExpr::Date),
                "datetime" => Ok(TypeExpr::DateTime),
                "any" => Ok(TypeExpr::Any),
                _ => Err(context.type_error(format!("Unknown basic type: {}", pair.as_str()))),
            },
            Rule::array_type => {
                let mut inner_pairs = pair.into_inner();
                if let Some(element_type_pair) = inner_pairs.next() {
                    let element_type = self.parse_type_expr(element_type_pair, context)?;
                    Ok(TypeExpr::Array(Box::new(element_type)))
                } else {
                    Err(context.syntax_error("Array type missing element type".to_string()))
                }
            }
            Rule::object_type => {
                let mut fields = HashMap::new();
                for field_pair in pair.into_inner() {
                    if field_pair.as_rule() == Rule::field_def {
                        let field_def = self.parse_field_def(field_pair, context)?;
                        fields.insert(field_def.name.clone(), field_def);
                    }
                }
                Ok(TypeExpr::Object(fields))
            }
            Rule::enum_type => {
                let mut values = Vec::new();
                for value_pair in pair.into_inner() {
                    if value_pair.as_rule() == Rule::enum_values {
                        for enum_value_pair in value_pair.into_inner() {
                            if enum_value_pair.as_rule() == Rule::value {
                                let value = self.parse_value(enum_value_pair, context)?;
                                if let Value::String(s) = value {
                                    values.push(s);
                                }
                            }
                        }
                    }
                }
                Ok(TypeExpr::Enum(values))
            }
            Rule::ref_type => {
                let mut inner_pairs = pair.into_inner();
                if let Some(type_name_pair) = inner_pairs.next() {
                    if type_name_pair.as_rule() == Rule::identifier {
                        Ok(TypeExpr::Ref(type_name_pair.as_str().to_string()))
                    } else {
                        Err(context.syntax_error("Reference type missing type name".to_string()))
                    }
                } else {
                    Err(context.syntax_error("Reference type missing type name".to_string()))
                }
            }
            Rule::identifier => Ok(TypeExpr::Alias(pair.as_str().to_string())),
            _ => Err(context.type_error(format!("Unsupported type expression rule: {:?}", pair.as_rule()))),
        }
    }

    /// Parse value
    fn parse_value(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<Value, ParseError> {
        match pair.as_rule() {
            Rule::value => {
                // Recursively parse sub-rules inside value
                let mut inner_pairs = pair.into_inner();
                if let Some(inner_pair) = inner_pairs.next() {
                    self.parse_value(inner_pair, context)
                } else {
                    Err(context.type_error("Empty value".to_string()))
                }
            }
            Rule::string => {
                let s = pair.as_str();
                // Remove quotes
                Ok(Value::String(s[1..s.len() - 1].to_string()))
            }
            Rule::integer => {
                let num_str = pair.as_str();
                Ok(Value::Integer(num_str.parse().map_err(|e| {
                    context.type_error(format!("Cannot parse integer: {}", e))
                })?))
            }
            Rule::number => {
                let num_str = pair.as_str();
                if num_str.contains('.') {
                    Ok(Value::Number(num_str.parse().map_err(|e| {
                        context.type_error(format!("Cannot parse number: {}", e))
                    })?))
                } else {
                    Ok(Value::Integer(num_str.parse().map_err(|e| {
                        context.type_error(format!("Cannot parse integer: {}", e))
                    })?))
                }
            }
            Rule::boolean => match pair.as_str() {
                "true" => Ok(Value::Boolean(true)),
                "false" => Ok(Value::Boolean(false)),
                _ => Err(context.type_error(format!("Invalid boolean value: {}", pair.as_str()))),
            },
            Rule::identifier => Ok(Value::String(pair.as_str().to_string())),
            Rule::enum_reference => Ok(Value::String(pair.as_str().to_string())),
            _ => Err(context.type_error(format!("Unsupported value type: {:?}", pair.as_rule()))),
        }
    }

    /// Parse method definition
    fn parse_method_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<MethodDef, ParseError> {
        let mut inner_pairs = pair.into_inner();
        let mut method_def = MethodDef {
            name: String::new(),
            description: None,
            http_method: HttpMethod::Get,
            uri: String::new(),
            access_level: AccessLevel::Public,
            rate_limit: None,
            params: HashMap::new(),
            response: TypeExpr::Any,
        };

        // Parse method name
        if let Some(name_pair) = inner_pairs.next() {
            method_def.name = name_pair.as_str().to_string();
        }

        // Parse method content
        for pair in inner_pairs {
            if pair.as_rule() == Rule::method_content {
                self.parse_method_content(pair, &mut method_def, context)?;
            }
        }

        Ok(method_def)
    }

    /// Parse method content (refactored method)
    fn parse_method_content(
        &self,
        pair: pest::iterators::Pair<Rule>,
        method_def: &mut MethodDef,
        context: &ParseContext,
    ) -> Result<(), ParseError> {
        let mut content_pairs = pair.into_inner();
        
        if let Some(content_pair) = content_pairs.next() {
            match content_pair.as_rule() {
                Rule::description_def => {
                    method_def.description = self.parse_string_content(content_pair)?;
                }
                Rule::http_method_def => {
                    method_def.http_method = self.parse_http_method(content_pair)?;
                }
                Rule::uri_def => {
                    method_def.uri = self.parse_string_content(content_pair)?.unwrap_or_default();
                }
                Rule::access_level_def => {
                    method_def.access_level = self.parse_access_level(content_pair)?;
                }
                Rule::rate_limit_def => {
                    method_def.rate_limit = self.parse_rate_limit(content_pair, context)?;
                }
                Rule::params_def => {
                    self.parse_params_def(content_pair, method_def, context)?;
                }
                Rule::response_def => {
                    method_def.response = self.parse_response_def(content_pair, context)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Parse string content (generic method)
    fn parse_string_content(&self, pair: pest::iterators::Pair<Rule>) -> Result<Option<String>, ParseError> {
        let mut inner_pairs = pair.into_inner();
        if let Some(string_pair) = inner_pairs.next() {
            if string_pair.as_rule() == Rule::string {
                let content = string_pair.as_str();
                return Ok(Some(content[1..content.len() - 1].to_string()));
            }
        }
        Ok(None)
    }

    /// Parse HTTP method
    fn parse_http_method(&self, pair: pest::iterators::Pair<Rule>) -> Result<HttpMethod, ParseError> {
        let mut inner_pairs = pair.into_inner();
        if let Some(method_pair) = inner_pairs.next() {
            match method_pair.as_str() {
                "GET" => Ok(HttpMethod::Get),
                "POST" => Ok(HttpMethod::Post),
                "PUT" => Ok(HttpMethod::Put),
                "DELETE" => Ok(HttpMethod::Delete),
                "PATCH" => Ok(HttpMethod::Patch),
                _ => Ok(HttpMethod::Get), // Default value
            }
        } else {
            Ok(HttpMethod::Get) // Default value
        }
    }

    /// Parse access level
    fn parse_access_level(&self, pair: pest::iterators::Pair<Rule>) -> Result<AccessLevel, ParseError> {
        let mut inner_pairs = pair.into_inner();
        if let Some(level_pair) = inner_pairs.next() {
            match level_pair.as_str() {
                "public" => Ok(AccessLevel::Public),
                "private" => Ok(AccessLevel::Private),
                "internal" => Ok(AccessLevel::Internal),
                _ => Ok(AccessLevel::Public), // Default value
            }
        } else {
            Ok(AccessLevel::Public) // Default value
        }
    }

    /// Parse rate limit
    fn parse_rate_limit(
        &self,
        pair: pest::iterators::Pair<Rule>,
        _context: &ParseContext,
    ) -> Result<Option<RateLimit>, ParseError> {
        let mut inner_pairs = pair.into_inner();
        if let Some(limit_type_pair) = inner_pairs.next() {
            match limit_type_pair.as_rule() {
                Rule::rate_limit_simple => {
                    let mut limits = Vec::new();
                    for limit_pair in limit_type_pair.into_inner() {
                        if limit_pair.as_rule() == Rule::integer {
                            if let Ok(limit) = limit_pair.as_str().parse::<u32>() {
                                limits.push(limit);
                            }
                        }
                    }
                    if limits.len() == 2 {
                        return Ok(Some(RateLimit {
                            requests: limits[0],
                            per_seconds: limits[1],
                        }));
                    }
                }
                Rule::rate_limit_object => {
                    let mut requests = 0;
                    let mut per_seconds = 0;
                    
                    for field_pair in limit_type_pair.into_inner() {
                        if field_pair.as_rule() == Rule::rate_limit_field {
                            let mut field_inner = field_pair.into_inner();
                            if let Some(field_name_pair) = field_inner.next() {
                                if let Some(value_pair) = field_inner.next() {
                                    if value_pair.as_rule() == Rule::integer {
                                        let value = value_pair.as_str().parse::<u32>().unwrap_or(0);
                                        match field_name_pair.as_str() {
                                            "requests" => requests = value,
                                            "per_seconds" => per_seconds = value,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    if requests > 0 && per_seconds > 0 {
                        return Ok(Some(RateLimit { requests, per_seconds }));
                    }
                }
                _ => {}
            }
        }
        Ok(None)
    }

    /// Parse parameter definition
    fn parse_params_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        method_def: &mut MethodDef,
        _context: &ParseContext,
    ) -> Result<(), ParseError> {
        let inner_pairs = pair.into_inner();
        
        for param_pair in inner_pairs {
            if param_pair.as_rule() == Rule::param_def {
                let param_def = self.parse_param_def(param_pair, _context)?;
                method_def.params.insert(param_def.name.clone(), param_def);
            }
        }
        
        Ok(())
    }

    fn parse_param_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<ParamDef, ParseError> {
        let mut inner_pairs = pair.into_inner();
        let mut param_def = ParamDef {
            name: String::new(),
            type_expr: TypeExpr::Any,
            optional: false,
            default_value: None,
            description: None,
        };

        // Parse parameter name
        if let Some(name_pair) = inner_pairs.next() {
            if name_pair.as_rule() == Rule::identifier {
                param_def.name = name_pair.as_str().to_string();
            } else {
                return Err(context.syntax_error("Parameter definition missing name".to_string()));
            }
        }

        // Parse type expression, optional marker and default value
        for pair in inner_pairs {
            match pair.as_rule() {
                Rule::type_expr => {
                    param_def.type_expr = self.parse_type_expr(pair, context)?;
                }
                Rule::optional_marker => {
                    param_def.optional = true;
                }
                Rule::default_value => {
                    let value_pairs = pair.into_inner();
                    for value_pair in value_pairs {
                        match value_pair.as_rule() {
                            Rule::value => {
                                param_def.default_value = Some(self.parse_value(value_pair, context)?);
                            }
                            Rule::comment => {
                                // Comments in default values are ignored but parsing won't fail
                                // If needed, comment processing logic can be added here
                            }
                            _ => {}
                        }
                    }
                }
                Rule::comment => {
                    // Parse comment as description
                    let comment = pair.as_str().trim();
                    if comment.starts_with("//") {
                        param_def.description = Some(comment[2..].trim().to_string());
                    }
                }
                _ => {}
            }
        }

        Ok(param_def)
    }

    /// Parse response definition
    fn parse_response_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<TypeExpr, ParseError> {
        let mut resp_pairs = pair.into_inner();
        if let Some(resp_pair) = resp_pairs.next() {
            self.parse_type_expr(resp_pair, context)
        } else {
            Ok(TypeExpr::Any)
        }
    }

    /// Parse template definition
    fn parse_template_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<TemplateDef, ParseError> {
        let mut inner_pairs = pair.into_inner();
        let mut template_def = TemplateDef {
            name: String::new(),
            content: HashMap::new(),
        };

        // Parse template name
        if let Some(name_pair) = inner_pairs.next() {
            template_def.name = name_pair.as_str().to_string();
        }

        // Parse template content
        for pair in inner_pairs {
            if pair.as_rule() == Rule::template_content {
                // template_content can contain module_content rules and method_content
                for content_pair in pair.into_inner() {
                    match content_pair.as_rule() {
                        Rule::property_def => {
                            let (key, value) = self.parse_property_def(content_pair, context)?;
                            template_def.content.insert(key, value);
                        }
                        Rule::method_content => {
                            // Process method content (description, http_method, uri, access_level, rate_limit, params, response)
                            for method_pair in content_pair.into_inner() {
                                match method_pair.as_rule() {
                                    Rule::description_def => {
                                        if let Some(desc) = self.parse_string_content(method_pair)? {
                                            template_def.content.insert("description".to_string(), Value::String(desc));
                                        }
                                    }
                                    Rule::http_method_def => {
                                        let http = self.parse_http_method(method_pair)?;
                                        let http_str = match http {
                                            HttpMethod::Get => "GET",
                                            HttpMethod::Post => "POST",
                                            HttpMethod::Put => "PUT",
                                            HttpMethod::Delete => "DELETE",
                                            HttpMethod::Patch => "PATCH",
                                        };
                                        template_def.content.insert("http_method".to_string(), Value::String(http_str.to_string()));
                                    }
                                    Rule::uri_def => {
                                        let uri = self.parse_string_content(method_pair)?.unwrap_or_default();
                                        template_def.content.insert("uri".to_string(), Value::String(uri));
                                    }
                                    Rule::access_level_def => {
                                        let level = self.parse_access_level(method_pair)?;
                                        let level_str = match level {
                                            AccessLevel::Public => "public",
                                            AccessLevel::Private => "private",
                                            AccessLevel::Internal => "internal",
                                        };
                                        template_def.content.insert("access_level".to_string(), Value::String(level_str.to_string()));
                                    }
                                    Rule::rate_limit_def => {
                                        if let Some(rate) = self.parse_rate_limit(method_pair, context)? {
                                            let mut obj = HashMap::new();
                                            obj.insert("requests".to_string(), Value::Integer(rate.requests as i64));
                                            obj.insert("per_seconds".to_string(), Value::Integer(rate.per_seconds as i64));
                                            template_def.content.insert("rate_limit".to_string(), Value::Object(obj));
                                        }
                                    }
                                    Rule::params_def => {
                                        // Parameter definitions require special handling
                                        let params = self.parse_params_template(method_pair, context)?;
                                        template_def.content.insert("params".to_string(), Value::Object(params));
                                    }
                                    Rule::response_def => {
                                        let ty = self.parse_response_def(method_pair, context)?;
                                        template_def.content.insert("response".to_string(), Value::String(ty.to_string_repr()));
                                    }
                                    _ => {
                                        // Unknown template method content rule - ignored
                                    }
                                }
                            }
                        }
                        _ => {
                            // Unknown template content rule - ignored
                        }
                    }
                }
            }
        }

        Ok(template_def)
    }

    /// Parse property definition
    fn parse_property_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<(String, Value), ParseError> {
        let mut inner_pairs = pair.into_inner();
        let mut key = String::new();
        let mut value = Value::Null;

        // Parse property key
        if let Some(key_pair) = inner_pairs.next() {
            if key_pair.as_rule() == Rule::identifier {
                key = key_pair.as_str().to_string();
            }
        }

        // Parse property value
        for pair in inner_pairs {
            if pair.as_rule() == Rule::property_value {
                value = self.parse_property_value(pair, context)?;
            }
        }

        Ok((key, value))
    }

    /// Parse parameter definitions in templates
    fn parse_params_template(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<HashMap<String, Value>, ParseError> {
                let mut params = HashMap::new();

        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::param_def {
                let param_def = self.parse_param_def(inner_pair, context)?;
                let mut param_obj = HashMap::new();

                // Convert parameter definition to object using TypeExpr string representation
                param_obj.insert(
                    "type".to_string(),
                    Value::String(param_def.type_expr.to_string_repr()),
                );
                param_obj.insert("optional".to_string(), Value::Boolean(param_def.optional));

                if let Some(default) = param_def.default_value {
                    param_obj.insert("default".to_string(), default);
                }

                if let Some(desc) = param_def.description {
                    param_obj.insert("description".to_string(), Value::String(desc));
                }

                params.insert(param_def.name, Value::Object(param_obj));
            }
        }

        Ok(params)
    }

    /// Parse property value
    fn parse_property_value(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<Value, ParseError> {
        match pair.as_rule() {
            Rule::property_value => {
                // Recursively parse sub-rules inside property_value
                let mut inner_pairs = pair.into_inner();
                if let Some(inner_pair) = inner_pairs.next() {
                    self.parse_property_value(inner_pair, context)
                } else {
                    Err(context.type_error("Null value is not supported".to_string()))
                }
            }
            Rule::string => {
                let s = pair.as_str();
                // Remove quotes
                Ok(Value::String(s[1..s.len() - 1].to_string()))
            }
            Rule::integer => {
                let num_str = pair.as_str();
                Ok(Value::Integer(num_str.parse().map_err(|e| {
                    context.type_error(format!("Failed to parse integer: {}", e))
                })?))
            }
            Rule::number => {
                let num_str = pair.as_str();
                if num_str.contains('.') {
                    Ok(Value::Number(num_str.parse().map_err(|e| {
                        context.type_error(format!("Failed to parse number: {}", e))
                    })?))
                } else {
                    Ok(Value::Integer(num_str.parse().map_err(|e| {
                        context.type_error(format!("Failed to parse integer: {}", e))
                    })?))
                }
            }
            Rule::boolean => match pair.as_str() {
                "true" => Ok(Value::Boolean(true)),
                "false" => Ok(Value::Boolean(false)),
                _ => Err(context.type_error(format!("Invalid boolean value: {}", pair.as_str()))),
            },
            Rule::identifier => Ok(Value::String(pair.as_str().to_string())),
            _ => Err(context.type_error(format!("Unsupported property value type: {:?}", pair.as_rule()))),
        }
    }

    /// Parse resource definition
    fn parse_resource_def(
        &self,
        pair: pest::iterators::Pair<Rule>,
        context: &ParseContext,
    ) -> Result<ResourceDef, ParseError> {
        let mut inner_pairs = pair.into_inner();
        let mut resource_def = ResourceDef {
            name: String::new(),
            resource_type: ResourceType::Entity, // Default type
            uri: String::new(),
            description: None,
        };

        // Parse resource name
        if let Some(name_pair) = inner_pairs.next() {
            resource_def.name = name_pair.as_str().to_string();
        }

        // Parse resource content
        for content_pair in inner_pairs {
            match content_pair.as_rule() {
                Rule::type_def => {
                    // Parse resource type
                    let type_def = self.parse_type_def(content_pair, context)?;
                    // Set resource type based on type name
                    match type_def.name.as_str() {
                        "collection" => resource_def.resource_type = ResourceType::Collection,
                        "entity" => resource_def.resource_type = ResourceType::Entity,
                        _ => {}
                    }
                }
                Rule::uri_def => {
                    resource_def.uri = self.parse_string_content(content_pair)?.unwrap_or_default();
                }
                Rule::description_def => {
                    resource_def.description = self.parse_string_content(content_pair)?;
                }
                Rule::property_def => {
                    // Parse property definition (e.g., type: collection)
                    let mut inner_pairs = content_pair.into_inner();
                    if let (Some(key_pair), Some(value_pair)) = (inner_pairs.next(), inner_pairs.next()) {
                        let key = key_pair.as_str();
                        let value = self.parse_property_value(value_pair, context)?;
                        
                        match key {
                            "type" => {
                                if let Value::String(s) = value {
                                    match s.as_str() {
                                        "collection" => resource_def.resource_type = ResourceType::Collection,
                                        "entity" => resource_def.resource_type = ResourceType::Entity,
                                        _ => {}
                                    }
                                }
                            }
                            "uri" => {
                                if let Value::String(s) = value {
                                    resource_def.uri = s;
                                }
                            }
                            "description" => {
                                if let Value::String(s) = value {
                                    resource_def.description = Some(s);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(resource_def)
    }

    /// Set module properties
    fn set_module_property(&self, module: &mut Module, key: &str, value: Value) {
        match key {
            "version" => {
                if let Value::String(s) = value {
                    module.version = Some(s);
                }
            }
            "description" => {
                if let Value::String(s) = value {
                    module.description = Some(s);
                }
            }
            "enabled" => {
                if let Value::Boolean(b) = value {
                    module.enabled = Some(b);
                }
            }
            "access_level" => {
                if let Value::String(s) = value {
                    module.access_level = match s.as_str() {
                        "public" => Some(AccessLevel::Public),
                        "private" => Some(AccessLevel::Private),
                        "internal" => Some(AccessLevel::Internal),
                        _ => None,
                    };
                }
            }
            "category" => {
                if let Value::String(s) = value {
                    module.category = Some(s);
                }
            }
            _ => {}
        }
    }

    /// Validate module
    fn validate_module(&self, module: &Module, context: &ParseContext) -> Result<(), ParseError> {
        // Check module name
        if module.name.is_empty() {
            return Err(context.semantic_error("Module name cannot be empty".to_string()));
        }

        // Validate type references
        for (type_name, type_def) in &module.types {
            for (field_name, field_def) in &type_def.fields {
                if let TypeExpr::Ref(ref_type_name) = &field_def.type_expr {
                    if !module.types.contains_key(ref_type_name) {
                        return Err(context.reference_error(format!(
                            "Type '{}' field '{}' references non-existent type '{}'",
                            type_name, field_name, ref_type_name
                        )));
                    }
                }
            }
        }

        // Validate method parameter types
        for (_, method_def) in &module.methods {
            for (_, param_def) in &method_def.params {
                self.validate_type_expr(&param_def.type_expr, module, context)?;
            }
            self.validate_type_expr(&method_def.response, module, context)?;
        }

        Ok(())
    }

    /// Validate type expression
    fn validate_type_expr(
        &self,
        type_expr: &TypeExpr,
        module: &Module,
        context: &ParseContext,
    ) -> Result<(), ParseError> {
        match type_expr {
            TypeExpr::Ref(ref_name) => {
                if !module.types.contains_key(ref_name) {
                    return Err(context.reference_error(format!("Referenced type not found: {}", ref_name)));
                }
            }
            TypeExpr::Array(element_type) => {
                self.validate_type_expr(element_type, module, context)?;
            }
            TypeExpr::Object(fields) => {
                for (_, field_def) in fields {
                    self.validate_type_expr(&field_def.type_expr, module, context)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Convert Pest error
    fn convert_pest_error(&self, error: pest::error::Error<Rule>, _source: &str) -> ParseError {
        let (line, column) = match error.line_col {
            pest::error::LineColLocation::Pos((line, column)) => (line, column),
            pest::error::LineColLocation::Span((line1, column1), (_line2, _column2)) => {
                (line1, column1)
            }
        };
        ParseError::SyntaxError {
            message: error.to_string(),
            line,
            column,
        }
    }

    /// Get parsed module
    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    /// Clear all cached modules
    pub fn clear_cache(&mut self) {
        self.modules.clear();
    }
}

impl Default for ZMLParserWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_module() {
        let source = r#"
module UserModule {
    version: "1.0.0"
    description: "User management module"
    
    type User {
        id: integer
        name: string
        email: string?
        age: integer? = 0
    }
}
"#;

        let mut parser = ZMLParserWrapper::new();
        let result = parser.parse(source);
        
        // Print error information for debugging
        if let Err(e) = &result {
            println!("Parse error: {:?}", e);
        }
        
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.name, "UserModule");
        assert_eq!(module.version, Some("1.0.0".to_string()));
        assert_eq!(module.description, Some("User management module".to_string()));
        assert!(module.types.contains_key("User"));
    }

    #[test]
    fn test_parse_type_module() {
        let source = r#"
module TypeModule {
    version: "1.0.0"

    type User {
        id: integer
        name: string
        email: string
        age: integer? = 18
        active: boolean = true
    }

    type Product {
        id: integer
        name: string
        price: number
        tags: array<string>
    }
}
"#;

        let mut parser = ZMLParserWrapper::new();
        let result = parser.parse(source);
        
        // Print error information for debugging
        if let Err(e) = &result {
            println!("Parse error: {:?}", e);
        }
        
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.name, "TypeModule");
        assert_eq!(module.version, Some("1.0.0".to_string()));
        
        // Verify User type
        assert!(module.types.contains_key("User"));
        let user_type = &module.types["User"];
        assert_eq!(user_type.fields.len(), 5);
        
        // Verify Product type
        assert!(module.types.contains_key("Product"));
        let product_type = &module.types["Product"];
        assert_eq!(product_type.fields.len(), 4);
    }

    #[test]
    fn test_parse_template_method_content() {
        let source = r#"
template base_method {
    description: "Base method"
    http_method: GET
    uri: "/v1/users"
    access_level: internal
    rate_limit: 100/60
    params {
        id: integer
        name: string? = "guest"
    }
    response: array<string>
}

module Minimal { }
"#;

        let mut parser = ZMLParserWrapper::new();
        let result = parser.parse(source);

        if let Err(e) = &result {
            println!("Parse error: {:?}", e);
        }

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name, "Minimal");
        assert!(module.templates.contains_key("base_method"));

        let tmpl = &module.templates["base_method"];
        let content = &tmpl.content;

        // Basic attributes
        assert_eq!(
            content.get("description"),
            Some(&Value::String("Base method".to_string()))
        );
        assert_eq!(content.get("http_method"), Some(&Value::String("GET".to_string())));
        assert_eq!(content.get("uri"), Some(&Value::String("/v1/users".to_string())));
        assert_eq!(content.get("access_level"), Some(&Value::String("internal".to_string())));

        // Rate limit
        let rate = content.get("rate_limit").expect("rate_limit missing");
        if let Value::Object(obj) = rate {
            assert_eq!(obj.get("requests"), Some(&Value::Integer(100)));
            assert_eq!(obj.get("per_seconds"), Some(&Value::Integer(60)));
        } else {
            panic!("rate_limit should be an object");
        }

        // Params
        let params = content.get("params").expect("params missing");
        if let Value::Object(pmap) = params {
            // id param
            let id_param = pmap.get("id").expect("id param missing");
            if let Value::Object(id_obj) = id_param {
                assert_eq!(id_obj.get("type"), Some(&Value::String("integer".to_string())));
                assert_eq!(id_obj.get("optional"), Some(&Value::Boolean(false)));
            } else {
                panic!("id param should be an object");
            }
            // name param
            let name_param = pmap.get("name").expect("name param missing");
            if let Value::Object(name_obj) = name_param {
                assert_eq!(name_obj.get("type"), Some(&Value::String("string".to_string())));
                assert_eq!(name_obj.get("optional"), Some(&Value::Boolean(true)));
                assert_eq!(name_obj.get("default"), Some(&Value::String("guest".to_string())));
            } else {
                panic!("name param should be an object");
            }
        } else {
            panic!("params should be an object");
        }

        // Response
        assert_eq!(
            content.get("response"),
            Some(&Value::String("array<string>".to_string()))
        );
    }
}
