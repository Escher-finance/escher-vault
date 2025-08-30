# CW4626 Vault - LP Staking on Astroport

A sophisticated vault system for automated liquidity provision (LP) staking on Astroport forks, implementing the CW4626 standard for tokenized vaults on Cosmos blockchains.

## 🚀 Overview

This project provides two vault implementations:

1. **`cw4626-base`** - Basic vault functionality following the CW4626 standard
2. **`cw4626-escher`** - Enhanced vault with automated LP management and incentives

## ✨ Features

### Core Vault Functionality
- **Deposit/Withdraw**: Users can deposit underlying tokens and receive vault shares
- **Mint/Redeem**: Exact share minting and redemption
- **CW20 Compliance**: Vault shares are standard CW20 tokens
- **Ownership Management**: Secure access control with `cw-ownable`

### LP Automation (Escher Contract)
- **Automated Liquidity Provision**: Automatic LP to Astroport pairs
- **Incentive Management**: Integration with Astroport's reward system
- **Oracle Integration**: Real-time price feeds for optimal LP management
- **Slippage Protection**: Configurable slippage tolerance
- **Role-Based Access**: Manager and Oracle roles for secure operations

## 🏗️ Architecture

```
User → Vault → Astroport LP Pair → Rewards & Incentives
  ↓         ↓           ↓
Shares   Auto-LP    Fee Collection
```

### How It Works
1. **User deposits** underlying tokens (e.g., USDC)
2. **Vault automatically** provides liquidity to Astroport pairs
3. **User receives** vault shares representing their LP position
4. **Vault earns** LP rewards, trading fees, and incentives
5. **User can redeem** shares for underlying tokens + accumulated rewards

## 📦 Project Structure

```
cw-vault/
├── packages/
│   └── cw4626/           # Core specification and messages
├── contracts/
│   ├── cw4626-base/      # Basic vault implementation
│   └── cw4626-escher/    # Enhanced LP automation vault
├── scripts/               # Build and deployment scripts
├── schema/                # Generated JSON schemas
└── tests/                 # Integration tests
```

## 🛠️ Prerequisites

- **Rust** 1.70+
- **wasm-opt** (binaryen)
- **CosmWasm** compatible blockchain
- **Astroport fork** (babydex)

## 🔧 Installation & Build

### 1. Clone the Repository
```bash
git clone https://github.com/your-org/cw-vault.git
cd cw-vault
```

### 2. Install Dependencies
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-opt
brew install binaryen  # macOS
# or
sudo apt install binaryen  # Ubuntu
```

### 3. Build Contracts
```bash
# Build all contracts
cargo wasm -p cw4626-base
cargo wasm -p cw4626-escher

# Generate schemas
cargo schema -p cw4626-base
cargo schema -p cw4626-escher
```

### 4. Run Tests
```bash
# Run all tests
cargo test

# Or use the test script
./scripts/test-vault.sh
```

## 🚀 Deployment

### Quick Deployment
```bash
# Deploy to testnet
./scripts/deploy.sh testnet escher my-key

# Deploy to mainnet
./scripts/deploy.sh mainnet escher my-key
```

### Manual Deployment
```bash
# 1. Upload contract code
wasmd tx wasm store target/wasm32-unknown-unknown/release/cw4626_escher.wasm \
  --from <key> --chain-id <chain-id> --gas auto --gas-adjustment 1.3

# 2. Instantiate contract
wasmd tx wasm instantiate <code-id> '{
  "underlying_token_address": "<cw20-token-address>",
  "share_name": "Escher USDC",
  "share_symbol": "eUSDC",
  "manager": "<manager-address>",
  "oracle": "<oracle-address>",
  "tower_incentives": "<tower-incentives-address>",
  "lp": "<astroport-pair-address>",
  "slippage_tolerance": "0.01",
  "incentives": [{"native_token": {"denom": "uatom"}}]
}' --from <key> --chain-id <chain-id>
```

## 📚 Usage Examples

### Deposit Assets
```bash
# 1. Approve spending
wasmd tx wasm execute <cw20-token> '{
  "increase_allowance": {
    "spender": "<vault-address>",
    "amount": "1000000"
  }
}' --from <user> --chain-id <chain-id>

# 2. Deposit to vault
wasmd tx wasm execute <vault-address> '{
  "deposit": {
    "assets": "1000000",
    "receiver": "<user-address>"
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

## 🔒 Security Features

- **Access Control**: Role-based permissions for managers and oracles
- **Slippage Protection**: Configurable maximum slippage tolerance
- **Oracle Validation**: Price feed verification and validation
- **Ownership Management**: Secure transfer and management of vault ownership
- **Input Validation**: Comprehensive parameter validation and bounds checking

## 📊 Monitoring

### Key Metrics
- Total assets under management
- Share price (assets/shares ratio)
- LP position performance
- Reward accumulation rate

### Events to Track
- Deposit/withdrawal events
- LP provision/withdrawal
- Price updates
- Role changes

## 🧪 Testing

### Run All Tests
```bash
cargo test
```

### Run Specific Tests
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# Specific test
cargo test test_name
```

### Test Coverage
```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate coverage report
cargo cov

# Open coverage in browser
cargo cov --open
```

## 🔧 Development

### Adding New Features
1. **Update messages** in `packages/cw4626/src/msg.rs`
2. **Implement logic** in contract files
3. **Add tests** for new functionality
4. **Update schemas** with `cargo schema`

### Code Quality
```bash
# Check for warnings
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## 📖 Documentation

- **Deployment Guide**: See `DEPLOYMENT.md`
- **API Reference**: Generated schemas in `schema/` directory
- **Integration Guide**: Examples and usage patterns

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Support

- **Issues**: Create an issue on GitHub
- **Documentation**: Check the `DEPLOYMENT.md` and generated schemas
- **Community**: Join our Discord/Telegram for discussions

## 🔗 Related Links

- **CW4626 Standard**: [Specification](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-4626.md)
- **CosmWasm**: [Documentation](https://docs.cosmwasm.com/)
- **Astroport**: [Protocol](https://astroport.fi/)
- **BabyDEX**: [Fork Repository](https://github.com/quasar-finance/babydex)

---

**Built with ❤️ for the Cosmos ecosystem**
