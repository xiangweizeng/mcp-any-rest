---
title: Guide
icon: book
article: false
index: true
footer: true
---

# Guide

Welcome to the MCP-ANY-REST official documentation.

## Table of Contents

- [Quick Start](./quickstart.md): Quickly install and run MCP-ANY-REST.
- [ZML Specification](./specification.md): Learn about ZML language syntax and features.
- [Auth Configuration Examples](./auth-examples.md): Configuration examples for various authentication scenarios, including direct auth, login auth, etc.
- [IDE Support](./ide-support.md): VSCode extension installation and usage guide.

## Project Structure

```
config/                         # Runtime config and ZML source
├── config.json                  # Server runtime config
├── modules.json                 # Module enablement mapping
├── presets/                     # Pre-configured module sets
└── zml/                         # ZML module definitions
    ├── project.zml
    ├── user.zml
    └── ...
src/
├── lib.rs                       # Library exports
├── main.rs                      # Main entry point
├── zml/                         # ZML language implementation
├── services/                    # Service orchestration
└── bin/                         # CLI and test tools
```
