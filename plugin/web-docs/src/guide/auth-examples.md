---
title: 认证配置
icon: lock
article: false
footer: true
---

# 认证配置

本文档提供了 mcp-any-rest 项目中常见的认证配置示例及其产生的 HTTP 数据效果。

## 目录

1. [直接认证](#直接认证)
   - [令牌认证](#令牌认证)
   - [Bearer 令牌认证](#bearer-令牌认证)
   - [基础认证](#基础认证)
   - [API 密钥认证](#api-密钥认证)
   - [自定义头认证](#自定义头认证)
2. [基于登录的认证](#基于登录的认证)
   - [JSON 表单登录](#json-表单登录)
   - [表单数据登录](#表单数据登录)
   - [OAuth2 登录](#oauth2-登录)
3. [高级配置](#高级配置)
   - [多重令牌提取](#多重令牌提取)
   - [令牌刷新配置](#令牌刷新配置)

## 直接认证

直接认证使用预配置的认证信息，这些信息直接包含在每个请求中。

### 令牌认证

这是最简单的认证形式，令牌直接包含在请求中。

#### 配置

```json
{
  "mode": "direct",
  "direct_config": {
    "auth_type": "token",
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ"
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### HTTP 数据效果

使用此配置发出请求时，将添加以下 HTTP 头：

```http
Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
```

#### 请求示例

```http
GET /api/users HTTP/1.1
Host: example.com
Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
Content-Type: application/json
```

### Bearer 令牌认证

Bearer 令牌认证与令牌认证类似，但遵循标准的 Bearer 令牌格式。

#### 配置

```json
{
  "mode": "direct",
  "direct_config": {
    "auth_type": "bearer",
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ"
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### HTTP 数据效果

使用此配置发出请求时，将添加以下 HTTP 头：

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
```

#### 请求示例

```http
GET /api/users HTTP/1.1
Host: example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
Content-Type: application/json
```

### 基础认证

基础认证使用 Base64 编码的用户名和密码。

#### 配置

```json
{
  "mode": "direct",
  "direct_config": {
    "auth_type": "basic",
    "username": "admin",
    "password": "secret123"
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### HTTP 数据效果

使用此配置发出请求时，将添加以下 HTTP 头：

```http
Authorization: Basic YWRtaW46c2VjcmV0MTIz
```

注意：`YWRtaW46c2VjcmV0MTIz` 是 `admin:secret123` 的 Base64 编码。

#### 请求示例

```http
GET /api/users HTTP/1.1
Host: example.com
Authorization: Basic YWRtaW46c2VjcmV0MTIz
Content-Type: application/json
```

### API 密钥认证

API 密钥认证使用自定义头发送 API 密钥。

#### 配置

```json
{
  "mode": "direct",
  "direct_config": {
    "auth_type": "apikey",
    "api_key_name": "X-API-Key",
    "token": "abc123def456ghi789"
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### HTTP 数据效果

使用此配置发出请求时，将添加以下 HTTP 头：

```http
X-API-Key: abc123def456ghi789
```

#### 请求示例

```http
GET /api/users HTTP/1.1
Host: example.com
X-API-Key: abc123def456ghi789
Content-Type: application/json
```

### 自定义头认证

自定义头认证允许您指定多个自定义头用于认证。

#### 配置

```json
{
  "mode": "direct",
  "direct_config": {
    "auth_type": "customheaders",
    "custom_headers": {
      "Authorization": "Bearer my-custom-token",
      "X-Client-ID": "my-client-id",
      "X-Request-ID": "unique-request-id"
    }
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### HTTP 数据效果

使用此配置发出请求时，将添加以下 HTTP 头：

```http
Authorization: Bearer my-custom-token
X-Client-ID: my-client-id
X-Request-ID: unique-request-id
```

#### 请求示例

```http
GET /api/users HTTP/1.1
Host: example.com
Authorization: Bearer my-custom-token
X-Client-ID: my-client-id
X-Request-ID: unique-request-id
Content-Type: application/json
```

## 基于登录的认证

基于登录的认证首先执行登录请求以获取认证令牌，然后在后续请求中使用这些令牌。

### JSON 表单登录

此方法使用 JSON 负载登录并从响应中提取令牌。

#### 配置

```json
{
  "mode": "login",
  "login_config": {
    "auth_type": "json",
    "url": "https://api.example.com/auth/login",
    "method": "POST",
    "headers": {
      "Content-Type": "application/json"
    },
    "body": {
      "format": "json",
      "content": {
        "username": "admin",
        "password": "secret123"
      }
    },
    "response_format": "json",
    "token_extraction": {
      "tokens": [
        {
          "source_location": "body",
          "source_key": "token",
          "format": "bearer",
          "target_location": "header",
          "target_key": "Authorization"
        }
      ]
    }
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### 登录请求

系统将首先发送此请求进行认证：

```http
POST /auth/login HTTP/1.1
Host: api.example.com
Content-Type: application/json

{
  "username": "admin",
  "password": "secret123"
}
```

#### 登录响应

假设服务器响应如下：

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ",
  "expires_in": 3600,
  "user": {
    "id": 1,
    "name": "Admin User"
  }
}
```

#### 后续请求的 HTTP 数据效果

登录成功后，后续请求将包含：

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
```

#### 请求示例

```http
GET /api/users HTTP/1.1
Host: api.example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
Content-Type: application/json
```

### 表单数据登录

此方法使用表单数据登录并从响应中提取令牌。

#### 配置

```json
{
  "mode": "login",
  "login_config": {
    "auth_type": "form",
    "url": "https://api.example.com/auth/login",
    "method": "POST",
    "headers": {
      "Content-Type": "application/x-www-form-urlencoded"
    },
    "body": {
      "format": "form",
      "content": {
        "username": "admin",
        "password": "secret123"
      }
    },
    "response_format": "json",
    "token_extraction": {
      "tokens": [
        {
          "source_location": "body",
          "source_key": "access_token",
          "format": "bearer",
          "target_location": "header",
          "target_key": "Authorization"
        }
      ]
    }
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### 登录请求

系统将首先发送此请求进行认证：

```http
POST /auth/login HTTP/1.1
Host: api.example.com
Content-Type: application/x-www-form-urlencoded

username=admin&password=secret123
```

#### 登录响应

假设服务器响应如下：

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

#### 后续请求的 HTTP 数据效果

登录成功后，后续请求将包含：

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
```

### OAuth2 登录

此方法使用 OAuth2 流程进行认证。

#### 配置

```json
{
  "mode": "login",
  "login_config": {
    "auth_type": "oauth2",
    "url": "https://oauth.example.com/token",
    "method": "POST",
    "headers": {
      "Content-Type": "application/x-www-form-urlencoded",
      "Authorization": "Basic base64(client_id:client_secret)"
    },
    "body": {
      "format": "form",
      "content": {
        "grant_type": "client_credentials",
        "scope": "api:read api:write"
      }
    },
    "response_format": "json",
    "token_extraction": {
      "tokens": [
        {
          "source_location": "body",
          "source_key": "access_token",
          "format": "bearer",
          "target_location": "header",
          "target_key": "Authorization"
        }
      ]
    }
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### 登录请求

系统将首先发送此请求进行认证：

```http
POST /token HTTP/1.1
Host: oauth.example.com
Content-Type: application/x-www-form-urlencoded
Authorization: Basic base64(client_id:client_secret)

grant_type=client_credentials&scope=api:read+api:write
```

#### 登录响应

假设服务器响应如下：

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "api:read api:write"
}
```

#### 后续请求的 HTTP 数据效果

登录成功后，后续请求将包含：

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
```

## 高级配置

### 多重令牌提取

此配置从单个登录响应中提取多个令牌。

#### 配置

```json
{
  "mode": "login",
  "login_config": {
    "auth_type": "json",
    "url": "https://api.example.com/auth/login",
    "method": "POST",
    "headers": {
      "Content-Type": "application/json"
    },
    "body": {
      "format": "json",
      "content": {
        "username": "admin",
        "password": "secret123"
      }
    },
    "response_format": "json",
    "token_extraction": {
      "tokens": [
        {
          "source_location": "body",
          "source_key": "access_token",
          "format": "bearer",
          "target_location": "header",
          "target_key": "Authorization"
        },
        {
          "source_location": "body",
          "source_key": "refresh_token",
          "format": "raw",
          "target_location": "header",
          "target_key": "X-Refresh-Token"
        },
        {
          "source_location": "header",
          "source_key": "X-CSRF-Token",
          "format": "raw",
          "target_location": "header",
          "target_key": "X-CSRF-Token"
        }
      ]
    }
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### 登录响应

假设服务器响应如下：

```http
HTTP/1.1 200 OK
Content-Type: application/json
X-CSRF-Token: csrf-token-12345

{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ",
  "refresh_token": "def50200e3b4...",
  "expires_in": 3600
}
```

#### 后续请求的 HTTP 数据效果

登录成功后，后续请求将包含：

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
X-Refresh-Token: def50200e3b4...
X-CSRF-Token: csrf-token-12345
```

### 令牌刷新配置

此配置包含一个单独的刷新端点用于令牌更新。

#### 配置

```json
{
  "mode": "login",
  "login_config": {
    "auth_type": "json",
    "url": "https://api.example.com/auth/login",
    "method": "POST",
    "headers": {
      "Content-Type": "application/json"
    },
    "body": {
      "format": "json",
      "content": {
        "username": "admin",
        "password": "secret123"
      }
    },
    "response_format": "json",
    "token_extraction": {
      "tokens": [
        {
          "source_location": "body",
          "source_key": "access_token",
          "format": "bearer",
          "target_location": "header",
          "target_key": "Authorization"
        },
        {
          "source_location": "body",
          "source_key": "refresh_token",
          "format": "raw",
          "target_location": "header",
          "target_key": "X-Refresh-Token"
        }
      ]
    },
    "refresh_url": "https://api.example.com/auth/refresh",
    "refresh_method": "POST"
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### 登录请求

系统将首先发送此请求进行认证：

```http
POST /auth/login HTTP/1.1
Host: api.example.com
Content-Type: application/json

{
  "username": "admin",
  "password": "secret123"
}
```

#### 登录响应

假设服务器响应如下：

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ",
  "refresh_token": "def50200e3b4...",
  "expires_in": 3600
}
```

#### 刷新请求（当令牌即将过期时）

当访问令牌即将过期时，系统将发送刷新请求：

```http
POST /auth/refresh HTTP/1.1
Host: api.example.com
Content-Type: application/json
X-Refresh-Token: def50200e3b4...

{
  "refresh_token": "def50200e3b4..."
}
```

#### 刷新响应

假设服务器响应如下：

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ",
  "refresh_token": "def50200e3b4...",
  "expires_in": 3600
}
```

#### 后续请求的 HTTP 数据效果

刷新成功后，后续请求将包含：

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
X-Refresh-Token: def50200e3b4...
```

## 实际示例：禅道 API 认证

这是一个如何为禅道 API 配置认证的实际示例。

#### 配置

```json
{
  "mode": "login",
  "login_config": {
    "auth_type": "json",
    "url": "https://zentao.example.com/api.php/v1/tokens",
    "method": "POST",
    "headers": {
      "Content-Type": "application/json"
    },
    "body": {
      "format": "json",
      "content": {
        "account": "admin",
        "password": "your-password"
      }
    },
    "response_format": "json",
    "token_extraction": {
      "tokens": [
        {
          "source_location": "body",
          "source_key": "token",
          "format": "token",
          "target_location": "header",
          "target_key": "Token"
        }
      ]
    }
  },
  "token_expiry": 3600,
  "refresh_buffer": 300,
  "max_retry_attempts": 3
}
```

#### 登录请求

系统将首先发送此请求进行认证：

```http
POST /api.php/v1/tokens HTTP/1.1
Host: zentao.example.com
Content-Type: application/json

{
  "account": "admin",
  "password": "your-password"
}
```

#### 登录响应

假设服务器响应如下：

```json
{
  "token": "1a2b3c4d5e6f7g8h9i0j",
  "user": {
    "id": 1,
    "account": "admin",
    "realname": "Administrator",
    "role": "admin"
  }
}
```

#### 后续请求的 HTTP 数据效果

登录成功后，后续请求将包含：

```http
Token: 1a2b3c4d5e6f7g8h9i0j
```

#### 请求示例

```http
GET /api.php/v1/users HTTP/1.1
Host: zentao.example.com
Token: 1a2b3c4d5e6f7g8h9i0j
Content-Type: application/json
```

## 结论

本文档涵盖了 mcp-any-rest 项目中的各种认证配置及其对 HTTP 请求的影响。认证系统非常灵活，可以适应几乎所有使用标准认证方法的 REST API。

有关认证系统实现的更多信息，请参考 `src/services/auth_service` 目录中的源代码。
