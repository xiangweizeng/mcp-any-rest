---
title: 指南
icon: book
article: false
index: true
footer: true
---

# 指南

欢迎阅读 MCP-ANY-REST 官方文档。

## 目录

- [快速上手](./quickstart.md): 快速安装和运行 MCP-ANY-REST。
- [ZML 规范](./specification.md): 了解 ZML 语言的语法和特性。
- [认证配置示例](./auth-examples.md): 各种认证场景的配置示例，包括直接认证、登录认证等。
- [IDE 支持](./ide-support.md): VSCode 插件安装与使用指南。

## 项目结构

```
config/                         # 运行时配置和 ZML 源码
├── config.json                  # 服务器运行时配置
├── modules.json                 # 模块启用映射
├── presets/                     # 预配置模块集
└── zml/                         # ZML 模块定义
    ├── project.zml
    ├── user.zml
    └── ...
src/
├── lib.rs                       # 库导出
├── main.rs                      # 主入口点
├── zml/                         # ZML 语言实现
├── services/                    # 服务编排
└── bin/                         # CLI 和测试工具
```
