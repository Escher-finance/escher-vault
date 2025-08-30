{
  description = "CW4626 Vault - LP Staking on Astroport";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust toolchain with specific version
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
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
            
            # CosmWasm development
            cosmwasm-check
            
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
            echo "1. Build contracts: cargo wasm -p cw4626-escher"
            echo "2. Run tests: cargo test"
            echo "3. Deploy: ./scripts/deploy-babylon-env.sh"
          '';

          # Rust environment
          RUST_BACKTRACE = "1";
          RUST_LOG = "info";
          
          # WebAssembly target
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER = "wasm-opt";
        };

      in {
        # Development shell
        devShells.default = devEnv;
        
        # Build outputs
        packages = {
          # Build base contract
          cw4626-base = pkgs.stdenv.mkDerivation {
            name = "cw4626-base";
            src = ./.;
            buildInputs = [ devEnv ];
            buildPhase = ''
              cargo wasm -p cw4626-base
            '';
            installPhase = ''
              mkdir -p $out
              cp target/wasm32-unknown-unknown/release/cw4626_base.wasm $out/
            '';
          };

          # Build escher contract
          cw4626-escher = pkgs.stdenv.mkDerivation {
            name = "cw4626-escher";
            src = ./.;
            buildInputs = [ devEnv ];
            buildPhase = ''
              cargo wasm -p cw4626-escher
            '';
            installPhase = ''
              mkdir -p $out
              cp target/wasm32-unknown-unknown/release/cw4626_escher.wasm $out/
            '';
          };
        };

        # Default package
        packages.default = self.packages.${system}.cw4626-escher;
      }
    );
}
