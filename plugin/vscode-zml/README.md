# ZML Syntax Highlighting and Formatting (VSCode Extension)

Provides syntax highlighting, language configuration, and formatting support for ZenTao Module Language (ZML).

## Installation

1. Open VSCode command palette and choose `Developer: Install Extension from VSIX`.
2. Package this folder as VSIX or run VSCode in extension development mode:

```bash
code --extensionDevelopmentPath ./vscode-zml
```

## Building

1. Install dependencies:

```bash
npm install -D @vscode/vsce@latest 
```

2. Build the extension:

```bash
npx @vscode/vsce package 
```

## Features

### Syntax Highlighting
- Keywords and declarations highlighting
- Built-in types and operators
- HTTP methods, access levels, rate-limit pattern
- Enum member references (Type.VALUE)
- Comments, strings, numbers, booleans
- Bracket/quote auto-closing based on language configuration

### Code Formatting
- Automatic indentation based on braces and blocks
- Configurable indent size (default: 2 spaces)
- Line length wrapping (configurable, default: 120 characters)
- Smart comma handling with automatic spacing
- On-type formatting for new lines and braces
- Support for comments (line and block)
- Formatting for arrays, objects, and method declarations

## Files

- `package.json`: registers language and grammar
- `language-configuration.json`: comments and brackets
- `syntaxes/zml.tmLanguage.json`: TextMate grammar

## Notes

- Associate `*.zml` by default; it will activate automatically when opening `.zml` files.