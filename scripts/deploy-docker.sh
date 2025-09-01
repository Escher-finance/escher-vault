#!/bin/bash

# Docker-based Nix Deployment Script for CW4626 Vault
set -e

echo "🐳 Docker + Nix Deployment Script for CW4626 Vault"
echo "=================================================="

# Check if Docker container is running
if ! docker-compose ps | grep -q "cw4626-vault-dev.*Up"; then
    echo "❌ Docker container is not running. Starting it now..."
    docker-compose up -d
    sleep 5
fi

# Check if container is healthy
if ! docker-compose exec cw4626-nix bash -c "echo 'Container is ready'" > /dev/null 2>&1; then
    echo "❌ Docker container is not responding. Please check the container status."
    exit 1
fi

echo "✅ Docker container is running and ready"

# Build contracts if needed
echo ""
echo "🔨 Building contracts..."
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo build --package cw4626-base --lib --target wasm32-unknown-unknown --release"
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo build --package cw4626-escher --lib --target wasm32-unknown-unknown --release"

# Optimize WASM files
echo ""
echo "⚡ Optimizing WASM files..."
docker-compose exec cw4626-nix bash -c "cd /workspace && wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_base.wasm -o target/wasm32-unknown-unknown/release/cw4626_base_optimized.wasm"
docker-compose exec cw4626-nix bash -c "cd /workspace && wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_escher.wasm -o target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm"

# Show file sizes
echo ""
echo "📊 Contract sizes:"
docker-compose exec cw4626-nix bash -c "cd /workspace && ls -lh target/wasm32-unknown-unknown/release/*_optimized.wasm"

# Check if babylond is available
echo ""
echo "🔍 Checking babylond availability..."
if docker-compose exec cw4626-nix bash -c "command -v babylond" > /dev/null 2>&1; then
    echo "✅ babylond is available in container"
    docker-compose exec cw4626-nix bash -c "babylond version"
else
    echo "⚠️  babylond is not available in container"
    echo "   You'll need to use the host system's babylond for deployment"
fi

echo ""
echo "🎯 Deployment Options:"
echo "1. Deploy from host system (recommended if babylond not in container)"
echo "2. Deploy from container (if babylond is available)"
echo "3. Just build and optimize contracts"
echo ""

read -p "Choose an option (1-3): " choice

case $choice in
    1)
        echo ""
        echo "🚀 Deploying from host system..."
        echo "   The optimized WASM files are available in:"
        echo "   - target/wasm32-unknown-unknown/release/cw4626_base_optimized.wasm"
        echo "   - target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm"
        echo ""
        echo "   Use these files with your host system's babylond CLI"
        ;;
    2)
        if docker-compose exec cw4626-nix bash -c "command -v babylond" > /dev/null 2>&1; then
            echo ""
            echo "🚀 Deploying from container..."
            echo "   This will use the container's babylond"
            echo "   Note: You'll need to configure your keys and network settings"
        else
            echo "❌ babylond not available in container. Falling back to option 1."
            echo "   The optimized WASM files are available in:"
            echo "   - target/wasm32-unknown-unknown/release/cw4626_base_optimized.wasm"
            echo "   - target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm"
        fi
        ;;
    3)
        echo ""
        echo "✅ Contracts built and optimized successfully!"
        echo "   Ready for deployment when you're ready."
        ;;
    *)
        echo "❌ Invalid option. Exiting."
        exit 1
        ;;
esac

echo ""
echo "🎉 Docker + Nix environment is ready for development and deployment!"
echo ""
echo "🔧 Useful commands:"
echo "   - Enter container: docker-compose exec cw4626-nix bash"
echo "   - View logs: docker-compose logs -f"
echo "   - Stop environment: docker-compose down"
echo "   - Rebuild: docker-compose up -d --build"
