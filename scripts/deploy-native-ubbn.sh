#!/bin/bash

# Deploy Native Ubbn Vault Contract
# This script deploys the modified base contract that supports native ubbn tokens

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

# Configuration
CHAIN_ID="bbn-test-5"
NODE="https://babylon-testnet-rpc.polkachu.com"
GAS_PRICES="0.025ubbn"

echo "🚀 Deploying Native Ubbn Vault Contract"
echo "========================================"
echo ""
print_info "Chain ID: $CHAIN_ID"
print_info "RPC Node: $NODE"
print_info "Gas Prices: $GAS_PRICES"
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

# Check if babylond is installed
if ! command -v babylond &> /dev/null; then
    print_error "babylond CLI not found. Please install the Babylon Genesis binary first."
    exit 1
fi

# Get key name from user
read -p "Enter your key name: " KEY_NAME

# Check if key exists
if ! babylond keys show $KEY_NAME &> /dev/null; then
    print_warning "Key '$KEY_NAME' not found. Creating new key..."
    babylond keys add $KEY_NAME
fi

# Get key address
KEY_ADDRESS=$(babylond keys show $KEY_NAME -a)
print_info "Using key: $KEY_NAME ($KEY_ADDRESS)"

# Check balance
BALANCE=$(babylond query bank balances $KEY_ADDRESS --node $NODE --output json | jq -r '.balances[] | select(.denom=="ubbn") | .amount // "0"')
if [ "$BALANCE" -lt 1000000 ]; then
    print_warning "Low balance: $BALANCE ubbn. Ensure sufficient funds for deployment."
fi

echo ""
print_status "Step 1: Uploading modified base contract code..."

# Upload contract
UPLOAD_TX=$(babylond tx wasm store target/wasm32-unknown-unknown/release/cw4626_base.wasm \
    --from $KEY_NAME \
    --chain-id $CHAIN_ID \
    --node $NODE \
    --gas auto \
    --gas-adjustment 1.3 \
    --gas-prices $GAS_PRICES \
    --output json \
    --yes)

if [ $? -eq 0 ]; then
    CODE_ID=$(echo $UPLOAD_TX | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
    print_status "Contract uploaded successfully! Code ID: $CODE_ID"
else
    print_error "Failed to upload contract"
    exit 1
fi

echo ""
print_status "Step 2: Ready to instantiate contract with native ubbn support"
echo ""
echo "📋 Instantiation Command for Native Ubbn:"
echo "babylond tx wasm instantiate $CODE_ID '\\"
echo "  {"
echo "    \"underlying_token\": {"
echo "      \"native\": {"
echo "        \"denom\": \"ubbn\""
echo "      }"
echo "    },"
echo "    \"share_name\": \"Native Ubbn Vault\","
echo "    \"share_symbol\": \"nUBBN\","
echo "    \"share_marketing\": {"
echo "      \"project\": \"https://your-project.com\","
echo "      \"description\": \"Vault for native ubbn tokens\","
echo "      \"marketing\": \"$KEY_ADDRESS\""
echo "    }"
echo "  }' \\"
echo "  --label \"native-ubbn-vault\" \\"
echo "  --admin $KEY_ADDRESS \\"
echo "  --from $KEY_NAME \\"
echo "  --chain-id $CHAIN_ID \\"
echo "  --node $NODE \\"
echo "  --gas auto \\"
echo "  --gas-adjustment 1.3 \\"
echo "  --gas-prices $GAS_PRICES \\"
echo "  --yes"
echo ""

print_info "Key Features of Modified Contract:"
echo "✅ Supports native ubbn tokens (no CW20 wrapper needed)"
echo "✅ Direct deposit/withdraw of ubbn tokens"
echo "✅ Automatic share token minting/burning"
echo "✅ Backward compatible with CW20 tokens"
echo "✅ Enhanced error handling for native operations"

echo ""
print_info "Next steps:"
echo "1. Run the instantiation command above"
echo "2. Test deposit_native with ubbn tokens"
echo "3. Test withdraw_native to receive ubbn back"
echo "4. Verify share token operations work correctly"

# Save deployment info
DEPLOYMENT_FILE="deployment-native-ubbn-$(date +%Y%m%d-%H%M%S).json"
cat > $DEPLOYMENT_FILE << EOF
{
    "network": "babylon-genesis-testnet",
    "contract_type": "base_with_native_support",
    "code_id": "$CODE_ID",
    "underlying_token": "native_ubbn",
    "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "deployed_by": "$KEY_NAME",
    "key_address": "$KEY_ADDRESS",
    "chain_id": "$CHAIN_ID",
    "node": "$NODE",
    "features": [
        "native_ubbn_support",
        "direct_deposit_withdraw",
        "automatic_share_minting",
        "backward_cw20_compatibility"
    ]
}
EOF

print_status "Deployment info saved to: $DEPLOYMENT_FILE"
print_status "You now have a vault contract that works directly with native ubbn tokens!"
