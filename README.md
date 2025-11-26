# MCP-ANY-REST

一个适用于任何 REST 后端的通用模型上下文协议 (MCP) 服务器。本项目提供服务编排、认证和动态模块支持。包含一个 ZenTao（禅道）集成作为示例配置。

## ZML 文档

查看 `docs/` 目录下的 ZML 文档索引：

- 索引：`docs/README.md`
- 规范：`docs/ZML_SPECIFICATION.md`
- 快速开始：`docs/ZML_QUICKSTART.md`
- 授权配置：`docs/ZML_AUTHORIZATION.md`

## 特性
- **ZML 配置**：使用零成本模块语言 (ZML) 进行声明式配置
- **令牌认证**：安全的基于令牌的认证
- **服务编排**：通过 ServiceComposer 提供统一接口
- **RMCP 合规**：完全符合 RMCP 协议标准

## 前置要求

- Rust 1.70+ (2021 edition)
- 一个具有 API 访问权限的 REST 服务（支持 ZenTao 示例）
- 环境变量配置

## 安装

1. 克隆仓库：
```bash
git clone <repository-url>
cd mcp-any-rest
```

2. 构建项目：

你可以使用提供的构建脚本同时构建 `zml` 和 `mcp-any-rest` 二进制文件并创建分发包：

- Windows:
```bash
.\build_and_copy.bat
```

- Linux/macOS:
```bash
./build_and_copy.sh
```

或者使用 cargo 手动构建：
```bash
cargo build --release
```

## 使用

### 运行服务器

服务器现在支持灵活的配置目录管理：

- **命令行指定**：使用 `--config-dir` 指定自定义配置目录
- **自动检测**：如果未指定配置目录，服务器将自动使用相对于可执行文件的 `config` 目录

```bash
# 使用自定义配置目录运行
cargo run --bin mcp-any-rest -- --transport http --config-dir /path/to/your/config

# 在 stdio 模式下运行（使用相对于可执行文件的配置目录）
cargo run --bin mcp-any-rest -- --transport stdio

# 在 release 模式下使用自定义配置目录运行
cargo run --release --bin mcp-any-rest -- --transport http --config-dir ./my-config

# 使用默认配置运行（配置目录相对于可执行文件）
./target/release/mcp-any-rest --transport http
```

### ZML 命令行工具

本项目包含一个用于管理 ZML 模块的 `zml` 命令行工具。

```bash
# 列出模块
cargo run --bin zml -- list

# 编译 ZML 为 JSON
cargo run --bin zml -- compile -i config/zml/project.zml --pretty
```

更多详情请参阅 `docs/ZML_QUICKSTART.md`。

### 配置目录结构

配置目录应包含以下文件：
```
config/
├── config.json                  # 服务器运行时配置
├── modules.json                 # 模块启用映射
└── presets/                     # 预配置模块集
    ├── full.json
    ├── index.json
    └── ...
```


## VSCode 扩展

`plugin/vscode-zml` 中提供了一个用于 ZML 语法高亮和格式化的 VSCode 扩展。

安装说明请参阅 `plugin/vscode-zml/README.md`。

## 项目结构

```
config/                         # 运行时配置和 ZML 源码
├── config.json                  # 服务器运行时配置
├── modules.json                 # 模块启用映射
├── presets/                     # 预配置模块集
│   ├── full.json
│   ├── index.json
│   └── project.json
└── zml/                         # ZML 模块定义 (源码)
    ├── project.zml
    ├── user.zml
    └── ...
src/
├── lib.rs                       # 库导出
├── main.rs                      # 主入口点
├── zml/                         # ZML 语言实现
│   ├── mod.rs
│   ├── ast.rs                   # 抽象语法树
│   ├── parser.rs                # ZML 解析器
│   ├── compiler.rs              # ZML 编译器 (ZML → JSON)
│   └── grammar.pest             # PEG 语法
├── services/                    # 服务编排
│   ├── mod.rs
│   └── composer_service/
│       └── module_registry.rs
└── bin/                         # CLI 和测试工具
    ├── zml.rs                   # 统一 ZML CLI (list, compile)
    ├── test_zml.rs              # ZML 编译/集成测试
    └── ...
docs/                            # ZML 规范和文档
├── README.md                    # 文档索引
├── ZML_SPECIFICATION.md         # ZML 语言规范
├── ZML_QUICKSTART.md            # 快速开始指南
└── ZML_AUTHORIZATION.md         # 授权配置
plugin/
├── vscode-zml/                  # ZML 语法的 VSCode 扩展
│   ├── README.md
│   └── syntaxes/zml.tmLanguage.json
└── web-docs/                    # Web 文档
tests/
├── zml_enum_tests.rs
└── zml_integration_test.rs
```

## 开发

### 添加新服务

要使用基于 ZML 的架构添加新服务模块：

1. 在 `config/zml/` 中创建一个新的 ZML 模块定义文件（例如 `config/zml/mymodule.zml`）
2. 使用 ZML 语法定义模块结构、方法和类型
3. 系统将在运行时自动加载并注册该模块
4. 服务编排层无需修改代码

### 测试

运行测试：
```bash
cargo test
```

### 生产构建

```bash
cargo build --release
```

## 许可证

本项目采用 MIT 许可证。

## 贡献

1. Fork 仓库
2. 创建特性分支
3. 提交更改
4. 添加测试
5. 提交 Pull Request

## 支持

如有问题或疑问，请在 GitHub 仓库中提交 issue。
