#!/bin/bash

# Deployment script for CW4626 Vault Contracts
# Supports multiple networks and contract types

set -e

# Configuration
NETWORK=${1:-"testnet"}
CONTRACT_TYPE=${2:-"escher"}
KEY_NAME=${3:-"default"}

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

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Check if WASM files exist
if [ ! -f "target/wasm32-unknown-unknown/release/cw4626_base.wasm" ]; then
    print_error "Base contract WASM file not found. Run './scripts/test-vault.sh' first."
    exit 1
fi

if [ ! -f "target/wasm32-unknown-unknown/release/cw4626_escher.wasm" ]; then
    print_error "Escher contract WASM file not found. Run './scripts/test-vault.sh' first."
    exit 1
fi

# Network configuration
case $NETWORK in
    "testnet")
        CHAIN_ID="babylon-testnet-1"
        NODE="https://babylon-testnet-rpc.polkachu.com"
        GAS_PRICES="0.025ubabylon"
        ;;
    "mainnet")
        CHAIN_ID="mainnet-1"
        NODE="https://rpc.example.com:443"
        GAS_PRICES="0.025uatom"
        ;;
    "babylon")
        CHAIN_ID="bbn-test-5"
        NODE="https://babylon-testnet-rpc.polkachu.com"
        GAS_PRICES="0.025ubbn"
        ;;
    "local")
        CHAIN_ID="localnet"
        NODE="http://localhost:26657"
        GAS_PRICES="0.025uatom"
        ;;
    *)
        print_error "Unknown network: $NETWORK. Use: testnet, mainnet, babylon, or local"
        exit 1
        ;;
esac

# Contract configuration
case $CONTRACT_TYPE in
    "base")
        CONTRACT_FILE="cw4626_base.wasm"
        CONTRACT_NAME="CW4626 Base Vault"
        ;;
    "escher")
        CONTRACT_FILE="cw4626_escher.wasm"
        CONTRACT_NAME="CW4626 Escher Vault"
        ;;
    *)
        print_error "Unknown contract type: $CONTRACT_TYPE. Use: base or escher"
        exit 1
        ;;
esac

print_info "Deploying $CONTRACT_NAME to $NETWORK"
print_info "Chain ID: $CHAIN_ID"
print_info "Node: $NODE"
print_info "Gas Prices: $GAS_PRICES"

# Check if key exists
if [ "$NETWORK" = "babylon" ]; then
    CLI_BINARY="babylond"
else
    CLI_BINARY="wasmd"
fi

if ! $CLI_BINARY keys show $KEY_NAME &> /dev/null; then
    print_warning "Key '$KEY_NAME' not found. Creating new key..."
    $CLI_BINARY keys add $KEY_NAME
fi

# Get key address
KEY_ADDRESS=$($CLI_BINARY keys show $KEY_NAME -a)
print_info "Using key: $KEY_NAME ($KEY_ADDRESS)"

# Check balance
if [ "$NETWORK" = "babylon" ]; then
    DENOM="ubbn"
    MIN_BALANCE=1000000
else
    DENOM="uatom"
    MIN_BALANCE=1000000
fi

BALANCE=$($CLI_BINARY query bank balances $KEY_ADDRESS --node $NODE --output json | jq -r ".balances[] | select(.denom==\"$DENOM\") | .amount // \"0\"")
if [ "$BALANCE" -lt $MIN_BALANCE ]; then
    print_warning "Low balance: $BALANCE $DENOM. Ensure sufficient funds for deployment."
fi

# Upload contract
print_status "Uploading contract code..."
UPLOAD_TX=$($CLI_BINARY tx wasm store target/wasm32-unknown-unknown/release/$CONTRACT_FILE \
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

# Save deployment info
DEPLOYMENT_FILE="deployment-${NETWORK}-${CONTRACT_TYPE}.json"
cat > $DEPLOYMENT_FILE << EOF
{
    "network": "$NETWORK",
    "contract_type": "$CONTRACT_TYPE",
    "code_id": "$CODE_ID",
    "contract_file": "$CONTRACT_FILE",
    "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "deployed_by": "$KEY_NAME",
    "key_address": "$KEY_ADDRESS",
    "chain_id": "$CHAIN_ID",
    "node": "$NODE"
}
EOF

print_status "Deployment info saved to: $DEPLOYMENT_FILE"

# Next steps
echo ""
print_status "🎉 Contract deployed successfully!"
echo ""
echo "📋 Next steps:"
echo "1. Instantiate the contract with:"
echo "   wasmd tx wasm instantiate $CODE_ID '{\"instantiate_msg\": \"here\"}' \\"
echo "     --from $KEY_NAME --chain-id $CHAIN_ID --gas auto --gas-adjustment 1.3"
echo ""
echo "2. Test the contract functionality"
echo "3. Set up monitoring and alerts"
echo "4. Update frontend configuration"
echo ""
echo "📁 Files:"
echo "   - Contract: target/wasm32-unknown-unknown/release/$CONTRACT_FILE"
echo "   - Deployment info: $DEPLOYMENT_FILE"
echo "   - Schemas: schema/"
echo ""
echo "🔗 Useful commands:"
if [ "$NETWORK" = "babylon" ]; then
    echo "   - Query contract: babylond query wasm contract-state smart <contract-address> '{\"query_msg\": \"here\"}'"
    echo "   - Execute contract: babylond tx wasm execute <contract-address> '{\"execute_msg\": \"here\"}' --from $KEY_NAME"
    echo "   - View logs: babylond query tx <tx-hash> --output json"
else
    echo "   - Query contract: wasmd query wasm contract-state smart <contract-address> '{\"query_msg\": \"here\"}'"
    echo "   - Execute contract: wasmd tx wasm execute <contract-address> '{\"execute_msg\": \"here\"}' --from $KEY_NAME"
    echo "   - View logs: wasmd query tx <tx-hash> --output json"
fi
