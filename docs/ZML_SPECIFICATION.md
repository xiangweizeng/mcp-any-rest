# ZML (Zero-cost Module Language) 语言规范

ZML 是一种专为 REST MCP 服务器设计的声明式配置语言。它旨在通过简洁、类型安全的语法来替代冗长的 JSON 配置，提供更好的可读性和可维护性。

## 1. 基础语法

### 1.1 注释
ZML 支持单行注释和多行注释，语法与 C/C++/Java/Rust 类似。

```zml
// 这是单行注释

/*
 * 这是
 * 多行注释
 */
```

### 1.2 标识符 (Identifiers)
标识符由字母、数字和下划线组成，必须以字母或下划线开头。

```zml
user_id
Product123
_internal_var
```

### 1.3 字面量 (Literals)
支持以下类型的字面量：

- **字符串**: 双引号包围，如 `"hello world"`
- **整数**: 如 `42`, `-10`
- **浮点数**: 如 `3.14`, `-0.01`
- **布尔值**: `true`, `false`

## 2. 模块 (Module)

模块是 ZML 的顶层结构，用于组织相关的 API 定义。

### 2.1 定义
使用 `module` 关键字定义模块。

```zml
module user {
    // 模块内容
}
```

### 2.2 继承 (Inheritance)
模块可以使用 `extends` 关键字继承另一个模块的定义（注意：具体实现依赖于编译器支持）。

```zml
module admin_user extends user {
    // ...
}
```

### 2.3 模块属性
模块内部可以直接定义属性，常见属性包括：

- `version`: 模块版本 (string)
- `description`: 模块描述 (string)
- `enabled`: 是否启用 (boolean)
- `access_level`: 访问级别 (public/private/internal)
- `base_url`: 基础 URL (string)

```zml
module user {
    version: "1.0.0"
    description: "用户管理模块"
    enabled: true
}
```

## 3. 类型系统 (Type System)

ZML 提供了丰富的类型系统来描述数据结构。

### 3.1 基础类型
| 类型 | 说明 |
|------|------|
| `string` | 字符串文本 |
| `integer` | 整数 |
| `number` | 浮点数 |
| `boolean` | 布尔值 |
| `date` | 日期 (YYYY-MM-DD) |
| `datetime` | 日期时间 (ISO 8601) |
| `any` | 任意类型 |

### 3.2 复合类型

#### 数组 (Array)
使用 `array<Type>` 语法。
```zml
array<string>
array<integer>
array<User>
```

#### 对象 (Object)
使用 `object { ... }` 语法定义匿名对象结构。
```zml
object {
    id: integer
    name: string
}
```

#### 内联枚举 (Inline Enum)
使用 `enum[...]` 语法。
```zml
enum[male, female]
enum[pending, active, closed]
```

### 3.3 类型定义 (Type Definition)
使用 `type` 关键字定义复用的数据结构。

```zml
type Address {
    street: string
    city: string
    zip_code: string? // 可选字段
}

type User {
    id: integer
    username: string
    address: Address  // 引用其他类型
    tags: array<string> = [] // 带默认值
}
```

### 3.4 枚举定义 (Enum Definition)
使用 `enum` 关键字定义命名枚举。

```zml
enum UserStatus {
    active
    inactive
    banned
}

// 带值的枚举（暂未完全支持，视具体实现而定）
enum ErrorCode {
    NotFound = 404
    ServerError = 500
}
```

### 3.5 引用 (Reference)
- **类型引用**: 直接使用类型名称，或使用 `ref:TypeName`（通常用于明确引用）。
- **枚举引用**: `EnumNameValue`。

### 3.6 字段修饰符
- **可选**: 在类型后加 `?`，如 `string?`。
- **默认值**: 使用 `=` 指定，如 `count: integer = 0`。

## 4. 方法 (Method)

方法定义了 API 的端点。

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

### 4.1 速率限制 (Rate Limit)
支持两种格式：
1. **简单格式**: `请求数/秒数`
   ```zml
   rate_limit: 100/60 // 60秒内100次请求
   ```
2. **对象格式**:
   ```zml
   rate_limit: { requests: 100, per_seconds: 60 }
   ```

### 4.2 参数 (Params)
在 `params` 块中定义请求参数（路径参数、查询参数或请求体参数，具体取决于实现）。

```zml
params {
    userId: integer            // 必需参数
    type: string = "normal"    // 带默认值
    detail: boolean?           // 可选参数
}
```

### 4.3 响应 (Response)
定义 API 的响应结构。

```zml
response: User
// 或者
response: object {
    code: integer
    data: array<User>
}
```

## 5. 资源 (Resource)

资源定义用于描述 RESTful 资源集合。

```zml
resource UserResource {
    uri: "/users"
    description: "用户资源集合"
    
    type UserEntity {
        id: integer
        name: string
    }
}
```

## 6. 模板 (Template)

模板用于定义可复用的配置块。

```zml
template CrudMethod {
    access_level: public
    rate_limit: 50/60
}

// 模块中可以通过某种机制复用模板（具体取决于编译器实现）
```

## 7. 完整示例

```zml
module shop {
    version: "1.0.0"
    description: "在线商店 API"
    enabled: true

    // 枚举定义
    enum OrderStatus {
        pending
        paid
        shipped
        completed
        cancelled
    }

    // 类型定义
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

    // 方法定义
    method get_products {
        description: "获取产品列表"
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
        description: "创建新订单"
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
