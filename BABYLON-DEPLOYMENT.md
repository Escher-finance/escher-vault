# Babylon Testnet Deployment Guide

## 🌟 Network Information

**Network**: Babylon Genesis Testnet  
**Chain ID**: `bbn-test-5`  
**Chain Name**: Babylon Testnet  
**Binary**: `babylond` v0.5.0  
**Genesis Date**: 2025-01-09  
**Denomination**: `ubbn` (1 BBN = 1,000,000 ubbn)

## 🏊‍♂️ **Your Astroport Pool**

**Pool Contract**: [`bbn1hkmstu883spzwj4k92g90fga3jv3n7ywswn6yr5nq3j4gas`](https://www.mintscan.io/babylon-testnet/wasm/contract/bbn1hkmstu883spzwj4k92g90fga3jv3n7ywswn6yr5nq3j4gas?sector=assets)

**Pool Type**: Concentrated Liquidity  
**Total LP Shares**: 711,144,084

**Pool Assets**:
- **Asset 1**: `ibc/3AA6631D204C192DDB757935A4C49A0E83EEEE14AC045E8A180CCB4EE08B6196`
- **Asset 2**: `ibc/DC9A0BC30A89A4C767CA2DA3BA1A4B1AB40F6666E720BB4F14213545216C86D8`

This pool will be used for automated LP provision and management by the Escher vault.

## 🔗 Endpoints

| Type | URL |
|------|-----|
| **RPC** | `https://babylon-testnet-rpc.polkachu.com` |
| **LCD** | `https://babylon-testnet-api.polkachu.com` |
| **gRPC** | `http://babylon-testnet-grpc.polkachu.com:20690` |
| **Explorer** | `https://testnet.mintscan.io/babylon` |

## 🚀 Quick Deployment

### 1. Deploy to Babylon Testnet
```bash
# Deploy Escher vault (recommended for LP automation)
./scripts/deploy.sh babylon escher my-key

# Or deploy base vault
./scripts/deploy.sh babylon base my-key
```

### 2. Get Testnet Tokens
Visit the [Babylon Faucet](https://faucet.babylon-testnet.com) to get testnet BABYLON tokens.

## 📋 Prerequisites

### Required Tools
- **babylond** CLI v0.5.0+ (Babylon Genesis binary)
- **jq** for JSON parsing
- **Sufficient balance** (at least 1 BABYLON for gas fees)

**Note**: Babylon uses `babylond` instead of `wasmd`. Make sure you have the correct binary installed.

### Check Installation
```bash
# Verify babylond version (should be v0.5.0+)
babylond version

# Check if you have a key
babylond keys list

# Create a new key if needed
babylond keys add my-key
```

## 🔧 Manual Deployment Steps

### Step 1: Upload Contract Code
```bash
# Upload Escher contract
babylond tx wasm store target/wasm32-unknown-unknown/release/cw4626_escher.wasm \
  --from my-key \
  --chain-id bbn-test-5 \
  --node https://babylon-testnet-rpc.polkachu.com \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices 0.025ubbn \
  --yes
```

### Step 2: Get Code ID
```bash
# Extract code ID from upload transaction
CODE_ID=$(babylond query wasm list-code --node https://babylon-testnet-rpc.polkachu.com --output json | jq -r '.code_infos[-1].code_id')
echo "Code ID: $CODE_ID"
```

### Step 3: Instantiate Contract
```bash
# Instantiate Escher vault
babylond tx wasm instantiate $CODE_ID '{
  "underlying_token_address": "babylon1...",
  "share_name": "Babylon Escher Vault",
  "share_symbol": "bESCHER",
  "share_marketing": {
    "project": "https://your-project.com",
    "description": "Automated LP vault for Babylon testnet",
    "marketing": "babylon1..."
  },
  "manager": "babylon1...",
  "oracle": "babylon1...",
  "tower_incentives": "babylon1...",
  "lp": "bbn1hkmstu883spzwj4k92g90fga3jv3n7ywswn6yr5nq3j4gas",
  "slippage_tolerance": "0.01",
  "incentives": [
    {
      "native_token": {
        "denom": "ubbn"
      }
    }
  ]
}' \
  --from my-key \
  --chain-id bbn-test-5 \
  --node https://babylon-testnet-rpc.polkachu.com \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices 0.025ubbn \
  --yes
```

## 🧪 Testing on Babylon

### 1. Check Contract State
```bash
# Get contract address from instantiation
CONTRACT_ADDRESS=$(babylond query wasm list-contract-by-code $CODE_ID --node https://babylon-testnet-rpc.polkachu.com --output json | jq -r '.contracts[-1]')

# Query vault info
babylond query wasm contract-state smart $CONTRACT_ADDRESS '{"asset": {}}' \
  --node https://babylon-testnet-rpc.polkachu.com

# Get total assets
babylond query wasm contract-state smart $CONTRACT_ADDRESS '{"total_assets": {}}' \
  --node https://babylon-testnet-rpc.polkachu.com
```

### 2. Test Deposit (if you have test tokens)
```bash
# First approve spending
babylond tx wasm execute <cw20-token-address> '{
  "increase_allowance": {
    "spender": "'$CONTRACT_ADDRESS'",
    "amount": "1000000"
  }
}' \
  --from my-key \
  --chain-id bbn-test-5 \
  --node https://babylon-testnet-rpc.polkachu.com \
  --gas auto \
  --gas-prices 0.025ubbn \
  --yes

# Then deposit
babylond tx wasm execute $CONTRACT_ADDRESS '{
  "deposit": {
    "assets": "1000000",
    "receiver": "'$(babylond keys show my-key -a)'"
  }
}' \
  --from my-key \
  --chain-id bbn-test-5 \
  --node https://babylon-testnet-rpc.polkachu.com \
  --gas auto \
  --gas-prices 0.025ubbn \
  --yes
```

## 🔍 Monitoring

### Check Transaction Status
```bash
# Get transaction hash from response
TX_HASH="<transaction-hash>"

# Query transaction details
babylond query tx $TX_HASH \
  --node https://babylon-testnet-rpc.polkachu.com \
  --output json
```

### View Contract Events
```bash
# Get contract events
babylond query wasm contract-state smart $CONTRACT_ADDRESS '{"config": {}}' \
  --node https://babylon-testnet-rpc.polkachu.com
```

## ⚠️ Important Notes

### Gas Configuration
- **Gas Prices**: `0.025ubbn`
- **Gas Adjustment**: `1.3` (recommended)
- **Minimum Balance**: At least 1 BBN for deployment

### Network-Specific Considerations
- **Babylon testnet** is for testing only
- **Contracts** need to be redeployed for mainnet
- **Test tokens** are available from faucet
- **Network stability** may vary during testnet phase

## 🚨 Troubleshooting

### Common Issues

1. **Insufficient Balance**
   ```bash
   # Check balance
   babylond query bank balances $(babylond keys show my-key -a) \
     --node https://babylon-testnet-rpc.polkachu.com
   ```

2. **Network Connection Issues**
   ```bash
   # Test RPC connection
   curl -s https://babylon-testnet-rpc.polkachu.com/status
   ```

3. **Contract Instantiation Failures**
   - Verify all addresses are valid Babylon addresses
   - Check that dependencies (Astroport, tokens) exist
   - Ensure sufficient gas for complex operations

### Getting Help
- **Babylon Discord**: Join the official Babylon testnet channel
- **GitHub Issues**: Report bugs in this repository
- **Network Status**: Check [Babylon testnet status](https://testnet.mintscan.io/babylon)

## 🔄 Next Steps

1. **Test thoroughly** on Babylon testnet
2. **Verify all functionality** works as expected
3. **Get community feedback** on testnet
4. **Prepare for mainnet** deployment
5. **Set up monitoring** and alerts

---

**Happy testing on Babylon! 🚀**
