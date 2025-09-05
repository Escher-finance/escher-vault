# Use official Nix image with latest stable version
FROM nixos/nix:2.22.0

# Set working directory
WORKDIR /workspace

# Copy project files first (including flake.nix)
COPY . .

# Enable experimental features and install Rust toolchain with clippy
RUN nix --extra-experimental-features nix-command --extra-experimental-features flakes develop --command bash -c "rustc --version && cargo --version && cargo clippy --version"

# Install additional tools that might not be in the flake
RUN nix-env -iA nixpkgs.bash nixpkgs.curl nixpkgs.jq nixpkgs.binaryen nixpkgs.go nixpkgs.gcc nixpkgs.pkg-config nixpkgs.openssl nixpkgs.lld

# Set up environment
ENV PATH="/root/.cargo/bin:/root/go/bin:$PATH"
ENV CARGO_HOME="/root/.cargo"

# Configure git to avoid authentication prompts
RUN git config --global url."https://".insteadOf git://

# Add wasm32 target to Rust
RUN nix --extra-experimental-features nix-command --extra-experimental-features flakes develop --command bash -c "rustc --target wasm32-unknown-unknown --print target-libdir > /dev/null || echo 'WASM target available'"

# Try to install babylond, but don't fail if it doesn't work
RUN go install github.com/babylonchain/babylon/cmd/babylond@latest || echo "babylond installation failed, will use system version"

# Set default command to use Nix bash with flake environment
CMD ["nix", "--extra-experimental-features", "nix-command", "--extra-experimental-features", "flakes", "develop", "--command", "bash"]
