#!/bin/bash

# Test script for CW4626 Vault Contracts
# This script tests basic functionality of the vault contracts

set -e

echo "🚀 Testing CW4626 Vault Contracts..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    print_error "Rust is not installed. Please install Rust first."
    exit 1
fi

# Check if wasm-opt is installed
if ! command -v wasm-opt &> /dev/null; then
    print_warning "wasm-opt not found. Installing binaryen..."
    if command -v brew &> /dev/null; then
        brew install binaryen
    else
        print_error "Please install binaryen manually"
        exit 1
    fi
fi

print_status "Building contracts..."

# Clean previous builds
cargo clean

# Build base contract
print_status "Building cw4626-base..."
cargo wasm -p cw4626-base

# Build escher contract
print_status "Building cw4626-escher..."
cargo wasm -p cw4626-escher

# Check if WASM files were created
if [ ! -f "target/wasm32-unknown-unknown/release/cw4626_base.wasm" ]; then
    print_error "Base contract WASM file not found"
    exit 1
fi

if [ ! -f "target/wasm32-unknown-unknown/release/cw4626_escher.wasm" ]; then
    print_error "Escher contract WASM file not found"
    exit 1
fi

print_status "WASM files created successfully"

# Get file sizes
BASE_SIZE=$(ls -lh target/wasm32-unknown-unknown/release/cw4626_base.wasm | awk '{print $5}')
ESCHER_SIZE=$(ls -lh target/wasm32-unknown-unknown/release/cw4626_escher.wasm | awk '{print $5}')

print_status "Base contract size: $BASE_SIZE"
print_status "Escher contract size: $ESCHER_SIZE"

# Generate schemas
print_status "Generating JSON schemas..."
cargo schema -p cw4626-base
cargo schema -p cw4626-escher

# Check if schemas were created
if [ ! -f "schema/cw4626-escher.json" ]; then
    print_error "Schema files not found"
    exit 1
fi

print_status "Schema files generated successfully"

# Run tests
print_status "Running tests..."
cargo test

print_status "All tests passed!"

# Run specific integration tests
print_status "Running integration tests..."
cargo test --test integration

print_status "Integration tests passed!"

# Check for warnings
print_status "Checking for warnings..."
cargo check 2>&1 | grep -i warning || print_status "No warnings found"

# Final status
echo ""
print_status "🎉 All checks completed successfully!"
echo ""
echo "📁 Generated files:"
echo "   - Base contract: target/wasm32-unknown-unknown/release/cw4626_base.wasm"
echo "   - Escher contract: target/wasm32-unknown-unknown/release/cw4626_escher.wasm"
echo "   - Schemas: schema/"
echo ""
echo "🚀 Ready for deployment!"
echo ""
echo "Next steps:"
echo "1. Deploy to testnet first"
echo "2. Test all functionality"
echo "3. Deploy to mainnet"
echo "4. Set up monitoring and alerts"
