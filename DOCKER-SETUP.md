# 🐳 Docker + Nix Setup Guide for CW4626 Vault

## 🎯 Overview

This guide covers setting up a **Docker-based Nix environment** for developing
and deploying CW4626 vault contracts. This approach gives you all the benefits
of Nix (reproducible builds, dependency management) without the complexity of
installing Nix directly on macOS.

## ✨ What You Get

- **🐧 Nix 2.31.0** - Package manager and build system
- **🦀 Rust 1.86.0** - Programming language and toolchain
- **📦 Cargo** - Rust package manager
- **⚡ wasm-opt 123** - WebAssembly optimizer
- **🐐 Go** - For building babylond (if needed)
- **🔧 Build tools** - gcc, pkg-config, openssl, lld
- **📱 WASM target** - For CosmWasm smart contracts

## 🚀 Quick Start

### 1. **Prerequisites**

- Docker Desktop installed and running
- Git (for cloning the repository)

### 2. **Start the Environment**

```bash
# Start the Docker + Nix environment
./scripts/dev-docker.sh

# Or manually:
docker-compose up -d
```

### 3. **Enter the Container**

```bash
docker-compose exec cw4626-nix bash
```

### 4. **Build Contracts**

```bash
# Inside the container
cargo build --package cw4626-escher --lib --target wasm32-unknown-unknown --release
```

### 5. **Optimize WASM Files**

```bash
# Inside the container
wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_escher.wasm -o target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm
```

## 📁 File Structure

```
cw-vault/
├── Dockerfile                 # Nix environment definition
├── docker-compose.yml         # Container orchestration
├── .dockerignore             # Files to exclude from build
├── scripts/
│   ├── dev-docker.sh         # Development environment script
│   └── deploy-docker.sh      # Deployment script
└── target/                   # Build outputs (mounted as volume)
    └── wasm32-unknown-unknown/release/
        └── cw4626_escher_optimized.wasm
```

## 🔧 Available Commands

### **Container Management**

```bash
# Start environment
docker-compose up -d

# Stop environment
docker-compose down

# View logs
docker-compose logs -f

# Rebuild and restart
docker-compose up -d --build

# Check status
docker-compose ps
```

### **Development Commands**

```bash
# Enter container
docker-compose exec cw4626-nix bash

# Build contracts
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo build --package cw4626-escher --lib --target wasm32-unknown-unknown --release"

# Run tests
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo test"

# Generate schema
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo schema"
```

### **Deployment Commands**

```bash
# Run deployment script
./scripts/deploy-docker.sh

# Check contract sizes
docker-compose exec cw4626-nix bash -c "cd /workspace && ls -lh target/wasm32-unknown-unknown/release/*_optimized.wasm"
```

## 📊 Contract Sizes

After optimization with `wasm-opt -Os`:

| Contract      | Original | Optimized | Reduction |
| ------------- | -------- | --------- | --------- |
| cw4626-escher | 665K     | 519K      | 22%       |

## 🌟 Benefits of This Setup

### **✅ Advantages**

- **Reproducible builds** - Same environment every time
- **No system conflicts** - Isolated from host system
- **Easy sharing** - Team members get identical environments
- **Version control** - Environment defined in code
- **Cross-platform** - Works on macOS, Linux, Windows
- **No SIP issues** - Bypasses macOS System Integrity Protection

### **⚠️ Considerations**

- **Docker dependency** - Requires Docker Desktop
- **Resource usage** - Container uses system resources
- **Network access** - Container needs internet for package downloads
- **Volume mounting** - Host files are accessible in container

## 🔍 Troubleshooting

### **Container Won't Start**

```bash
# Check Docker status
docker info

# Check container logs
docker-compose logs

# Rebuild container
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

### **Build Failures**

```bash
# Check if container is running
docker-compose ps

# Enter container and check tools
docker-compose exec cw4626-nix bash
rustc --version
cargo --version
wasm-opt --version
```

### **Permission Issues**

```bash
# Fix file permissions
chmod +x scripts/*.sh

# Check Docker volumes
docker volume ls
```

## 🚀 Next Steps

1. **Test the environment** - Build contracts and verify everything works
2. **Configure deployment** - Set up your Babylon testnet configuration
3. **Deploy contracts** - Use the optimized WASM files with babylond
4. **Monitor and test** - Verify contract functionality on testnet

## 📚 Related Documentation

- [NIX-SETUP.md](./NIX-SETUP.md) - Alternative Nix installation methods
- [BABYLON-DEPLOYMENT.md](./BABYLON-DEPLOYMENT.md) - Babylon-specific deployment
- [DEPLOYMENT.md](./DEPLOYMENT.md) - General deployment guide

## 🆘 Support

If you encounter issues:

1. **Check container logs**: `docker-compose logs -f`
2. **Verify Docker status**: `docker info`
3. **Rebuild container**: `docker-compose up -d --build`
4. **Check file permissions**: Ensure scripts are executable

---

**🎉 Congratulations!** You now have a fully functional Docker + Nix environment
for CW4626 vault development and deployment.
