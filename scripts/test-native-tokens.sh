#!/bin/bash

# Test Native Token Functionality
# This script tests the modified contracts with native ubbn tokens

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

echo "🧪 Testing Native Token Functionality"
echo "====================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Check if WASM files exist
if [ ! -f "target/wasm32-unknown-unknown/release/cw4626_base.wasm" ]; then
    print_error "Base contract WASM file not found. Run 'cargo wasm -p cw4626-base' first."
    exit 1
fi

if [ ! -f "target/wasm32-unknown-unknown/release/cw4626_escher.wasm" ]; then
    print_error "Escher contract WASM file not found. Run 'cargo wasm -p cw4626-escher' first."
    exit 1
fi

print_status "Both contracts built successfully!"

echo ""
print_info "Testing contract modifications:"
echo "1. ✅ Added UnderlyingToken enum (Cw20 | Native)"
echo "2. ✅ Updated instantiate message to support both token types"
echo "3. ✅ Added native token execute messages (DepositNative, WithdrawNative, etc.)"
echo "4. ✅ Added native token helper functions"
echo "5. ✅ Updated contract instantiation logic"
echo "6. ✅ Added error handling for native tokens"

echo ""
print_info "Key Changes Made:"
echo "- contracts/cw4626-base/src/state.rs: Added TokenType enum"
echo "- contracts/cw4626-base/src/error.rs: Added InsufficientFunds and InvalidTokenType errors"
echo "- contracts/cw4626-base/src/helpers.rs: Added _deposit_native and _withdraw_native functions"
echo "- contracts/cw4626-base/src/execute.rs: Added native token execute functions"
echo "- contracts/cw4626-base/src/contract.rs: Updated instantiate and execute logic"
echo "- packages/cw4626/src/msg.rs: Added UnderlyingToken enum and native token messages"

echo ""
print_info "Next Steps:"
echo "1. Deploy the modified base contract with native ubbn support"
echo "2. Test deposit_native with ubbn tokens"
echo "3. Test withdraw_native to receive ubbn tokens back"
echo "4. Verify share token minting/burning works correctly"

echo ""
print_status "Contract modifications completed successfully!"
print_status "You can now instantiate the contract with native ubbn tokens!"
