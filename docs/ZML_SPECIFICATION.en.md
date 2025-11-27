# ZML (Zero-cost Module Language) Specification

ZML is a declarative configuration language designed for REST MCP servers. It aims to replace verbose JSON configurations with a concise, type-safe syntax, providing better readability and maintainability.

## 1. Basic Syntax

### 1.1 Comments
ZML supports single-line and multi-line comments, similar to C/C++/Java/Rust syntax.

```zml
// This is a single-line comment

/*
 * This is
 * a multi-line comment
 */
```

### 1.2 Identifiers
Identifiers consist of letters, numbers, and underscores, and must start with a letter or an underscore.

```zml
user_id
Product123
_internal_var
```

### 1.3 Literals
The following literal types are supported:

- **String**: Enclosed in double quotes, e.g., `"hello world"`
- **Integer**: e.g., `42`, `-10`
- **Float**: e.g., `3.14`, `-0.01`
- **Boolean**: `true`, `false`

## 2. Module

A module is the top-level structure in ZML, used to organize related API definitions.

### 2.1 Definition
Use the `module` keyword to define a module.

```zml
module user {
    // Module content
}
```

### 2.2 Inheritance
Modules can use the `extends` keyword to inherit from another module definition (Note: concrete implementation depends on compiler support).

```zml
module admin_user extends user {
    // ...
}
```

### 2.3 Module Attributes
Attributes can be defined directly within a module. Common attributes include:

- `version`: Module version (string)
- `description`: Module description (string)
- `enabled`: Whether enabled (boolean)
- `access_level`: Access level (public/private/internal)
- `base_url`: Base URL (string)

```zml
module user {
    version: "1.0.0"
    description: "User Management Module"
    enabled: true
}
```

## 3. Type System

ZML provides a rich type system to describe data structures.

### 3.1 Basic Types
| Type | Description |
|------|-------------|
| `string` | String text |
| `integer` | Integer |
| `number` | Floating-point number |
| `boolean` | Boolean value |
| `date` | Date (YYYY-MM-DD) |
| `datetime` | Date and Time (ISO 8601) |
| `any` | Any type |

### 3.2 Composite Types

#### Array
Use `array<Type>` syntax.
```zml
array<string>
array<integer>
array<User>
```

#### Object
Use `object { ... }` syntax to define anonymous object structures.
```zml
object {
    id: integer
    name: string
}
```

#### Inline Enum
Use `enum[...]` syntax.
```zml
enum[male, female]
enum[pending, active, closed]
```

### 3.3 Type Definition
Use the `type` keyword to define reusable data structures.

```zml
type Address {
    street: string
    city: string
    zip_code: string? // Optional field
}

type User {
    id: integer
    username: string
    address: Address  // Reference other types
    tags: array<string> = [] // With default value
}
```

### 3.4 Enum Definition
Use the `enum` keyword to define named enums.

```zml
enum UserStatus {
    active
    inactive
    banned
}

// Enum with values (not fully supported yet, depends on implementation)
enum ErrorCode {
    NotFound = 404
    ServerError = 500
}
```

### 3.5 References
- **Type Reference**: Use the type name directly, or use `ref:TypeName` (usually for explicit referencing).
- **Enum Reference**: `EnumNameValue`.

### 3.6 Field Modifiers
- **Optional**: Add `?` after the type, e.g., `string?`.
- **Default Value**: Use `=` to specify, e.g., `count: integer = 0`.

## 4. Method

Methods define API endpoints.

```zml
method <name> {
    description: <string>
    http_method: <GET|POST|PUT|DELETE|PATCH>
    uri: <string>
    access_level: <public|private|internal>
    rate_limit: <limit>
    
    params { ... }
    response: <type>
}
```

### 4.1 Rate Limit
Supports two formats:
1. **Simple format**: `requests/seconds`
   ```zml
   rate_limit: 100/60 // 100 requests per 60 seconds
   ```
2. **Object format**:
   ```zml
   rate_limit: { requests: 100, per_seconds: 60 }
   ```

### 4.2 Parameters (Params)
Define request parameters in the `params` block (path parameters, query parameters, or body parameters, depending on implementation).

```zml
params {
    userId: integer            // Required parameter
    type: string = "normal"    // With default value
    detail: boolean?           // Optional parameter
}
```

### 4.3 Response
Define the API response structure.

```zml
response: User
// Or
response: object {
    code: integer
    data: array<User>
}
```

## 5. Resource

Resource definitions are used to describe RESTful resource collections.

```zml
resource UserResource {
    uri: "/users"
    description: "User resource collection"
    
    type UserEntity {
        id: integer
        name: string
    }
}
```

## 6. Template

Templates are used to define reusable configuration blocks.

```zml
template CrudMethod {
    access_level: public
    rate_limit: 50/60
}

// Modules can reuse templates through some mechanism (depends on compiler implementation)
```

## 7. Full Example

```zml
module shop {
    version: "1.0.0"
    description: "Online Shop API"
    enabled: true

    // Enum definition
    enum OrderStatus {
        pending
        paid
        shipped
        completed
        cancelled
    }

    // Type definition
    type Product {
        id: integer
        name: string
        price: number
        tags: array<string>
    }

    type Order {
        id: integer
        user_id: integer
        items: array<Product>
        status: OrderStatus
        created_at: datetime
    }

    // Method definition
    method get_products {
        description: "Get product list"
        http_method: GET
        uri: "/products"
        access_level: public
        rate_limit: 100/60

        params {
            page: integer = 1
            limit: integer = 20
            category: string?
        }

        response: object {
            total: integer
            items: array<Product>
        }
    }

    method create_order {
        description: "Create new order"
        http_method: POST
        uri: "/orders"
        access_level: private
        
        params {
            product_ids: array<integer>
            address: string
        }

        response: Order
    }
}
```
