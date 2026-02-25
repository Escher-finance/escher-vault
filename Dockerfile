# Use official Nix image with latest stable version
FROM nixos/nix:2.22.0

# Set working directory
WORKDIR /workspace

COPY flake.nix flake.lock .

# Enable nix experimental features
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

# Install Rust toolchain with clippy
RUN nix develop .# --command bash -c "rustc --version && cargo --version"

# Install additional tools that might not be in the flake
# RUN nix-env -iA nixpkgs.bash nixpkgs.curl nixpkgs.jq nixpkgs.binaryen nixpkgs.go nixpkgs.gcc nixpkgs.pkg-config nixpkgs.openssl nixpkgs.lld

# Set up environment
ENV PATH="/root/.cargo/bin:/root/go/bin:$PATH"
ENV CARGO_HOME="/root/.cargo"

# Add wasm32 target to Rust
# RUN nix develop --command bash -c "rustc --target wasm32-unknown-unknown --print target-libdir > /dev/null || echo 'WASM target available'"

# Try to install babylond, but don't fail if it doesn't work
# RUN go install github.com/babylonchain/babylon/cmd/babylond@latest || echo "babylond installation failed, will use system version"

RUN git config --global --add safe.directory /workspace

COPY . .

CMD ["sleep", "infinity"]
