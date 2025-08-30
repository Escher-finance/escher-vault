#!/usr/bin/env bash

# Babylon Deployment Script for Nix Environment
# This script works within the Nix development shell

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

echo "🚀 Nix-Enabled Babylon Deployment"
echo "=================================="
echo ""

# Check if we're in a Nix environment
if [ -z "$IN_NIX_SHELL" ]; then
    print_warning "Not in Nix shell. Starting Nix development environment..."
    echo "Run: nix develop"
    echo "Then run this script again."
    exit 1
fi

print_status "Nix environment detected! ✅"

# Check if required tools are available
print_info "Checking available tools..."

if command -v rustc &> /dev/null; then
    print_status "Rust: $(rustc --version)"
else
    print_error "Rust not found in Nix environment"
    exit 1
fi

if command -v wasm-opt &> /dev/null; then
    print_status "wasm-opt: $(wasm-opt --version)"
else
    print_error "wasm-opt not found in Nix environment"
    exit 1
fi

if command -v jq &> /dev/null; then
    print_status "jq: $(jq --version)"
else
    print_error "jq not found in Nix environment"
    exit 1
fi

# Check if babylond is available
if command -v babylond &> /dev/null; then
    print_status "babylond: $(babylond version)"
else
    print_warning "babylond not found. You may need to install it separately or add it to the Nix flake."
    print_info "Install with: go install github.com/babylonlabs/babylon/cmd/babylond@latest"
fi

echo ""
print_info "Building contracts with Nix environment..."

# Build contracts
if [ ! -f "target/wasm32-unknown-unknown/release/cw4626_escher.wasm" ]; then
    print_status "Building Escher contract..."
    cargo wasm -p cw4626-escher
else
    print_status "Escher contract already built"
fi

if [ ! -f "target/wasm32-unknown-unknown/release/cw4626_base.wasm" ]; then
    print_status "Building base contract..."
    cargo wasm -p cw4626-base
else
    print_status "Base contract already built"
fi

echo ""
print_status "Contracts built successfully! 🎉"

# Check if .env file exists
if [ -f ".env" ]; then
    print_info "Loading configuration from .env file..."
    export $(cat .env | grep -v '^#' | xargs)
else
    print_warning "No .env file found. Using default values."
    export BABYLON_CHAIN_ID=${BABYLON_CHAIN_ID:-"bbn-test-5"}
    export BABYLON_RPC_NODE=${BABYLON_RPC_NODE:-"https://babylon-testnet-rpc.polkachu.com"}
    export BABYLON_GAS_PRICES=${BABYLON_GAS_PRICES:-"0.025ubbn"}
    export BABYLON_POOL_ADDRESS=${BABYLON_POOL_ADDRESS:-"bbn1hkmstu883spzwj4k92g90fga3jv3n7ywswn6yr5nq3j4gas"}
fi

# Display current configuration
echo ""
print_info "Current Configuration:"
echo "  Chain ID: $BABYLON_CHAIN_ID"
echo "  RPC Node: $BABYLON_RPC_NODE"
echo "  Gas Prices: $BABYLON_GAS_PRICES"
echo "  Pool Address: $BABYLON_POOL_ADDRESS"

# Check if babylond is available for deployment
if ! command -v babylond &> /dev/null; then
    echo ""
    print_error "Cannot proceed with deployment - babylond not found"
    echo ""
    print_info "To install babylond:"
    echo "1. Install Go: nix-env -iA nixpkgs.go"
    echo "2. Install babylond: go install github.com/babylonlabs/babylon/cmd/babylond@latest"
    echo "3. Add to PATH: export PATH=\$PATH:\$HOME/go/bin"
    echo ""
    print_info "Or add babylond to the Nix flake and rebuild: nix develop"
    exit 1
fi

echo ""
print_info "Ready for deployment! 🚀"
echo ""
print_info "Next steps:"
echo "1. Set your key name: export BABYLON_KEY_NAME=my-key"
echo "2. Set remaining config: export BABYLON_UNDERLYING_TOKEN=bbn1..."
echo "3. Run deployment: ./scripts/deploy-babylon-env.sh"
echo ""
print_info "Or create a .env file with all your configuration and run:"
echo "  ./scripts/deploy-babylon-env.sh"
