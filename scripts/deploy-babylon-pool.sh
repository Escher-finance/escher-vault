#!/bin/bash

# Babylon Pool-Specific Deployment Script
# This script deploys the Escher vault configured for your specific Astroport pool

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

# Configuration for your pool
POOL_ADDRESS="bbn1hkmstu883spzwj4h38yph6wqv4k92g90fga3jv3n7ywswn6yr5nq3j4gas"
CHAIN_ID="bbn-test-5"
NODE="https://babylon-testnet-rpc.polkachu.com"
GAS_PRICES="0.025ubbn"

echo "🚀 Deploying Escher Vault for Your Babylon Pool"
echo "=================================================="
echo ""
print_info "Pool Address: $POOL_ADDRESS"
print_info "Chain ID: $CHAIN_ID"
print_info "RPC Node: $NODE"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Check if WASM files exist
if [ ! -f "target/wasm32-unknown-unknown/release/cw4626_escher.wasm" ]; then
    print_error "Escher contract WASM file not found. Run './scripts/test-vault.sh' first."
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
print_status "Step 1: Uploading Escher contract code..."

# Upload contract
UPLOAD_TX=$(babylond tx wasm store target/wasm32-unknown-unknown/release/cw4626_escher.wasm \
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
print_status "Step 2: Ready to instantiate contract"
echo ""
echo "📋 Instantiation Command:"
echo "babylond tx wasm instantiate $CODE_ID '\\"
echo "  {"
echo "    \"underlying_token_address\": \"<your-token-address>\","
echo "    \"share_name\": \"Babylon LP Vault\","
echo "    \"share_symbol\": \"bLP\","
echo "    \"share_marketing\": {"
echo "      \"project\": \"https://your-project.com\","
echo "      \"description\": \"Automated LP vault for Babylon testnet\","
echo "      \"marketing\": \"<marketing-address>\""
echo "    },"
echo "    \"manager\": \"<manager-address>\","
echo "    \"oracle\": \"<oracle-address>\","
echo "    \"tower_incentives\": \"<tower-incentives-address>\","
echo "    \"lp\": \"$POOL_ADDRESS\","
echo "    \"slippage_tolerance\": \"0.01\","
echo "    \"incentives\": ["
echo "      {"
echo "        \"native_token\": {"
echo "          \"denom\": \"ubbn\""
echo "        }"
echo "      }"
echo "    ]"
echo "  }' \\"
echo "  --from $KEY_NAME \\"
echo "  --chain-id $CHAIN_ID \\"
echo "  --node $NODE \\"
echo "  --gas auto \\"
echo "  --gas-adjustment 1.3 \\"
echo "  --gas-prices $GAS_PRICES \\"
echo "  --yes"
echo ""

print_info "Next steps:"
echo "1. Replace placeholder addresses with actual addresses"
echo "2. Run the instantiation command"
echo "3. Test the vault functionality"
echo "4. Verify LP operations work correctly"

# Save deployment info
DEPLOYMENT_FILE="deployment-babylon-pool-$(date +%Y%m%d-%H%M%S).json"
cat > $DEPLOYMENT_FILE << EOF
{
    "network": "babylon-genesis-testnet",
    "contract_type": "escher",
    "code_id": "$CODE_ID",
    "pool_address": "$POOL_ADDRESS",
    "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "deployed_by": "$KEY_NAME",
    "key_address": "$KEY_ADDRESS",
    "chain_id": "$CHAIN_ID",
    "node": "$NODE"
}
EOF

print_status "Deployment info saved to: $DEPLOYMENT_FILE"
