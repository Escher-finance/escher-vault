# Use official Nix image
FROM nixos/nix:2.31.0

# Set working directory
WORKDIR /workspace

# Install all tools in one go using a shell.nix approach
RUN nix-env -iA nixpkgs.bash nixpkgs.curl nixpkgs.jq nixpkgs.binaryen nixpkgs.rustc nixpkgs.cargo nixpkgs.go nixpkgs.gcc nixpkgs.pkg-config nixpkgs.openssl nixpkgs.lld

# Set up environment
ENV PATH="/root/.cargo/bin:/root/go/bin:$PATH"
ENV CARGO_HOME="/root/.cargo"

# Configure git to avoid authentication prompts
RUN git config --global url."https://".insteadOf git://

# Copy project files
COPY . .

# Add wasm32 target to Rust
RUN rustc --target wasm32-unknown-unknown --print target-libdir > /dev/null || echo "WASM target available"

# Try to install babylond, but don't fail if it doesn't work
RUN go install github.com/babylonchain/babylon/cmd/babylond@latest || echo "babylond installation failed, will use system version"

# Set default command to use Nix bash
CMD ["bash"]
