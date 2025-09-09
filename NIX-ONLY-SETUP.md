# 🐧 Nix-Only Setup Guide for CW4626 Vault

## 🎯 Overview

This guide shows how to use the CW4626 vault system with **Nix only** (no Docker required). Perfect for users who already have Nix installed and prefer native development.

## ✨ What You Get

- **🐧 Nix 2.31.0+** - Package manager and build system
- **🦀 Rust 1.86.0** - Latest stable toolchain
- **⚡ wasm-opt 123** - WebAssembly optimizer
- **🔧 Build tools** - gcc, pkg-config, openssl, lld
- **📱 WASM target** - For CosmWasm smart contracts
- **🌌 Babylon tools** - For deployment (if available)

## 🚀 Quick Start

### 1. **Prerequisites**
- **Nix** installed and working
- **Git** for cloning the repository

### 2. **Clone and Enter Nix Environment**
```bash
# Clone repository
git clone https://github.com/your-org/cw-vault.git
cd cw-vault

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

## 🔧 Available Commands

### **Development Commands**
```bash
# Build contracts
cargo build --package cw4626-escher --lib --target wasm32-unknown-unknown --release

# Run tests
cargo test

# Generate schemas
cargo schema

# Check code
cargo check
cargo fmt
cargo clippy
```

### **WASM Commands**
```bash
# Build WASM
cargo wasm -p cw4626-escher

# Optimize WASM
wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_escher.wasm -o optimized.wasm
```

### **Deployment Commands**
```bash
# Use existing deployment scripts
./scripts/deploy-babylon-env.sh
./scripts/deploy-babylon-pool.sh

# Or deploy manually with optimized files
# Files: target/wasm32-unknown-unknown/release/*_optimized.wasm
```

## 📁 File Structure

```
cw-vault/
├── flake.nix                 # Nix environment definition
├── scripts/                  # Build and deployment scripts
├── target/                   # Build outputs
│   └── wasm32-unknown-unknown/release/
│       └── cw4626_escher_optimized.wasm
└── contracts/                # Smart contract source code
```

## 🎯 Complete Workflow Example

```bash
# 1. Enter Nix environment
nix develop

# 2. Build contracts
cargo build --package cw4626-escher --lib --target wasm32-unknown-unknown --release

# 3. Run tests
cargo test

# 4. Optimize WASM
wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_escher.wasm -o target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm

# 5. Deploy
./scripts/deploy-babylon-env.sh
```

## 🔍 Troubleshooting

### **Nix Environment Issues**
```bash
# Check Nix status
nix --version
nix flake --version

# Update Nix
nix upgrade-nix

# Clear Nix cache
nix store gc
```

### **Build Issues**
```bash
# Check tools
rustc --version
cargo --version
wasm-opt --version

# Clean build
cargo clean
cargo build
```

### **WASM Issues**
```bash
# Check WASM target
rustup target list | grep wasm

# Add WASM target
rustup target add wasm32-unknown-unknown

# Verify target
rustc --target wasm32-unknown-unknown --print target-libdir
```

## 🌟 Benefits of Nix-Only Setup

### **✅ Advantages**
- **🐧 Native Nix** - Full Nix experience
- **🚀 Faster builds** - No Docker overhead
- **💾 Less disk space** - No container images
- **🔧 Direct access** - All tools available directly
- **📱 Better integration** - Works with your existing setup

### **⚠️ Considerations**
- **System dependencies** - May conflict with existing tools
- **Version management** - Need to manage Nix versions
- **Platform specific** - May not work on all systems

## 🔄 Alternative: Hybrid Approach

If you want the best of both worlds:

```bash
# Use Nix for development
nix develop

# Use Docker for deployment
./scripts/deploy-docker.sh
```

## 📚 Related Documentation

- **🐳 Docker + Nix**: See `DOCKER-SETUP.md`
- **🚀 Deployment**: See `DEPLOYMENT.md`
- **🌌 Babylon**: See `BABYLON-DEPLOYMENT.md`
- **📋 Quick Reference**: See `NIX-QUICK-REFERENCE.md`

## 🆘 Support

### **Common Issues**
1. **Nix not found**: Install Nix first
2. **WASM target missing**: Run `rustup target add wasm32-unknown-unknown`
3. **Build failures**: Check `cargo clean` and rebuild
4. **Permission issues**: Check file permissions

### **Getting Help**
- **Check Nix status**: `nix doctor`
- **Verify environment**: `nix develop --command env`
- **Check tools**: `which rustc && which cargo && which wasm-opt`

---

## 🎯 **Getting Started Checklist (Nix-Only)**

- [ ] Install Nix (if not already installed)
- [ ] Clone repository
- [ ] Run `nix develop`
- [ ] Build contracts with Nix
- [ ] Run tests with Nix
- [ ] Optimize WASM files
- [ ] Deploy to testnet

---

**🎉 Your Nix-only environment is ready for development and deployment!**
