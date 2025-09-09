{
  description = "CW4626 Vault - LP Staking on Astroport";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        astroportSrc = pkgs.fetchFromGitHub {
          owner = "quasar-finance";
          repo = "babydex";
          rev = "8fce1b955a1769a1f4286c73cbfd36701753ac1e";
          sha256 = pkgs.lib.fakeSha256; # TODO: replace with fixed hash
        };

        # Rust toolchain with specific version
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # Development environment
        devEnv = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # WebAssembly tools
            binaryen
            wasm-pack

            # Babylon CLI (if available in nixpkgs)
            # babylond

            # Development tools
            jq
            curl
            git
            pkg-config

            # Additional tools
            nodejs_20
            yarn
          ];

          # Environment variables
          shellHook = ''
            echo "🚀 CW4626 Vault Development Environment"
            echo "======================================"
            echo "✅ Rust: $(rustc --version)"
            echo "✅ Cargo: $(cargo --version)"
            echo "✅ wasm-opt: $(wasm-opt --version)"
            echo "✅ Node.js: $(node --version)"
            echo ""
            echo "🔧 Available commands:"
            echo "  - cargo build    # Build contracts"
            echo "  - cargo wasm     # Build WebAssembly"
            echo "  - cargo test     # Run tests"
            echo "  - cargo schema   # Generate schemas"
            echo ""
            echo "📚 Next steps:"
            echo "1. Build contracts: ./scripts/build-optimize.sh"
            echo "2. Run tests: cargo test"
          '';

          # Rust environment
          RUST_BACKTRACE = "1";
          RUST_LOG = "info";

          # WebAssembly target
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER = "wasm-opt";
        };

      in
      {
        # Development shell
        devShells.default = devEnv;

        # Build outputs
        packages = {
          # CI-friendly build that does not rely on devShell (no shellHook execution)
          cw4626-escher = pkgs.stdenv.mkDerivation {
            name = "cw4626-escher";
            src = ./.;
            # Tools required for build/optimization
            nativeBuildInputs = [
              rustToolchain
              pkgs.binaryen
              pkgs.wasm-pack
              pkgs.jq
              pkgs.curl
              pkgs.git
              pkgs.pkg-config
              pkgs.nodejs_20
              pkgs.yarn
            ];
            buildPhase = ''
              # Ensure cargo/git have a writable home on CI builders (e.g., garnix)
              export HOME="$TMPDIR"
              export CARGO_HOME="$TMPDIR/.cargo"
              mkdir -p "$CARGO_HOME"

              export RUSTFLAGS="-C target-feature=-reference-types"
              # Use git CLI for fetching git deps (more reliable in some CI envs)
              export CARGO_NET_GIT_FETCH_WITH_CLI=true

              # Instruct Cargo to use vendored paths for astroport crates
              mkdir -p "$CARGO_HOME"
              cat > "$CARGO_HOME/config.toml" <<'CFG'
              [patch.'https://github.com/quasar-finance/babydex.git']
              astroport = { path = "${astroportSrc}/packages/astroport" }
              astroport-factory = { path = "${astroportSrc}/contracts/astroport-factory" }
              astroport-pcl-common = { path = "${astroportSrc}/packages/astroport-pcl-common" }
              astroport-pair-concentrated = { path = "${astroportSrc}/contracts/astroport-pair-concentrated" }
              CFG

              cargo build --release --lib --target wasm32-unknown-unknown -p cw4626-escher
              mkdir -p artifacts
              wasm-opt -Oz --signext-lowering --strip-debug --strip-producers \
                target/wasm32-unknown-unknown/release/cw4626_escher.wasm \
                -o artifacts/cw4626_escher.wasm
            '';
            installPhase = ''
              mkdir -p $out
              cp artifacts/cw4626_escher.wasm $out/
            '';
          };
        };

        # Default package
        packages.default = self.packages.${system}.cw4626-escher;
      }
    );
}
