# CW4626 Vault Deployment Guide

## Overview

This guide covers deploying the CW4626 vault contracts for LP staking on
Astroport forks.

## Contracts

### 1. cw4626-escher

- **Purpose**: Enhanced vault with LP automation and incentives
- **Features**: All base features + automated liquidity provision, oracle
  integration, role-based access
- **Use Case**: Production vault with automated LP management

## Prerequisites

### Required Tools

- Rust 1.70+
- wasm-opt (binaryen)
- CosmWasm compatible blockchain
- CLI tools (wasmd, junod, etc.)

### Dependencies

- Astroport fork (babydex)
- CW20 token contracts
- Oracle price feeds

## Build Commands

```bash
# Build all contracts
cargo wasm -p cw4626-escher

# Generate schemas
cargo schema -p cw4626-escher

# Run tests
cargo test
```

## Deployment Steps

### 1. Upload Contract Code

```bash
# Upload escher contract
wasmd tx wasm store target/wasm32-unknown-unknown/release/cw4626_escher.wasm \
  --from <key> --chain-id <chain-id> --gas auto --gas-adjustment 1.3
```

### 2. Instantiate Escher Contract

```bash
# Enhanced vault with LP automation
wasmd tx wasm instantiate <code-id> '{
  "underlying_token_address": "<cw20-token-address>",
  "share_name": "Escher USDC",
  "share_symbol": "eUSDC",
  "share_marketing": {
    "project": "https://your-project.com",
    "description": "Automated USDC LP Vault",
    "marketing": "<marketing-address>"
  },
  "manager": "<manager-address>",
  "oracle": "<oracle-address>",
  "tower_incentives": "<tower-incentives-address>",
  "lp": "<astroport-pair-address>",
  "slippage_tolerance": "0.01",
  "incentives": [
    {
      "native_token": {
        "denom": "uatom"
      }
    }
  ]
}' --from <key> --chain-id <chain-id> --gas auto --gas-adjustment 1.3
```

## Configuration

### Escher Contract

- **manager**: Address with vault management privileges
- **oracle**: Address authorized to update price feeds
- **tower_incentives**: Astroport incentives contract
- **lp**: Astroport concentrated liquidity pair
- **slippage_tolerance**: Maximum acceptable slippage (0.01 = 1%)
- **incentives**: List of reward tokens

## Usage Examples

### Deposit Assets

```bash
# Approve spending
wasmd tx wasm execute <cw20-token> '{
  "increase_allowance": {
    "spender": "<vault-address>",
    "amount": "1000000"
  }
}' --from <user> --chain-id <chain-id>

# Deposit to vault
wasmd tx wasm execute <vault-address> '{
  "deposit": {
    "assets": "1000000",
    "receiver": "<user-address>"
  }
}' --from <user> --chain-id <chain-id>
```

### Withdraw Assets

```bash
# Withdraw assets
wasmd tx wasm execute <vault-address> '{
  "withdraw": {
    "assets": "1000000",
    "receiver": "<user-address>",
    "owner": "<user-address>"
  }
}' --from <user> --chain-id <chain-id>
```

### Query Vault State

```bash
# Get vault info
wasmd query wasm contract-state smart <vault-address> '{"asset": {}}'

# Get total assets
wasmd query wasm contract-state smart <vault-address> '{"total_assets": {}}'

# Get user balance
wasmd query wasm contract-state smart <vault-address> '{"balance": {"address": "<user-address>"}}'
```

## Security Considerations

### Access Control

- **Manager role**: Can update vault configuration
- **Oracle role**: Can update price feeds
- **Owner**: Can transfer ownership

### Slippage Protection

- Configured slippage tolerance prevents excessive losses
- Oracle price validation ensures accurate pricing

### Emergency Procedures

- Owner can pause operations if needed
- Manager can adjust parameters in emergency

## Monitoring

### Key Metrics

- Total assets under management
- Share price (assets/shares ratio)
- LP position performance
- Reward accumulation

### Events to Track

- Deposit/withdrawal events
- LP provision/withdrawal
- Price updates
- Role changes

## Troubleshooting

### Common Issues

1. **Insufficient allowance**: Approve token spending before deposit
2. **Slippage exceeded**: Check current market conditions
3. **Oracle price stale**: Ensure price feeds are updated
4. **Insufficient balance**: Check user token balances

### Debug Commands

```bash
# Check contract state
wasmd query wasm contract-state raw <vault-address>

# View contract logs
wasmd query wasm contract-state smart <vault-address> '{"config": {}}'
```

## Next Steps

1. **Test on testnet** before mainnet deployment
2. **Set up monitoring** for vault performance
3. **Configure frontend** for user interaction
4. **Implement automation** for LP management
5. **Set up alerts** for critical events

## Support

For issues and questions:

- Check test logs and contract state
- Review error messages in transaction responses
- Consult CosmWasm documentation
- Review contract source code
