#!/usr/bin/env bash

# Simplified Environment Setup for CW4626 Vault
# This script works with existing tools (Rust, Cargo, wasm-opt)

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

echo "🚀 CW4626 Vault Environment Setup"
echo "=================================="
echo ""

# Check if required tools are available
print_info "Checking available tools..."

# Check Rust
if command -v rustc &> /dev/null; then
    print_status "Rust: $(rustc --version)"
else
    print_error "Rust not found. Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check Cargo
if command -v cargo &> /dev/null; then
    print_status "Cargo: $(cargo --version)"
else
    print_error "Cargo not found. Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check wasm-opt
if command -v wasm-opt &> /dev/null; then
    print_status "wasm-opt: $(wasm-opt --version)"
else
    print_warning "wasm-opt not found. Installing via Homebrew..."
    if command -v brew &> /dev/null; then
        brew install binaryen
        print_status "wasm-opt installed via Homebrew"
    else
        print_error "Homebrew not found. Please install wasm-opt manually:"
        echo "  brew install binaryen"
        exit 1
    fi
fi

# Check jq
if command -v jq &> /dev/null; then
    print_status "jq: $(jq --version)"
else
    print_warning "jq not found. Installing via Homebrew..."
    if command -v brew &> /dev/null; then
        brew install jq
        print_status "jq installed via Homebrew"
    else
        print_error "Homebrew not found. Please install jq manually"
        exit 1
    fi
fi

# Check if babylond is available
if command -v babylond &> /dev/null; then
    print_status "babylond: $(babylond version)"
else
    print_warning "babylond not found. You'll need to install it for deployment."
    print_info "Install with: go install github.com/babylonlabs/babylon/cmd/babylond@latest"
fi

echo ""
print_status "Environment setup complete! 🎉"

# Set up environment variables
export RUST_BACKTRACE=1
export RUST_LOG=info
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-opt

# Load .env file if it exists
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

echo ""
print_info "Environment variables set:"
echo "  RUST_BACKTRACE=$RUST_BACKTRACE"
echo "  RUST_LOG=$RUST_LOG"
echo "  CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=$CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER"

echo ""
print_info "Next steps:"
echo "1. Build contracts: cargo wasm -p cw4626-escher"
echo "2. Run tests: cargo test"
echo "3. Deploy: ./scripts/deploy-babylon-env.sh"
echo ""
print_info "To make these environment variables permanent, add to your shell profile:"
echo "  export RUST_BACKTRACE=1"
echo "  export RUST_LOG=info"
echo "  export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-opt"
