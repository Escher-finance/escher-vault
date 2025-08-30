#!/bin/bash

# Clippy Code Quality Check Script for CW4626 Vault
set -e

echo "🔍 Running Clippy Code Quality Check..."
echo "========================================"

# Check if we're in Docker environment
if command -v docker-compose >/dev/null 2>&1 && docker-compose ps | grep -q "cw4626-vault-dev.*Up"; then
    echo "🐳 Running Clippy in Docker + Nix environment..."
    
    # Run Clippy on all packages
    echo ""
    echo "📦 Checking cw4626 package..."
    docker-compose exec cw4626-nix bash -c "cd /workspace && cargo clippy --package cw4626 -- -W clippy::all"
    
    echo ""
    echo "📦 Checking cw4626-base contract..."
    docker-compose exec cw4626-nix bash -c "cd /workspace && cargo clippy --package cw4626-base -- -W clippy::all"
    
    echo ""
    echo "📦 Checking cw4626-escher contract..."
    docker-compose exec cw4626-nix bash -c "cd /workspace && cargo clippy --package cw4626-escher -- -W clippy::all"
    
    echo ""
    echo "📦 Running Clippy on all packages..."
    docker-compose exec cw4626-nix bash -c "cd /workspace && cargo clippy --workspace -- -W clippy::all"
    
elif command -v nix >/dev/null 2>&1; then
    echo "🐧 Running Clippy in Nix environment..."
    
    # Check if we're in a Nix shell
    if [[ -n "$IN_NIX_SHELL" ]]; then
        echo "✅ Already in Nix shell, running Clippy..."
        
        # Run Clippy on all packages
        echo ""
        echo "📦 Checking cw4626 package..."
        cargo clippy --package cw4626 -- -W clippy::all
        
        echo ""
        echo "📦 Checking cw4626-base contract..."
        cargo clippy --package cw4626-base -- -W clippy::all
        
        echo ""
        echo "📦 Checking cw4626-escher contract..."
        cargo clippy --package cw4626-escher -- -W clippy::all
        
        echo ""
        echo "📦 Running Clippy on all packages..."
        cargo clippy --workspace -- -W clippy::all
        
    else
        echo "🔄 Entering Nix shell to run Clippy..."
        nix develop --command bash -c "
            echo '📦 Checking cw4626 package...'
            cargo clippy --package cw4626 -- -W clippy::all
            
            echo ''
            echo '📦 Checking cw4626-base contract...'
            cargo clippy --package cw4626-base -- -W clippy::all
            
            echo ''
            echo '📦 Checking cw4626-escher contract...'
            cargo clippy --package cw4626-escher -- -W clippy::all
            
            echo ''
            echo '📦 Running Clippy on all packages...'
            cargo clippy --workspace -- -W clippy::all
        "
    fi
    
else
    echo "🔧 Running Clippy in traditional environment..."
    
    # Check if Clippy is available
    if ! command -v cargo-clippy >/dev/null 2>&1 && ! cargo clippy --version >/dev/null 2>&1; then
        echo "❌ Clippy not found. Installing..."
        rustup component add clippy
    fi
    
    # Run Clippy on all packages
    echo ""
    echo "📦 Checking cw4626 package..."
    cargo clippy --package cw4626 -- -W clippy::all
    
    echo ""
    echo "📦 Checking cw4626-base contract..."
    cargo clippy --package cw4626-base -- -W clippy::all
    
    echo ""
    echo "📦 Checking cw4626-escher contract..."
    cargo clippy --package cw4626-escher -- -W clippy::all
    
    echo ""
    echo "📦 Running Clippy on all packages..."
    cargo clippy --workspace -- -W clippy::all
fi

echo ""
echo "✅ Clippy check completed!"
echo ""
echo "💡 Tips:"
echo "  - Fix warnings with: cargo clippy --fix"
echo "  - View specific warnings: cargo clippy --package <package> -- -W <lint-name>"
echo "  - Suppress specific warnings: #[allow(clippy::<lint-name>)]"
echo ""
echo "🔗 Learn more: https://rust-lang.github.io/rust-clippy/"
