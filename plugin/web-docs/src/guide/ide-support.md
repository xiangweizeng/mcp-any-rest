---
title: IDE 支持
icon: tools
article: false
footer: true
---

# IDE 支持

本项目提供了 VSCode 插件 `vscode-zml`，支持 ZML 语言的语法高亮和代码格式化，极大地提升了编写 ZML 配置的体验。

## 功能特性

- **语法高亮**: 支持关键字、内置类型、HTTP 方法、注释、字符串等的高亮显示。
- **代码格式化**: 支持自动缩进、智能逗号处理、可配置的行宽和缩进大小。
- **自动闭合**: 支持括号和引号的自动闭合。

## 安装与使用

插件源码位于 `plugin/vscode-zml` 目录下。

1. **构建插件**:
   进入插件目录并安装依赖：
   ```bash
   cd plugin/vscode-zml
   npm install
   npx @vscode/vsce package
   ```
   这将生成一个 `.vsix` 扩展文件。

2. **安装插件**:
   在 VSCode 中打开命令面板 (Windows/Linux: `Ctrl+Shift+P`, macOS: `Cmd+Shift+P`)，输入并选择 `Extensions: Install from VSIX...`，然后选择上一步生成的 `.vsix` 文件。

3. **开发模式运行**:
   如果你想调试插件，可以使用以下命令以开发模式启动 VSCode：
   ```bash
   code --extensionDevelopmentPath ./plugin/vscode-zml
   ```

安装完成后，打开任意 `.zml` 文件即可自动激活插件，享受语法高亮和格式化功能。
