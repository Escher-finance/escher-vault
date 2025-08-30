# 🚀 Nix Quick Reference - CW4626 Vault

## 🎯 **One-Command Setup**
```bash
# Start everything with one command
./scripts/dev-docker.sh
```

## 🔧 **Essential Commands**

### **Environment Management**
```bash
# Start Nix environment
docker-compose up -d

# Stop Nix environment
docker-compose down

# Check status
docker-compose ps

# View logs
docker-compose logs -f
```

### **Development Workflow**
```bash
# Enter Nix container
docker-compose exec cw4626-nix bash

# Build contracts
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo build --package cw4626-base --lib --target wasm32-unknown-unknown --release"
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo build --package cw4626-escher --lib --target wasm32-unknown-unknown --release"

# Run tests
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo test"

# Generate schemas
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo schema"
```

### **WASM Optimization**
```bash
# Optimize with Nix wasm-opt
docker-compose exec cw4626-nix bash -c "cd /workspace && wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_base.wasm -o target/wasm32-unknown-unknown/release/cw4626_base_optimized.wasm"
docker-compose exec cw4626-nix bash -c "cd /workspace && wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_escher.wasm -o target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm"

# Check file sizes
docker-compose exec cw4626-nix bash -c "cd /workspace && ls -lh target/wasm32-unknown-unknown/release/*_optimized.wasm"
```

### **Deployment**
```bash
# Use deployment script
./scripts/deploy-docker.sh

# Or deploy manually with optimized files
# Files: target/wasm32-unknown-unknown/release/*_optimized.wasm
```

## 📊 **What You Get with Nix**

| Tool | Version | Purpose |
|------|---------|---------|
| **🐧 Nix** | 2.31.0 | Package manager |
| **🦀 Rust** | 1.86.0 | Programming language |
| **⚡ wasm-opt** | 123 | WASM optimizer |
| **🔧 gcc** | 14.3.0 | C compiler |
| **🔗 lld** | 19.1.7 | Linker |
| **📦 pkg-config** | 0.29.2 | Build configuration |

## 🎯 **Complete Workflow Example**

```bash
# 1. Start environment
./scripts/dev-docker.sh

# 2. Build contracts
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo build --package cw4626-base --lib --target wasm32-unknown-unknown --release"
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo build --package cw4626-escher --lib --target wasm32-unknown-unknown --release"

# 3. Run tests
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo test"

# 4. Optimize WASM
docker-compose exec cw4626-nix bash -c "cd /workspace && wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_base.wasm -o target/wasm32-unknown-unknown/release/cw4626_base_optimized.wasm"
docker-compose exec cw4626-nix bash -c "cd /workspace && wasm-opt -Os target/wasm32-unknown-unknown/release/cw4626_escher.wasm -o target/wasm32-unknown-unknown/release/cw4626_escher_optimized.wasm"

# 5. Deploy
./scripts/deploy-docker.sh
```

## 🚨 **Troubleshooting**

### **Container Issues**
```bash
# Rebuild container
docker-compose down
docker-compose build --no-cache
docker-compose up -d

# Check container health
docker-compose exec cw4626-nix bash -c "echo 'Container is healthy'"
```

### **Build Issues**
```bash
# Check tools
docker-compose exec cw4626-nix bash -c "rustc --version && cargo --version && wasm-opt --version"

# Clean build
docker-compose exec cw4626-nix bash -c "cd /workspace && cargo clean && cargo build"
```

### **Permission Issues**
```bash
# Fix script permissions
chmod +x scripts/*.sh

# Check Docker volumes
docker volume ls
```

## 🎉 **Success Indicators**

✅ **Container running**: `docker-compose ps` shows "Up" status  
✅ **Builds working**: `cargo build` completes without errors  
✅ **Tests passing**: `cargo test` shows "test result: ok"  
✅ **WASM optimized**: File sizes reduced by ~22%  
✅ **Ready to deploy**: Optimized .wasm files generated  

---

**🚀 Your Nix environment is ready for production!**
