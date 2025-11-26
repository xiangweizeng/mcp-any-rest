---
title: 快速上手
icon: play
article: false
footer: true
---

# 快速上手

本指南将帮助您快速安装并运行 MCP-ANY-REST。

## 前置要求

- Rust 1.70+ (2021 edition)
- 一个可访问的 REST 服务 (例如 ZenTao)
- 配置环境变量

## 安装

1. 克隆仓库:
   ```bash
   git clone <repository-url>
   cd mcp-any-rest
   ```

2. 构建项目:
   ```bash
   cargo build --release
   ```

## 运行服务器

服务器支持灵活的配置目录管理：

- **命令行指定**: 使用 `--config-dir` 指定自定义配置目录
- **自动检测**: 如果未指定，服务器将自动使用相对于可执行文件的 `config` 目录

```bash
# 使用自定义配置目录运行
cargo run --bin mcp-any-rest -- --transport http --config-dir /path/to/your/config

# 在 stdio 模式下运行 (使用可执行文件相对路径的 config 目录)
cargo run --bin mcp-any-rest -- --transport stdio

# 在 release 模式下使用自定义配置目录运行
cargo run --release --bin mcp-any-rest -- --transport http --config-dir ./my-config

# 使用默认配置运行 (配置目录相对于可执行文件)
./target/release/mcp-any-rest --transport http
```

## ZML 工具使用

项目提供了 `zml` 工具，用于管理 ZML 模块。

### 常用命令

- **列出模块** (使用默认配置目录):
  ```bash
  cargo run --bin zml -- list
  ```

- **编译模块** (从文件读取并输出 JSON):
  ```bash
  cargo run --bin zml -- compile -i config/zml/project.zml > project.json
  ```

- **美化输出**:
  ```bash
  cargo run --bin zml -- compile -i config/zml/project.zml --pretty
  ```

### 示例：定义并编译模块

创建一个简单的 `project.zml` 文件：

```zml
module project {
    version: "1.0.0"
    description: "Project service"
    enabled: true
    access_level: public

    type project_user {
        id: integer
        account: string
    }

    method get_project_list {
        description: "Get project list"
        http_method: GET
        uri: "projects"
        
        params {
            page: integer = 1
            limit: integer = 20
        }

        response: object {
            page: integer
            total: integer
        }
    }
}
```

编译该模块：
```bash
cargo run --bin zml -- compile -i config/zml/project.zml --pretty
```
