---
title: IDE Support
icon: tools
article: false
footer: true
---

# IDE Support

This project provides a VSCode extension `vscode-zml`, which supports ZML language syntax highlighting and code formatting, greatly improving the experience of writing ZML configurations.

## Features

- **Syntax Highlighting**: Supports highlighting of keywords, built-in types, HTTP methods, comments, strings, etc.
- **Code Formatting**: Supports automatic indentation, smart comma handling, configurable line width and indentation size.
- **Auto Closing**: Supports automatic closing of brackets and quotes.

## Installation and Usage

The extension source code is located in the `plugin/vscode-zml` directory.

1. **Build the Extension**:
   Enter the plugin directory and install dependencies:
   ```bash
   cd plugin/vscode-zml
   npm install
   npx @vscode/vsce package
   ```
   This will generate a `.vsix` extension file.

2. **Install the Extension**:
   Open the Command Palette in VSCode (Windows/Linux: `Ctrl+Shift+P`, macOS: `Cmd+Shift+P`), type and select `Extensions: Install from VSIX...`, then select the `.vsix` file generated in the previous step.

3. **Run in Development Mode**:
   If you want to debug the extension, you can launch VSCode in development mode using the following command:
   ```bash
   code --extensionDevelopmentPath ./plugin/vscode-zml
   ```

Once installed, open any `.zml` file to automatically activate the extension and enjoy syntax highlighting and formatting features.
