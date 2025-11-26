// ZML集成测试
use mcp_any_rest::zml::{process_zml, ZMLProcessor};

#[test]
fn test_zml_processor_basic() {
    let source = r#"
module TestModule {
    version: "1.0.0"
    description: "Test module"
    enabled: true
    access_level: public
    category: "test"
}
"#;

    let mut processor = ZMLProcessor::new();
    let result = processor.process(source);
    assert!(result.is_ok(), "ZML processing failed: {:?}", result.err());
    
    let json = result.unwrap();
    assert_eq!(json["name"], "TestModule");
    assert_eq!(json["version"], "1.0.0");
    assert_eq!(json["description"], "Test module");
    assert_eq!(json["enabled"], true);
    assert_eq!(json["access_level"], "public");
}

#[test]
fn test_process_zml_function() {
    let source = r#"
module SimpleModule {
    version: "1.0.0"
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "ZML processing failed: {:?}", result.err());
    
    let json = result.unwrap();
    assert_eq!(json["name"], "SimpleModule");
    assert_eq!(json["version"], "1.0.0");
}

#[test]
fn test_zml_with_types() {
    let source = r#"
module UserModule {
    version: "1.0.0"
    
    type User {
        id: integer
        name: string
        email: string
        age: integer? = 18
        active: boolean = true
    }
    
    enum Status {
        pending
        active
        inactive
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "ZML processing failed: {:?}", result.err());
    
    let json = result.unwrap();
    assert_eq!(json["name"], "UserModule");
    assert_eq!(json["version"], "1.0.0");
    
    // 检查类型定义
    let types = json["types"].as_object().expect("Should contain types field");
    assert!(types.contains_key("User"));
    
    // 检查枚举定义
    let enums = json["enums"].as_object().expect("Should contain enums field");
    assert!(enums.contains_key("Status"));
}

#[test]
fn test_zml_with_methods() {
    let source = r#"
module ApiModule {
    version: "1.0.0"
    
    method getUser {
        description: "Get user information"
        http_method: GET
        uri: "/users/{id}"
        access_level: public
        rate_limit: { requests: 100, per_seconds: 60 }
        
        params {
            id: integer
            include_details: boolean? = false
        }
        
        response: integer
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "ZML processing failed: {:?}", result.err());
    
    let json = result.unwrap();
    assert_eq!(json["name"], "ApiModule");
    
    // 检查方法定义
    let methods = json["methods"].as_object().expect("Should contain methods field");   
    assert!(methods.contains_key("getUser"));
}

#[test]
fn test_zml_with_resources() {
    let source = r#"
module ResourceModule {
    version: "1.0.0"
    
    resource users {
        type: collection
        uri: "/users"
        description: "User resource collection"
    }
    
    resource user {
        type: entity
        uri: "/users/{id}"
        description: "Single user resource"
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "ZML processing failed: {:?}", result.err());
    
    let json = result.unwrap();
    assert_eq!(json["name"], "ResourceModule");
    
    // Check resource definitions
    let resources = json["resources"].as_object().expect("Should contain resources field");
    assert!(resources.contains_key("users"));
    assert!(resources.contains_key("user"));
}

#[test]
fn test_zml_complex_example() {
    let source = r#"
module ComplexModule {
        version: "1.0.0"
        description: "Complex module example"
        enabled: true
        access_level: internal
        category: "complex"
        
        type User {
        id: integer
        name: string
        email: string
        age: integer? = 18
        active: boolean = true
        tags: array<string>
        metadata: object {
            key: string
            value: any
        }
    }
    
    enum Status {
        pending
        active
        inactive
    }
    
    method getUser {
        description: "Get user information"
        http_method: GET
        uri: "/users/{id}"
        access_level: public
        rate_limit: { requests: 100, per_seconds: 60 }
        
        params {
            id: integer
            include_details: boolean? = false
        }
        
        response: User
    }
    
    method createUser {
        description: "Create a new user"
        http_method: POST
        uri: "/users"
        access_level: internal
        
        params {
            name: string
            email: string
        }
        
        response: User
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "ZML processing failed: {:?}", result.err());
    
    let json = result.unwrap();
    assert_eq!(json["name"], "ComplexModule");
    assert_eq!(json["version"], "1.0.0");
    
    // Check type definitions
    let types = json["types"].as_object().expect("Should contain types field");
    assert!(types.contains_key("User"));
    
    // Check enum definitions
    let enums = json["enums"].as_object().expect("Should contain enums field");
    assert!(enums.contains_key("Status"));
    let status_enum = &enums["Status"];
    assert_eq!(status_enum["name"], "Status");
    let status_values = status_enum["values"].as_object().expect("Enum should contain values");
    assert!(status_values.contains_key("pending"));
    assert!(status_values.contains_key("active"));
    assert!(status_values.contains_key("inactive"));
    
    // Check method definitions
    let methods = json["methods"].as_object().expect("Should contain methods field");
    assert!(methods.contains_key("getUser"));
    assert!(methods.contains_key("createUser"));
}

#[test]
fn test_zml_error_handling() {
    // Test syntax error
    let invalid_source = "module { invalid syntax }";
    let result = process_zml(invalid_source);
    assert!(result.is_err(), "Should return error");
    
    // Test empty input
    let empty_source = "";
    let result = process_zml(empty_source);
    assert!(result.is_err(), "Should return error");
}

#[test]
fn test_zml_file_processing() {
    // Test file processing (requires creating a temporary file)
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    
    let dir = tempdir().expect("Failed to create temporary directory");
    let file_path = dir.path().join("test.zml");
    
    let mut file = File::create(&file_path).expect("Failed to create temporary file");
    file.write_all(b"module FileModule { version: \"1.0.0\" }").expect("Failed to write to temporary file");
    
    let result = mcp_any_rest::zml::process_zml_file(file_path.to_str().unwrap());
    assert!(result.is_ok(), "Failed to process ZML file: {:?}", result.err());
    
    let json = result.unwrap();
    assert_eq!(json["name"], "FileModule");
    assert_eq!(json["version"], "1.0.0");
    
    dir.close().expect("Failed to delete temporary directory");
}

#[test]
fn test_zml_enum_basic() {
    let source = r#"
module EnumModule {
    version: "1.0.0"
    
    enum UserStatus {
        ACTIVE
        INACTIVE
        SUSPENDED
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Failed to process ZML file: {:?}", result.err());
    
    let json = result.unwrap();
    assert_eq!(json["name"], "EnumModule");
    
    let enums = json["enums"].as_object().expect("Should contain enums field");
    assert!(enums.contains_key("UserStatus"));
    
    let user_status = &enums["UserStatus"];
    assert_eq!(user_status["name"], "UserStatus");
    
    let values = user_status["values"].as_object().expect("Enum should contain values");
    assert_eq!(values.len(), 3);
    assert!(values.contains_key("ACTIVE"));
    assert!(values.contains_key("INACTIVE"));
    assert!(values.contains_key("SUSPENDED"));
}

#[test]
fn test_zml_enum_with_values() {
    let source = r#"
module PriorityModule {
    version: "1.0.0"
    
    enum Priority {
        LOW = 1
        MEDIUM = 2
        HIGH = 3
        URGENT = 4
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Failed to process ZML file: {:?}", result.err());
    
    let json = result.unwrap();
    let enums = json["enums"].as_object().expect("Should contain enums field");
    let priority = &enums["Priority"];
    
    let values = priority["values"].as_object().expect("Enum should contain values");
    assert_eq!(values["LOW"]["value"], 1);
    assert_eq!(values["MEDIUM"]["value"], 2);
    assert_eq!(values["HIGH"]["value"], 3);
    assert_eq!(values["URGENT"]["value"], 4);
}

#[test]
fn test_zml_enum_with_comments() {
    let source = r#"
module StatusModule {
    version: "1.0.0"
    
    enum Status {
        // Draft status
        DRAFT
        // Published status
        PUBLISHED
        // Archived status
        ARCHIVED
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Failed to process ZML file: {:?}", result.err());
    
    let json = result.unwrap();
    let enums = json["enums"].as_object().expect("Should contain enums field");
    assert!(enums.contains_key("Status"));
}

#[test]
fn test_zml_enum_in_method_params() {
    let source = r#"
module UserModule {
    version: "1.0.0"
    
    enum UserStatus {
        ACTIVE
        INACTIVE
    }
    
    type User {
        id: integer
        name: string
        status: UserStatus
    }
    
    method updateUserStatus {
        params {
            userId: integer
            newStatus: UserStatus
        }
        response: boolean
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Failed to process ZML file: {:?}", result.err());
    
    let json = result.unwrap();
    let types = json["types"].as_object().expect("Should contain types field");
    let user_type = &types["User"];
    
    // 验证类型字段使用枚举
    let properties = user_type["properties"].as_object().expect("Type should contain properties");
    assert_eq!(properties["status"]["type"], "UserStatus");
    
    // 验证方法参数使用枚举
    let methods = json["methods"].as_object().expect("Should contain methods field");
    let update_method = &methods["updateUserStatus"];
    let params = update_method["params"].as_object().expect("Method should contain params");
    assert_eq!(params["newStatus"]["type"], "UserStatus");
}

#[test]
fn test_zml_enum_in_array() {
    let source = r#"
module PermissionModule {
    version: "1.0.0"
    
    enum Permission {
        READ
        WRITE
        DELETE
    }
    
    type Role {
        id: integer
        name: string
        permissions: array<Permission>
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Failed to process ZML file: {:?}", result.err());
    
    let json = result.unwrap();
    let types = json["types"].as_object().expect("Should contain types field");
    let role_type = &types["Role"];
    
    let properties = role_type["properties"].as_object().expect("Type should contain properties");
    let permissions_type = &properties["permissions"]["type"];
    assert_eq!(permissions_type["type"], "array");
    assert_eq!(permissions_type["items"], "Permission");
}

#[test]
fn test_zml_enum_optional_parameter() {
    let source = r#"
module ConfigModule {
    version: "1.0.0"
    
    enum Theme {
        LIGHT
        DARK
        AUTO
    }
    
    type UserPreferences {
        userId: integer
        theme: Theme? = LIGHT
        notifications: boolean = true
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Failed to process ZML file: {:?}", result.err());
    
    let json = result.unwrap();
    let types = json["types"].as_object().expect("Should contain types field");
    let prefs_type = &types["UserPreferences"];
    
    let properties = prefs_type["properties"].as_object().expect("Type should contain properties");
    let theme_field = &properties["theme"];
    assert_eq!(theme_field["type"], "Theme");
    assert_eq!(theme_field["optional"], true);
    assert_eq!(theme_field["default"], "LIGHT");
}

#[test]
fn test_zml_multiple_enums() {
    let source = r#"
module MultiEnumModule {
    version: "1.0.0"
    
    enum UserStatus {
        ACTIVE
        INACTIVE
        BANNED
    }
    
    enum OrderStatus {
        PENDING
        PROCESSING
        COMPLETED
        CANCELLED
    }
    
    enum Priority {
        LOW = 1
        MEDIUM = 2
        HIGH = 3
    }
    
    type User {
        id: integer
        status: UserStatus
    }
    
    type Order {
        id: integer
        status: OrderStatus
        priority: Priority
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Failed to process ZML file: {:?}", result.err());
    
    let json = result.unwrap();
    let enums = json["enums"].as_object().expect("Should contain enums field");
    
    assert_eq!(enums.len(), 3);
    assert!(enums.contains_key("UserStatus"));
    assert!(enums.contains_key("OrderStatus"));
    assert!(enums.contains_key("Priority"));
}

#[test]
fn test_zml_enum_special_names() {
    let source = r#"
module SpecialEnumModule {
    version: "1.0.0"
    
    enum HttpMethod {
        GET
        POST
        PUT
        DELETE
        PATCH
    }
    
    enum ContentType {
        APPLICATION_JSON = "application/json"
        TEXT_HTML = "text/html"
        TEXT_PLAIN = "text/plain"
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Failed to process ZML file: {:?}", result.err());
    
    let json = result.unwrap();
    let enums = json["enums"].as_object().expect("Should contain enums field");
    
    let http_method = &enums["HttpMethod"];
    let values = http_method["values"].as_object().expect("Enum should contain values");
    assert_eq!(values.len(), 5);
    
    let content_type = &enums["ContentType"];
    let ct_values = content_type["values"].as_object().expect("Enum should contain values");
    assert_eq!(ct_values["APPLICATION_JSON"]["value"], "application/json");
    assert_eq!(ct_values["TEXT_HTML"]["value"], "text/html");
}