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

## 🏗️ Architecture

### How It Works

1. **User deposits** underlying token
2. **User receives** vault shares representing their position
3. **Vault manager**
   - Stakes in the Escher Hub (from the same or other network via ZKGM)
   - Provides liquidity to Astroport pairs
   - Handles the Vault's position
4. **Vault earns** LP rewards, trading fees, and incentives
5. **User can redeem** shares for underlying tokens + accumulated rewards

---

## 🚀 **Option 1: Quick start with Docker + Nix (recommended)**

### 1. **Start Docker**

```bash
./scripts/dev-docker.sh
```

### 2. **Build Contracts with Nix through Docker**

```bash
docker-compose exec cw4626-nix ./scripts/nix-bash.sh ./scripts/build-optimize.sh

# To instead enter into an interactive shell run `nix-bash.sh` without any arguments. Like this:
docker-compose exec cw4626-nix ./scripts/nix-bash.sh
```

### 3. **Run Tests**

```bash
docker-compose exec cw4626-nix ./scripts/nix-bash.sh cargo test
```

## 🐧 **Option 2: Quick start with Nix**

### 2. **Enter Nix Environment**

```bash
./scripts/nix-bash.sh
```

### 3. **Build Contracts**

```bash
./scripts/build-optimize.sh
```

### 4. **Run Tests**

```bash
cargo test
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

# Build optimized version
./scripts/build-optimize.sh
```

### 3. **Run Tests**

```bash
cargo test
```

---

## 🚀 Deployment

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
  ...
}' --from <key> --chain-id <chain-id>
```

## 🧪 Testing

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

### **Code Quality**

```bash
# Check for warnings
cargo check

# Format code
cargo fmt

# Lint code with Clippy
cargo clippy --workspace -- -D warnings
```

## 📖 Deployment Code Verification

- **🚀 Deployment Guide**: See `DEPLOYMENT.md`
- **✅ Deployment Code Verification**: See `VERIFY.md`
- **API Reference**: Generated schemas in `schema/` directory

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the `LICENSE` file for
details.

## 🆘 Support

- **Issues**: Create an issue on GitHub
- **Documentation**: Check the setup guides and generated schemas
- **Community**: Join our Discord/Telegram for discussions

## 🔗 Related Links

- **EIP4626 Standard**:
  [Specification](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-4626.md)
- **CosmWasm**: [Documentation](https://docs.cosmwasm.com/)
- **Astroport**: [Protocol](https://docs.astroport.fi/)
- **Union**: [Protocol](https://docs.union.build/)
- **TowerFi/BabyDEX**: [Repository](https://github.com/quasar-finance/babydex)

---

**Built with ❤️ for the Cosmos ecosystem using Docker and Nix**
