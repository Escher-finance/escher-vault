# 🐧 Nix Setup Guide for CW4626 Vault

This guide shows you how to use Nix for a reproducible development environment with all the tools needed for Babylon testnet deployment.

## 🎯 **Why Nix?**

- **Reproducible Environment** - Same setup on any machine
- **Dependency Management** - All tools in one place
- **Isolation** - No conflicts with system packages
- **Team Consistency** - Everyone gets identical environment
- **Easy Tool Installation** - No manual installation needed

## 🚀 **Quick Start**

### **1. Install Nix (if not already installed)**
```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://get.determinate.systems/nix | sh -s - install

# Or use the official installer
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### **2. Enable Flakes (if not already enabled)**
```bash
# Add to ~/.config/nix/nix.conf
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

### **3. Enter Development Environment**
```bash
# Enter Nix development shell
nix develop

# Or with direnv (recommended)
direnv allow
```

## 🔧 **What's Included in the Nix Environment**

### **Core Development Tools**
- **Rust Toolchain** - Latest stable with WASM target
- **WebAssembly Tools** - binaryen, wasm-pack
- **Development Tools** - jq, curl, git, pkg-config
- **CosmWasm Tools** - cosmwasm-check
- **Node.js** - For frontend development

### **Environment Variables**
- **Rust Configuration** - RUST_BACKTRACE, RUST_LOG
- **WASM Target** - wasm32-unknown-unknown
- **Babylon Configuration** - Chain ID, RPC endpoints, pool address

## 📁 **Nix Files Structure**

```
cw-vault/
├── flake.nix              # Main Nix configuration
├── .envrc                  # Direnv configuration
├── scripts/
│   ├── deploy-babylon-nix.sh    # Nix-aware deployment
│   └── deploy-babylon-env.sh    # Environment-based deployment
└── NIX-SETUP.md           # This guide
```

## 🚀 **Using Nix for Development**

### **Enter Development Environment**
```bash
# Option 1: Direct Nix command
nix develop

# Option 2: With direnv (auto-loads when you cd into project)
direnv allow
cd cw-vault  # Environment automatically loads
```

### **Build Contracts**
```bash
# Build all contracts
cargo wasm -p cw4626-escher

# Or use Nix packages
nix build .#cw4626-escher
```

### **Run Tests**
```bash
# All tests
cargo test

# Specific tests
cargo test --test integration
```

### **Generate Schemas**
```bash
cargo schema -p cw4626-escher
```

## 🌍 **Environment Configuration**

### **Create .env File**
```bash
# Copy example and edit
cp env.example .env
nano .env
```

### **Example .env Content**
```bash
# Babylon Testnet Configuration
BABYLON_CHAIN_ID=bbn-test-5
BABYLON_RPC_NODE=https://babylon-testnet-rpc.polkachu.com
BABYLON_GAS_PRICES=0.025ubbn
BABYLON_POOL_ADDRESS=bbn1hkmstu883spzwj4k92g90fga3jv3n7ywswn6yr5nq3j4gas
BABYLON_KEY_NAME=my-key
BABYLON_UNDERLYING_TOKEN=bbn1...
BABYLON_MANAGER=bbn1...
BABYLON_ORACLE=bbn1...
BABYLON_TOWER_INCENTIVES=bbn1...
```

## 🚀 **Deployment with Nix**

### **1. Check Nix Environment**
```bash
./scripts/deploy-babylon-nix.sh
```

### **2. Deploy with Environment Variables**
```bash
./scripts/deploy-babylon-env.sh
```

### **3. Manual Deployment**
```bash
# Upload contract
babylond tx wasm store target/wasm32-unknown-unknown/release/cw4626_escher.wasm \
  --from $BABYLON_KEY_NAME \
  --chain-id $BABYLON_CHAIN_ID \
  --node $BABYLON_RPC_NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $BABYLON_GAS_PRICES \
  --yes
```

## 🔧 **Adding babylond to Nix**

### **Option 1: Install via Go (Recommended)**
```bash
# Install Go in Nix environment
nix-env -iA nixpkgs.go

# Install babylond
go install github.com/babylonlabs/babylon/cmd/babylond@latest

# Add to PATH
export PATH=$PATH:$HOME/go/bin
```

### **Option 2: Add to Nix Flake**
```nix
# In flake.nix, add to buildInputs:
buildInputs = with pkgs; [
  # ... existing inputs ...
  
  # Babylon CLI
  (pkgs.buildGoModule rec {
    pname = "babylond";
    version = "v0.5.0";
    src = pkgs.fetchFromGitHub {
      owner = "babylonlabs";
      repo = "babylon";
      rev = version;
      sha256 = "sha256-..."; # You'll need to get this
    };
    vendorSha256 = "sha256-..."; # You'll need to get this
  });
];
```

## 🧪 **Testing Your Nix Setup**

### **Verify Tools**
```bash
# Check if tools are available
rustc --version
cargo --version
wasm-opt --version
jq --version
node --version
```

### **Test Build**
```bash
# Clean and rebuild
cargo clean
cargo wasm -p cw4626-escher
```

### **Test Deployment Script**
```bash
./scripts/deploy-babylon-nix.sh
```

## 🔄 **Updating Nix Environment**

### **Update Flake**
```bash
# Update inputs
nix flake update

# Rebuild environment
nix develop
```

### **Clean and Rebuild**
```bash
# Clean Nix store
nix store gc

# Rebuild development environment
nix develop
```

## 🚨 **Troubleshooting**

### **Common Issues**

1. **Flakes not enabled**
   ```bash
   echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
   ```

2. **Permission denied**
   ```bash
   chmod +x scripts/*.sh
   ```

3. **Tools not found**
   ```bash
   # Re-enter Nix environment
   nix develop
   ```

4. **Build failures**
   ```bash
   # Clean and rebuild
   cargo clean
   nix develop
   cargo build
   ```

### **Getting Help**
- **Nix Documentation**: https://nixos.org/guides/
- **Flakes Guide**: https://nixos.wiki/wiki/Flakes
- **Rust Overlay**: https://github.com/oxalica/rust-overlay

## 🎉 **Benefits of This Setup**

1. **Reproducible** - Same environment everywhere
2. **Isolated** - No system conflicts
3. **Comprehensive** - All tools included
4. **Maintainable** - Easy to update and modify
5. **Team-Friendly** - Consistent across developers

---

**Happy developing with Nix! 🐧**
