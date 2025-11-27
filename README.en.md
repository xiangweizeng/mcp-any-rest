# MCP-ANY-REST

## Project Vision

In today's rapid AI development, Large Language Models (LLMs) show amazing capabilities, but securely and efficiently connecting them to massive existing business systems (REST APIs) remains a challenge.

**MCP-ANY-REST** was born for this. Our mission is to **bridge the gap between LLMs and business data**.

We are not just building a tool, but a bridge—a bridge that allows AI to understand and manipulate real-world data. By following the Model Context Protocol (MCP) standard, we enable any RESTful service to join the AI ecosystem with zero cost.

## Core Value

::: tip Write Once, Connect Anywhere
No need to repeat development for every AI model. Configure once, run everywhere.
:::

## Tech Stack

This project is built on a modern tech stack ensuring high performance and maintainability:

| Domain | Technology |
| :--- | :--- |
| **Core Logic** | Rust |
| **Protocol Standard** | Model Context Protocol (MCP) |
| **Config Language** | ZML |
| **Doc Site** | VuePress + Theme Hope |

## ZML Documentation

See ZML docs index in `docs/`:

- Index: `docs/README.md`
- Specification: `docs/ZML_SPECIFICATION.md`
- Quickstart: `docs/ZML_QUICKSTART.md`
- Authorization: `docs/ZML_AUTHORIZATION.md`

## Features
- **ZML Configuration**: Declarative configuration using Zero-cost Module Language (ZML)
- **Token Authentication**: Secure token-based authentication
- **Service Composition**: Unified interface through ServiceComposer
- **RMCP Compliance**: Full compliance with RMCP protocol standards

## Prerequisites

- Rust 1.70+ (2021 edition)
- A REST service with API access (ZenTao example supported)
- Environment variables configuration

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd mcp-any-rest
```

2. Build the project:

You can use the provided build scripts to build both `zml` and `mcp-any-rest` binaries and create a distribution package:

- Windows:
```bash
.\build_and_copy.bat
```

- Linux/macOS:
```bash
./build_and_copy.sh
```

Or build manually using cargo:
```bash
cargo build --release
```

## Usage

### Running the Server

The server now supports flexible configuration directory management:

- **Command line specified**: Use `--config-dir` to specify a custom configuration directory
- **Automatic detection**: If no config directory is specified, the server will automatically use the `config` directory relative to the executable

```bash
# Run with custom configuration directory
cargo run --bin mcp-any-rest -- --transport http --config-dir /path/to/your/config

# Run in stdio mode (uses executable-relative config directory)
cargo run --bin mcp-any-rest -- --transport stdio

# Run in release mode with custom config directory
cargo run --release --bin mcp-any-rest -- --transport http --config-dir ./my-config

# Run with default configuration (config directory relative to executable)
./target/release/mcp-any-rest --transport http
```

### ZML CLI Tool

The project includes a `zml` CLI tool for managing ZML modules.

```bash
# List modules
cargo run --bin zml -- list

# Compile ZML to JSON
cargo run --bin zml -- compile -i config/zml/project.zml --pretty
```

See `docs/ZML_QUICKSTART.md` for more details.

### Configuration Directory Structure

The configuration directory should contain the following files:
```
config/
├── config.json                  # Server runtime configuration
├── modules.json                 # Module enablement map
└── presets/                     # Preconfigured module sets
    ├── full.json
    ├── index.json
    └── ...
```


## VSCode Extension

A VSCode extension for ZML syntax highlighting and formatting is available in `plugin/vscode-zml`.

See `plugin/vscode-zml/README.md` for installation instructions.

## Project Structure

```
config/                         # Runtime configuration and ZML sources
├── config.json                  # Server runtime configuration
├── modules.json                 # Module enablement map
├── presets/                     # Preconfigured module sets
│   ├── full.json
│   ├── index.json
│   └── project.json
└── zml/                         # ZML module definitions (source)
    ├── project.zml
    ├── user.zml
    └── ...
src/
├── lib.rs                       # Library exports
├── main.rs                      # Main entry point
├── zml/                         # ZML language implementation
│   ├── mod.rs
│   ├── ast.rs                   # Abstract syntax tree
│   ├── parser.rs                # ZML parser
│   ├── compiler.rs              # ZML compiler (ZML → JSON)
│   └── grammar.pest             # PEG grammar
├── services/                    # Service composition
│   ├── mod.rs
│   └── composer_service/
│       └── module_registry.rs
└── bin/                         # CLI and test utilities
    ├── zml.rs                   # Unified ZML CLI (list, compile)
    ├── test_zml.rs              # ZML compile/integration tests
    └── ...
docs/                            # ZML specification and documentation
├── README.md                    # Docs index
├── ZML_SPECIFICATION.md         # ZML Language Specification
├── ZML_QUICKSTART.md            # Quickstart Guide
└── ZML_AUTHORIZATION.md         # Authorization Configuration
plugin/
├── vscode-zml/                  # VSCode extension for ZML syntax
│   ├── README.md
│   └── syntaxes/zml.tmLanguage.json
└── web-docs/                    # Web documentation
tests/
├── zml_enum_tests.rs
└── zml_integration_test.rs
```

## Development

### Adding New Services

To add a new service module using the ZML-based architecture:

1. Create a new ZML module definition file in `config/zml/` (e.g., `config/zml/mymodule.zml`)
2. Define the module structure, methods, and types using ZML syntax
3. The system will automatically load and register the module at runtime
4. No code changes are required in the service composition layer

### Testing

Run tests with:
```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

## License

This project is licensed under the MIT License.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Support

For issues and questions, please open an issue on the GitHub repository.