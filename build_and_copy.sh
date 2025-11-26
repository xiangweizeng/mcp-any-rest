#!/bin/bash

echo "========================================"
echo "Building zml and mcp-any-rest executables"
echo "========================================"

# Build zml executable
echo "Building zml executable..."
cargo build --release --bin zml
if [ $? -ne 0 ]; then
    echo "Error: Failed to build zml executable"
    exit 1
fi

# Build mcp-any-rest executable
echo "Building mcp-any-rest executable..."
cargo build --release --bin mcp-any-rest
if [ $? -ne 0 ]; then
    echo "Error: Failed to build mcp-any-rest executable"
    exit 1
fi

# Create package directory if it doesn't exist
mkdir -p package/config/presets
mkdir -p package/config/zml

# Copy executables to package directory
echo "Copying executables to package directory..."
cp target/release/zml.exe package/zml.exe 2>/dev/null || echo "Warning: zml.exe not found or copy failed"
cp target/release/mcp-any-rest.exe package/mcp-any-rest.exe 2>/dev/null || echo "Warning: mcp-any-rest.exe not found or copy failed"

# Copy configuration files
echo "Copying configuration files..."
cp config/config.json package/config/config.json 2>/dev/null || echo "Warning: config.json not found"
cp config/modules.json package/config/modules.json 2>/dev/null || echo "Warning: modules.json not found"
cp config/mcp-stdio-example.json package/config/mcp-stdio-example.json 2>/dev/null || echo "Warning: mcp-stdio-example.json not found"

# Copy ZML files
echo "Copying ZML files..."
cp config/zml/*.zml package/config/zml/ 2>/dev/null || echo "Warning: No ZML files found or copy failed"

# Copy preset files
echo "Copying preset files..."
cp config/presets/*.json package/config/presets/ 2>/dev/null || echo "Warning: No preset files found or copy failed"

echo "========================================"
echo "Build and copy completed successfully!"
echo "========================================"
echo ""
echo "Executables copied to package directory:"
echo "- package/zml.exe"
echo "- package/mcp-any-rest.exe"
echo ""
echo "Configuration files copied to package/config/"
echo ""
echo "Ready to use!"