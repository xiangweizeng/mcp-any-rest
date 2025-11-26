---
title: Auth Configuration
icon: lock
article: false
footer: true
---

# Auth Configuration

This document provides examples of common authentication configurations in the mcp-any-rest project and their HTTP data effects.

## Table of Contents

1. [Direct Authentication](#direct-authentication)
   - [Token Authentication](#token-authentication)
   - [Bearer Token Authentication](#bearer-token-authentication)
   - [Basic Authentication](#basic-authentication)
   - [API Key Authentication](#api-key-authentication)
   - [Custom Header Authentication](#custom-header-authentication)
2. [Login Based Authentication](#login-based-authentication)
   - [JSON Form Login](#json-form-login)
   - [Form Data Login](#form-data-login)
   - [OAuth2 Login](#oauth2-login)
3. [Advanced Configuration](#advanced-configuration)
   - [Multiple Token Extraction](#multiple-token-extraction)
   - [Token Refresh Configuration](#token-refresh-configuration)

## Direct Authentication

Direct authentication uses pre-configured credentials that are directly included in every request.

### Token Authentication

This is the simplest form of authentication, where the token is directly included in the request.

#### Configuration

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

#### HTTP Data Effect

When making a request with this configuration, the following HTTP header will be added:

```http
Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
```

#### Request Example

```http
GET /api/users HTTP/1.1
Host: example.com
Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
Content-Type: application/json
```

### Bearer Token Authentication

Bearer Token authentication is similar to Token authentication, but follows the standard Bearer Token format.

#### Configuration

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

#### HTTP Data Effect

When making a request with this configuration, the following HTTP header will be added:

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
```

#### Request Example

```http
GET /api/users HTTP/1.1
Host: example.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
```
