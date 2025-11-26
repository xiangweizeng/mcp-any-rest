#!/bin/bash
cd /d/project/mcp-any-rest

# Kill any existing mcp-any-rest processes
echo "Stopping existing mcp-any-rest processes..."
pkill -f "mcp-any-rest" || true
sleep 2

# Wait for port to be free
echo "Waiting for port 8081 to be free..."
while lsof -i :8081 > /dev/null 2>&1; do
    echo "Port 8081 still in use, waiting..."
    sleep 1
done

echo "Starting mcp-any-rest..."
cargo run --bin mcp-any-rest -- --transport http --config-dir  config/