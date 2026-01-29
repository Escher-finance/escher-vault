# CW4626 Vault - LP Staking on Astroport

A sophisticated vault system for automated liquidity provision (LP) staking on
Astroport forks, implementing the CW4626 standard for tokenized vaults on Cosmos
blockchains.

## 🚀 Overview

This project provides the following vault implementations:

1. **`cw4626-escher`** - Enhanced vault with automated LP management, liquid
   staking and incentives

## ✨ Features

### Core Vault Functionality

- **Deposit**: Users can deposit underlying tokens and receive vault shares
- **Redeem**: Exact share minting and redemption
- **CW20 Compliance**: Vault shares are standard CW20 tokens while the
  underlying asset can be CW20 or native
- **Escher Hub Integration**: Liquid stake on the Escher Hub in the same chain
  or across chains

### Management and security

- **Incentive Management**: Integration with Astroport's reward system
- **Oracle Integration**: Real-time price feeds for optimal LP management
- **Slippage Protection**: Configurable slippage tolerance
- **Role-Based Access**: Manager and Oracle roles for secure operations
- **Ownership Management**: Secure transfer and management of vault ownership
- **Input Validation**: Comprehensive parameter validation and bounds checking

### How It Works

1. **User deposits** underlying token
2. **User receives** vault shares representing their position
3. **Vault manager**
   - Stakes in the Escher Hub
   - Provides liquidity to Astroport pairs
   - Handles the Vault's position
4. **Vault earns** LP rewards, trading fees, and incentives
5. **User can redeem** shares for underlying tokens + accumulated rewards

## 🛠️ Prerequisites

### 🐳 **Option 1: Docker + Nix (Recommended for most users)**

- **Docker Desktop** installed and running
- **Git** for cloning the repository

### 🐧 **Option 2: Nix Only (For Nix users)**

- **Nix** installed and working
- **Git** for cloning the repository

### 🔧 **Option 3: Traditional Setup (Fallback)**

- **Rust** 1.70+
- **wasm-opt** (binaryen)
- **CosmWasm** compatible blockchain
- **Astroport fork** (babydex)

---

## 🚀 **Quick Start with Docker + Nix (Recommended)**

### 1. **Clone and Setup**

```bash
git clone https://github.com/your-org/cw-vault.git
cd cw-vault
```

### 2. **Start Nix Environment**

```bash
# Start the Docker + Nix environment
./scripts/dev-docker.sh

# Or manually:
docker-compose up -d
```

### 3. **Build Contracts with Nix**

```bash
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo build --package cw4626-escher --lib --target wasm32-unknown-unknown --release"
```

### 4. **Run Tests with Nix**

```bash
# Run all tests
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo test"

# Or use the test script
./scripts/test-vault.sh
```

### 5. **Optimize WASM Files**

```bash
# Optimize with Nix wasm-opt
docker-compose exec cw4626-nix bash -c "cd /workspace && wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_escher.wasm -o target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm"
```

### 6. **Deploy with Nix**

```bash
# Use the deployment script
./scripts/deploy-docker.sh

# Or manually deploy the optimized files:
# - target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm
```

---

## 🐧 **Option 2: Nix Only (No Docker Required)**

### 1. **Clone and Setup**

```bash
git clone https://github.com/your-org/cw-vault.git
cd cw-vault
```

### 2. **Enter Nix Environment**

```bash
# Enter Nix development environment
nix develop
```

### 3. **Build Contracts with Nix**

```bash
# Build both contracts
cargo build --package cw4626-escher --lib --target wasm32-unknown-unknown --release
```

### 4. **Run Tests with Nix**

```bash
# Run all tests
cargo test

# Or run specific packages
cargo test --package cw4626-escher
```

### 5. **Optimize WASM Files**

```bash
# Optimize with Nix wasm-opt
wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_escher.wasm -o target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm
```

### 6. **Deploy with Nix**

```bash
# Use existing deployment scripts
./scripts/deploy-babylon-env.sh

# Or manually deploy the optimized files:
# - target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm
```

---

## 🔧 **Option 3: Traditional Installation & Build**

### 1. **Install Dependencies**

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-opt
brew install binaryen  # macOS
# or
sudo apt install binaryen  # Ubuntu
```

### 2. **Build Contracts**

```bash
# Build all contracts
cargo wasm -p cw4626-escher

# Generate schemas
cargo schema -p cw4626-escher
```

### 3. **Run Tests**

```bash
# Run all tests
cargo test

# Or use the test script
./scripts/test-vault.sh
```

---

## 🚀 Deployment

### **Quick Deployment with Nix**

```bash
# Deploy to testnet
./scripts/deploy.sh testnet escher my-key

# Deploy to mainnet
./scripts/deploy.sh mainnet escher my-key
```

### **Manual Deployment**

```bash
# 1. Upload contract code
wasmd tx wasm store target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm \
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

### **Run Tests with Nix (Recommended)**

```bash
# All tests
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo test"

# Specific package
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo test --package cw4626-escher"
```

### **Run Tests Traditionally**

```bash
# All tests
cargo test

# Specific tests
cargo test --lib
cargo test --test integration
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

### **Development with Nix (Recommended)**

```bash
# Enter Nix environment
docker-compose exec cw4626-nix bash

# Inside container:
cargo check
cargo fmt
cargo clippy
cargo build
cargo test
```

### **Adding New Features**

1. **Update messages** in `packages/cw4626/src/msg.rs`
2. **Implement logic** in contract files
3. **Add tests** for new functionality

### **Code Quality**

```bash
# Check for warnings
cargo check

# Format code
cargo fmt

# Lint code with Clippy
cargo clippy --workspace -- -D warnings

# Or use the Clippy script
./scripts/clippy.sh
```

## 📖 Documentation

- **🐳 Docker + Nix Setup**: See `DOCKER-SETUP.md`
- **🐧 Nix Only Setup**: See `NIX-ONLY-SETUP.md`
- **🐧 Nix Setup**: See `NIX-SETUP.md`
- **🚀 Deployment Guide**: See `DEPLOYMENT.md`
- **✅ Deployment Code Verification**: See `VERIFY.md`
- **🌌 Babylon Deployment**: See `BABYLON-DEPLOYMENT.md`
- **📋 Quick Reference**: See `NIX-QUICK-REFERENCE.md`
- **API Reference**: Generated schemas in `schema/` directory
- **Integration Guide**: Examples and usage patterns

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for
details.

## 🆘 Support

- **Issues**: Create an issue on GitHub
- **Documentation**: Check the setup guides and generated schemas
- **Community**: Join our Discord/Telegram for discussions

## 🔗 Related Links

- **CW4626 Standard**:
  [Specification](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-4626.md)
- **CosmWasm**: [Documentation](https://docs.cosmwasm.com/)
- **Astroport**: [Protocol](https://astroport.fi/)
- **BabyDEX**: [Fork Repository](https://github.com/quasar-finance/babydex)

---

**Built with ❤️ for the Cosmos ecosystem using �� Nix + 🐳 Docker**
