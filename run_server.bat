@echo off
cd /d d:\project\mcp-any-rest

echo Starting mcp-any-rest...
cargo run --bin mcp-any-rest -- --transport http --config-dir config/