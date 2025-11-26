//! ZML Parser Demo Program

use mcp_any_rest::zml::process_zml;

fn main() {
    println!("=== ZML Parser Demo Program ===\n");
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let demo_num = if args.len() > 2 && args[1] == "--demo" {
        match args[2].parse::<u32>() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Error: Invalid demo number");
                return;
            }
        }
    } else {
        // Default to run all demos
        println!("Running all demos...\n"); 
        demo_basic_module();
        demo_module_with_types();
        demo_module_with_methods();
        demo_complex_module();
        demo_enum_and_description();
        demo_error_handling();
        return;
    };
    
    // Run the specified demo
    match demo_num {
        1 => demo_basic_module(),
        2 => demo_module_with_types(),
        3 => demo_module_with_methods(),
        4 => demo_complex_module(),
        5 => demo_enum_and_description(),
        6 => demo_error_handling(),
        _ => eprintln!("Error: Invalid demo number. Please use 1-6."),
    }
}

fn demo_enum_and_description() {
    println!("Demo 5: Enum Definitions and Field Descriptions");
    println!("----------------------------------------------");
    
    let source = r#"
module EnumModule {
    version: "1.0.0"
    description: "Enum Definitions and Field Descriptions Example"
    enabled: true
    access_level: internal
    category: "enum"
    
    enum UserStatus {
        ACTIVE // Active user
        INACTIVE // Inactive user
        SUSPENDED // Suspended user
        DELETED // Deleted user
    }
    
    enum Priority {
        LOW = 1 // Low priority
        MEDIUM = 2 // Medium priority
        HIGH = 3 // High priority
        URGENT = 4 // Urgent priority
    }
    
    enum Color {
        RED = "red" // Red
        GREEN = "green" // Green
        BLUE = "blue" // Blue
        BLACK // Black (default value)
        WHITE // White (default value)
    }
    
    type User {
        id: integer // User ID
        name: string // User name
        email: string // User email
        status: UserStatus // User status
        priority: Priority? = Priority.MEDIUM // Priority, default is medium
        favorite_color: Color? // Favorite color, optional
        age: integer? = 18 // Age, optional, default is 18
        active: boolean = true // Whether active, default is true
    }
    
    method updateUserStatus {
        description: "Update user status"
        http_method: POST
        uri: "/users/{userId}/status"
        access_level: internal
        
        params {
            id: integer // User ID
            status: UserStatus // New user status
            reason: string? // Reason description, optional
        }
        
        response: User
    }
}"#;
    
    println!("ZML Code:");
    println!("{}", source);
    
    match process_zml(source) {
        Ok(json) => {
            println!("Parsing successful! JSON output:");
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        Err(e) => {
            println!("Parsing failed: {}", e);
        }
    }
    
    println!("\n");
}

fn demo_basic_module() {
    println!("Demo 1: Basic Module Parsing");
    println!("----------------------------------------------");
    
    let source = r#"
module BasicModule {
    version: "1.0.0"
    description: "Basic Module Example"
    enabled: true
    access_level: public
    category: "demo"
}
"#;
    
    println!("ZML Code:");
    println!("{}", source);
    
    match process_zml(source) {
        Ok(json) => {
            println!("Parsing successful! JSON output:");
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        Err(e) => {
            println!("Parsing failed: {}", e);
        }
    }
    
    println!("\n");
}

fn demo_module_with_types() {
    println!("Demo 2: Module with Type Definitions");
    println!("----------------------------------------------");     
    
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
    
    type Status {
        value: string
    }
    
    type Product {
        id: integer
        name: string
        price: number
        tags: array<string>
    }
}
"#;
    
    println!("ZML Code:");
    println!("{}", source);
    
    match process_zml(source) {
        Ok(json) => {
            println!("Parsing successful! JSON output:");
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        Err(e) => {
            println!("Parsing failed: {}", e);
        }
    }
    
    println!("\n");
}

fn demo_module_with_methods() {
    println!("Demo 3: Module with Method Definitions");
    println!("----------------------------------------------");     
    println!("------------------------");
    
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
    
    method createUser {
        description: "Create a new user"    
        http_method: POST
        uri: "/users"
        access_level: internal
        
        params {
            name: string
            email: string
            age: integer? = 18
        }
        
        response: integer
    }
}
"#;
    
    println!("ZML Code:");
    println!("{}", source);
    
    match process_zml(source) {
        Ok(json) => {
            println!("Parsing successful! JSON output:");
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        Err(e) => {
            println!("Parsing failed: {}", e);
        }
    }
    
    println!("\n");
}

fn demo_complex_module() {
    println!("Demo 4: Complex Module Parsing");
    println!("----------------------------------------------");         
    println!("------------------------");
    
    let source = r#"
module ComplexModule {
    version: "1.0.0"
    description: "Complex Module Example"
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
    
    type Status {
        value: string
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
    
    resource users {
        uri: "/users"
        description: "User resource collection"
    }
}
"#;
    
    println!("ZML Code:");
    println!("{}", source);
    
    match process_zml(source) {
        Ok(json) => {
            println!("Parsing successful! JSON output:");
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        Err(e) => {
            println!("Parsing failed: {}", e);
        }
    }
    
    println!("\n");
}

fn demo_error_handling() {
    println!("Demo 6: Error Handling");
    println!("----------------------------------------------");         
    println!("------------------------");
    
    // 测试语法错误
    let invalid_source = "module InvalidModule { invalid syntax }";
    println!("Invalid ZML Code: {}", invalid_source);
    
    match process_zml(invalid_source) {
        Ok(json) => {
            println!("Unexpected success! JSON output: {}", json);
        }
        Err(e) => {
            println!("Correctly captured error: {}", e);
        }
    }
    
    // 测试空输入
    let empty_source = "";
    println!("Empty Input: {}", empty_source);
    
    match process_zml(empty_source) {
        Ok(json) => {
            println!("Unexpected success! JSON output: {}", json);
        }
        Err(e) => {
            println!("Correctly captured error: {}", e);
        }
    }
    
    println!("\n=== Demo End ===");
}