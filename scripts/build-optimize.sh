#!/usr/bin/env bash

# Build the contract (lib only to avoid schema binary issues)
cargo build --release --target wasm32-unknown-unknown --package cw4626-escher --lib

# Create artifacts directory
mkdir -p artifacts

# Optimize the WASM file with aggressive size optimization
wasm-opt -Oz --signext-lowering --strip-debug --strip-producers "target/wasm32-unknown-unknown/release/cw4626_escher.wasm" -o "artifacts/cw4626_escher.wasm"

echo "✅ Contract built and optimized successfully!"
echo "📁 WASM file: artifacts/cw4626_escher.wasm"
