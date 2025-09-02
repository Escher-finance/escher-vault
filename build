#!/usr/bin/env bash

cargo wasm -p cw4626-escher
mkdir artifacts 2>/dev/null
wasm-opt -O3 --signext-lowering "target/wasm32-unknown-unknown/release/cw4626_escher.wasm" -o "artifacts/cw4626_escher.wasm"
