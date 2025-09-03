#!/bin/bash

# Babylon Deployment Script with Environment Variables
# Set your configuration in .env file or export variables

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

# Load environment variables from .env file if it exists
if [ -f ".env" ]; then
    print_info "Loading configuration from .env file..."
    # Load environment variables, excluding comments and empty lines
    while IFS= read -r line; do
        # Skip comments and empty lines
        if [[ ! "$line" =~ ^[[:space:]]*# ]] && [[ -n "$line" ]]; then
            export "$line"
        fi
    done < ".env"
fi

# Required environment variables
REQUIRED_VARS=(
    "BABYLON_CHAIN_ID"
    "BABYLON_RPC_NODE"
    "BABYLON_GAS_PRICES"
    "BABYLON_POOL_ADDRESS"
    "BABYLON_KEY_NAME"
    "BABYLON_UNDERLYING_TOKEN"
    "BABYLON_MANAGER"
    "BABYLON_ORACLE"
    "BABYLON_TOWER_INCENTIVES"
)

# Check if all required variables are set
MISSING_VARS=()
for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var}" ]; then
        MISSING_VARS+=("$var")
    fi
done

if [ ${#MISSING_VARS[@]} -ne 0 ]; then
    print_error "Missing required environment variables:"
    for var in "${MISSING_VARS[@]}"; do
        echo "  - $var"
    done
    echo ""
    echo "Please set these variables in your .env file or export them:"
    echo "Example .env file:"
    cat << 'EOF'
BABYLON_CHAIN_ID=bbn-test-5
BABYLON_RPC_NODE=https://babylon-testnet-rpc.polkachu.com
BABYLON_GAS_PRICES=0.025ubbn
BABYLON_POOL_ADDRESS=bbn1hkmstu883spzwj4k92g90fga3jv3n7ywswn6yr5nq3j4gas
BABYLON_KEY_NAME=my-key
BABYLON_UNDERLYING_TOKEN=bbn1...
BABYLON_MANAGER=bbn1...
BABYLON_ORACLE=bbn1...
BABYLON_TOWER_INCENTIVES=bbn1...
EOF
    exit 1
fi

echo "🚀 Deploying Escher Vault with Environment Variables"
echo "====================================================="
echo ""
print_info "Chain ID: $BABYLON_CHAIN_ID"
print_info "RPC Node: $BABYLON_RPC_NODE"
print_info "Gas Prices: $BABYLON_GAS_PRICES"
print_info "Pool Address: $BABYLON_POOL_ADDRESS"
print_info "Key Name: $BABYLON_KEY_NAME"
print_info "Underlying Token: $BABYLON_UNDERLYING_TOKEN"
print_info "Manager: $BABYLON_MANAGER"
print_info "Oracle: $BABYLON_ORACLE"
print_info "Tower Incentives: $BABYLON_TOWER_INCENTIVES"
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

# Check if key exists
if ! babylond keys show $BABYLON_KEY_NAME &> /dev/null; then
    print_error "Key '$BABYLON_KEY_NAME' not found. Please create it first with: babylond keys add $BABYLON_KEY_NAME"
    exit 1
fi

# Get key address
KEY_ADDRESS=$(babylond keys show $BABYLON_KEY_NAME -a)
print_info "Using key: $BABYLON_KEY_NAME ($KEY_ADDRESS)"

# Check balance
BALANCE=$(babylond query bank balances $KEY_ADDRESS --node $BABYLON_RPC_NODE --output json | jq -r '.balances[] | select(.denom=="ubbn") | .amount // "0"')
if [ "$BALANCE" -lt 1000000 ]; then
    print_warning "Low balance: $BALANCE ubbn. Ensure sufficient funds for deployment."
fi

echo ""
print_status "Step 1: Uploading Escher contract code..."

# Upload contract
UPLOAD_TX=$(babylond tx wasm store target/wasm32-unknown-unknown/release/cw4626_escher.wasm \
    --from $BABYLON_KEY_NAME \
    --chain-id $BABYLON_CHAIN_ID \
    --node $BABYLON_RPC_NODE \
    --gas auto \
    --gas-adjustment 1.3 \
    --gas-prices $BABYLON_GAS_PRICES \
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
print_status "Step 2: Instantiating contract with your configuration..."

# Instantiate contract
INSTANTIATE_TX=$(babylond tx wasm instantiate $CODE_ID "{
  \"managers\": [\"$BABYLON_MANAGER\"],
  \"oracles\": [\"$BABYLON_ORACLE\"],
  \"underlying_token\": {
    \"native_token\": {
      \"denom\": \"ubbn\"
    }
  },
  \"share_name\": \"Babylon LP Vault\",
  \"share_symbol\": \"bLP\",
  \"share_marketing\": {
    \"project\": \"https://your-project.com\",
    \"description\": \"Automated LP vault for Babylon testnet\",
    \"marketing\": \"$BABYLON_MANAGER\"
  },
  \"tower_incentives\": \"$BABYLON_TOWER_INCENTIVES\",
  \"lp\": \"$BABYLON_POOL_ADDRESS\",
  \"slippage_tolerance\": \"0.01\",
  \"incentives\": [
    {
      \"native_token\": {
        \"denom\": \"ubbn\"
      }
    }
  ],
  \"staking_contract\": null
}" \
  --from $BABYLON_KEY_NAME \
  --chain-id $BABYLON_CHAIN_ID \
  --node $BABYLON_RPC_NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $BABYLON_GAS_PRICES \
  --yes)

if [ $? -eq 0 ]; then
    CONTRACT_ADDRESS=$(echo $INSTANTIATE_TX | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
    print_status "Contract instantiated successfully! Contract Address: $CONTRACT_ADDRESS"
else
    print_error "Failed to instantiate contract"
    exit 1
fi

# Save deployment info
DEPLOYMENT_FILE="deployment-babylon-env-$(date +%Y%m%d-%H%M%S).json"
cat > $DEPLOYMENT_FILE << EOF
{
    "network": "babylon-genesis-testnet",
    "contract_type": "escher",
    "code_id": "$CODE_ID",
    "contract_address": "$CONTRACT_ADDRESS",
    "pool_address": "$BABYLON_POOL_ADDRESS",
    "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "deployed_by": "$BABYLON_KEY_NAME",
    "key_address": "$KEY_ADDRESS",
    "chain_id": "$BABYLON_CHAIN_ID",
    "node": "$BABYLON_RPC_NODE",
    "configuration": {
        "underlying_token": "$BABYLON_UNDERLYING_TOKEN",
        "manager": "$BABYLON_MANAGER",
        "oracle": "$BABYLON_ORACLE",
        "tower_incentives": "$BABYLON_TOWER_INCENTIVES"
    }
}
EOF

print_status "Deployment completed successfully! 🎉"
print_status "Deployment info saved to: $DEPLOYMENT_FILE"
echo ""
print_info "Your vault is now deployed at: $CONTRACT_ADDRESS"
echo ""
print_info "Next steps:"
echo "1. Test the vault functionality"
echo "2. Verify LP operations work correctly"
echo "3. Set up monitoring and alerts"
echo "4. Share the contract address with users"
