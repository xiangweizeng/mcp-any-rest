---
title: ZML Language Specification
icon: book
article: false
index: true
footer: true
---

# ZML Language Specification

ZML (Zero-cost Module Language) is a declarative configuration language designed for REST MCP servers. It aims to replace verbose JSON configuration with concise, type-safe syntax, providing better readability and maintainability.

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
Identifiers consist of letters, numbers, and underscores, and must start with a letter or underscore.

```zml
user_id
Product123
_internal_var
```

### 1.3 Literals
The following types of literals are supported:

- **String**: Enclosed in double quotes, e.g., `"hello world"`
- **Integer**: e.g., `42`, `-10`
- **Float**: e.g., `3.14`, `-0.01`
- **Boolean**: `true`, `false`

## 2. Module

Module is the top-level structure in ZML, used to organize related API definitions.

### 2.1 Definition
Use the `module` keyword to define a module.

```zml
module user {
    // Module content
}
```

### 2.2 Module Attributes
Attributes can be defined directly inside a module. Common attributes include:

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
| `number` | Floating point number |
| `boolean` | Boolean value |
| `date` | Date (YYYY-MM-DD) |
| `datetime` | Date time (ISO 8601) |
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
