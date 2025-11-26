// ZML enum comprehensive test suite
use mcp_any_rest::zml::process_zml;

#[test]
fn test_enum_basic_definition() {
    let source = r#"
module TestModule {
    enum SimpleEnum {
        VALUE1
        VALUE2
        VALUE3
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Basic enum definition failed: {:?}", result.err());
    
    let json = result.unwrap();
    let enums = json["enums"].as_object().expect("Should contain enums field");
    assert!(enums.contains_key("SimpleEnum"));
    
    let simple_enum = &enums["SimpleEnum"];
    assert_eq!(simple_enum["name"], "SimpleEnum");
    assert!(simple_enum["values"].is_object());
}

#[test]
fn test_enum_with_comments() {
    let source = r#"
module TestModule {
    enum UserStatus {
        ACTIVE // Active user
        INACTIVE // Inactive user
        SUSPENDED // Suspended user
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Enum with comments failed: {:?}", result.err());
    
    let json = result.unwrap();
    let enums = json["enums"].as_object().expect("Should contain enums field"); 
    assert!(enums.contains_key("UserStatus"));
}

#[test]
fn test_enum_with_values() {
    let source = r#"
module TestModule {
    enum Priority {
        LOW = 1
        MEDIUM = 2
        HIGH = 3
        URGENT = 4
    }
    
    enum Color {
        RED = "red"
        GREEN = "green"
        BLUE = "blue"
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Enum with values failed: {:?}", result.err());
    
    let json = result.unwrap();
    let enums = json["enums"].as_object().expect("Should contain enums field"); 
    assert!(enums.contains_key("Priority"));
    assert!(enums.contains_key("Color"));
}

#[test]
fn test_enum_in_type_fields() {
    let source = r#"
module TestModule {
    enum UserStatus {
        ACTIVE
        INACTIVE
        SUSPENDED
    }
    
    type User {
        id: integer
        name: string
        status: UserStatus
        priority: Priority
    }
    
    enum Priority {
        LOW
        MEDIUM
        HIGH
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Enum in type fields failed: {:?}", result.err());
    
    let json = result.unwrap();
    let types = json["types"].as_object().expect("Should contain types field");
    let enums = json["enums"].as_object().expect("Should contain enums field");
    
    assert!(types.contains_key("User"));
    assert!(enums.contains_key("UserStatus"));
    assert!(enums.contains_key("Priority"));
}

#[test]
fn test_enum_in_method_params() {
    let source = r#"
module TestModule {
    enum UserStatus {
        ACTIVE
        INACTIVE
    }
    
    method updateStatus {
        description: "Update user status"
        http_method: POST
        uri: "/users/{id}/status"
        
        params {
            id: integer
            status: UserStatus
            reason: string
        }
        
        response: boolean
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Enum in method params failed: {:?}", result.err());
    
    let json = result.unwrap();
    let methods = json["methods"].as_object().expect("Should contain methods field");
    assert!(methods.contains_key("updateStatus"));
}

#[test]
fn test_enum_optional_parameters() {
    let source = r#"
module TestModule {
    enum Status {
        ACTIVE
        INACTIVE
    }
    
    type User {
        id: integer
        name: string
        status: Status? = ACTIVE
        priority: Priority? = LOW
    }
    
    enum Priority {
        LOW = 1
        MEDIUM = 2
        HIGH = 3
    }
    
    method createUser {
        params {
            name: string
            status: Status? = ACTIVE
            priority: Priority? = LOW
        }
        
        response: User
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Enum optional params failed: {:?}", result.err());
    
    let json = result.unwrap();
    let types = json["types"].as_object().expect("Should contain types field");
    let methods = json["methods"].as_object().expect("Should contain methods field");
    
    assert!(types.contains_key("User"));
    assert!(methods.contains_key("createUser"));
}

#[test]
fn test_enum_array_types() {
    let source = r#"
module TestModule {
    enum Permission {
        READ
        WRITE
        DELETE
        ADMIN
    }
    
    type Role {
        id: integer
        name: string
        permissions: array<Permission>
    }
    
    method updatePermissions {
        params {
            roleId: integer
            permissions: array<Permission>
        }
        
        response: Role
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Enum array types failed: {:?}", result.err());
    
    let json = result.unwrap();
    let types = json["types"].as_object().expect("Should contain types field");
    let methods = json["methods"].as_object().expect("Should contain methods field");
    
    assert!(types.contains_key("Role"));
    assert!(methods.contains_key("updatePermissions"));
}

#[test]
fn test_enum_complex_scenario() {
    let source = r#"
module ComplexEnumModule {
    version: "1.0.0"
    description: "Complex enum scenario test"
    
    // User status enum
    enum UserStatus {
        ACTIVE = "active"
        INACTIVE = "inactive"
        SUSPENDED = "suspended"
        DELETED = "deleted"
    }
    
    // Priority enum
    enum Priority {
        LOW = 1
        MEDIUM = 2
        HIGH = 3
        URGENT = 4
    }
    
    // Permission enum
    enum Permission {
        READ
        WRITE
        DELETE
        ADMIN
    }
    
    type User {
        id: integer
        name: string
        email: string
        status: UserStatus = ACTIVE
        priority: Priority = MEDIUM
        permissions: array<Permission>
    }
    
    method createUser {
        description: "Create a new user"    
        http_method: POST
        uri: "/users"
        
        params {
            name: string
            email: string
            status: UserStatus? = ACTIVE
            priority: Priority? = MEDIUM
            permissions: array<Permission>?
        }
        
        response: User
    }
    
    method updateUserStatus {
        description: "Update user status"       
        http_method: PUT
        uri: "/users/{id}/status"
        
        params {
            id: integer
            status: UserStatus
            reason: string
        }
        
        response: User
    }
    
    resource users {
        type: collection
        uri: "/users"
        description: "User resource collection"
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Complex enum scenario failed: {:?}", result.err());
    
    let json = result.unwrap();
    assert_eq!(json["name"], "ComplexEnumModule");
    assert_eq!(json["version"], "1.0.0");
    
    let enums = json["enums"].as_object().expect("Should contain enums field");
    let types = json["types"].as_object().expect("Should contain types field");
    let methods = json["methods"].as_object().expect("Should contain methods field");
    let resources = json["resources"].as_object().expect("Should contain resources field");
    
    // Verify enum definitions
    assert!(enums.contains_key("UserStatus"));
    assert!(enums.contains_key("Priority"));
    assert!(enums.contains_key("Permission"));
    
    // Verify type definitions
    assert!(types.contains_key("User"));
    
    // Verify method definitions
    assert!(methods.contains_key("createUser"));
    assert!(methods.contains_key("updateUserStatus"));
    
    // Verify resource definitions
    assert!(resources.contains_key("users"));
}

#[test]
fn test_enum_error_cases() {
    // Test empty enum definition
    let empty_enum = r#"
module TestModule {
    enum EmptyEnum {
    }
}
"#;
    
    let result = process_zml(empty_enum);
    assert!(result.is_ok(), "Empty enum definition should be allowed: {:?}", result.err());
    
    // Test duplicate enum values (should be allowed, handled later)
    let duplicate_values = r#"
module TestModule {
    enum Status {
        ACTIVE
        ACTIVE
    }
}
"#;
    
    let result = process_zml(duplicate_values);
    assert!(result.is_ok(), "Duplicate enum values should be allowed: {:?}", result.err());
}

#[test]
fn test_enum_with_special_characters() {
    let source = r#"
module TestModule {
    enum ErrorCode {
        SUCCESS = 0
        INVALID_INPUT = 1001
        UNAUTHORIZED = 2001
        FORBIDDEN = 2003
        NOT_FOUND = 404
        INTERNAL_ERROR = 500
    }
    
    enum HttpMethod {
        GET = "GET"
        POST = "POST"
        PUT = "PUT"
        DELETE = "DELETE"
        PATCH = "PATCH"
    }
}
"#;

    let result = process_zml(source);
    assert!(result.is_ok(), "Enum with special characters failed: {:?}", result.err());
    
    let json = result.unwrap();
    let enums = json["enums"].as_object().expect("Should contain enums field");
    assert!(enums.contains_key("ErrorCode"));
    assert!(enums.contains_key("HttpMethod"));
}