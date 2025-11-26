---
title: Quick Start
icon: play
article: false
footer: true
---

# Quick Start

This guide will help you quickly install and run MCP-ANY-REST.

## Prerequisites

- Rust 1.70+ (2021 edition)
- An accessible REST service (e.g. ZenTao)
- Configured environment variables

## Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd mcp-any-rest
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Running the Server

The server supports flexible configuration directory management:

- **Command Line Specification**: Use `--config-dir` to specify a custom configuration directory
- **Auto Detection**: If not specified, the server will automatically use the `config` directory relative to the executable

```bash
# Run with custom configuration directory
cargo run --bin mcp-any-rest -- --transport http --config-dir /path/to/your/config

# Run in stdio mode (using config directory relative to executable)
cargo run --bin mcp-any-rest -- --transport stdio

# Run in release mode with custom configuration directory
cargo run --release --bin mcp-any-rest -- --transport http --config-dir ./my-config

# Run with default configuration (config directory relative to executable)
./target/release/mcp-any-rest --transport http
```

## ZML Tool Usage

The project provides a `zml` tool for managing ZML modules.

### Common Commands

- **List Modules** (using default configuration directory):
  ```bash
  cargo run --bin zml -- list
  ```

- **Compile Module** (read from file and output JSON):
  ```bash
  cargo run --bin zml -- compile -i config/zml/project.zml > project.json
  ```

- **Pretty Print Output**:
  ```bash
  cargo run --bin zml -- compile -i config/zml/project.zml --pretty
  ```

### Example: Defining and Compiling a Module

Create a simple `project.zml` file:

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

Compile the module:
```bash
cargo run --bin zml -- compile -i config/zml/project.zml --pretty
```
