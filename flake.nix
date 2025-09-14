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
        lib = pkgs.lib;

        astroportSrc = pkgs.fetchFromGitHub {
          owner = "quasar-finance";
          repo = "babydex";
          rev = "8fce1b955a1769a1f4286c73cbfd36701753ac1e";
          sha256 = "sha256-2MkxcBG9rd3B8aivY4bXdByd+fnuqJ8zuwVIk+RdHZU=";
        };

        # Use the fixed Union branch that removes static_assertions conflicts
        unionSrc = pkgs.fetchFromGitHub {
          owner = "aso20455";
          repo = "union";
          rev = "fix/deleted";
          sha256 = "sha256-2KV6nGl/ApSq/DL8ow+FX92fQfvMzqGjkisPs8Fbx3o=";
        };

        # Create individual source packages for Union crates to avoid dependency conflicts
        unionlabsPrimitivesSrc = unionSrc + "/lib/unionlabs-primitives";
        ucs03ZkgmSrc = unionSrc + "/cosmwasm/ibc-union/app/ucs03-zkgm";
        ibcUnionSpecSrc = unionSrc + "/lib/ibc-union-spec";




        # Rust toolchain with specific version
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
           "rust-src"
            "rust-analyzer"
            "clippy"
            "llvm-tools-preview"
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
          # Build the WASM contract using stdenv.mkDerivation to avoid vendoring issues
          cw4626-escher = pkgs.stdenv.mkDerivation {
            pname = "cw4626-escher";
            version = "0.1.0";
            src = ./.;

            # Use our custom toolchain
            rustc = rustToolchain;
            cargo = rustToolchain;

            nativeBuildInputs = [
              rustToolchain
              pkgs.binaryen
              pkgs.pkg-config
              pkgs.lld_18
              pkgs.cacert
            ];

            # Environment and build configuration
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            RUSTFLAGS = "-C target-feature=-reference-types,-bulk-memory";

            # Patch git dependencies before building
            prePatch = ''
              export CARGO_HOME=$(pwd)/.cargo-home
              mkdir -p $CARGO_HOME
              
              cat > $CARGO_HOME/config.toml <<'CFG'
              [patch.'https://github.com/quasar-finance/babydex.git']
              astroport = { path = "${astroportSrc}/packages/astroport" }
              astroport-factory = { path = "${astroportSrc}/contracts/factory" }
              astroport-pair = { path = "${astroportSrc}/contracts/pair" }
              astroport-pair-concentrated = { path = "${astroportSrc}/contracts/pair_concentrated" }
              astroport-pcl-common = { path = "${astroportSrc}/packages/astroport_pcl_common" }

              [patch.'https://github.com/unionlabs/union']
              unionlabs-primitives = { path = "${unionlabsPrimitivesSrc}" }
              ucs03-zkgm = { path = "${ucs03ZkgmSrc}" }
              ibc-union-spec = { path = "${ibcUnionSpecSrc}" }
              CFG
              
              # Also create a Cargo.toml patch in the workspace root
              cat > Cargo.toml.patch <<'PATCH'
              [patch.crates-io]
              unionlabs-primitives = { path = "${unionlabsPrimitivesSrc}" }
              ucs03-zkgm = { path = "${ucs03ZkgmSrc}" }
              ibc-union-spec = { path = "${ibcUnionSpecSrc}" }
              PATCH
            '';

            # Build only the library for the specific package
            buildPhase = ''
              runHook preBuild
              
              # Apply patches to Cargo.toml
              cat >> Cargo.toml <<'PATCH'
              
              [patch.crates-io]
              unionlabs-primitives = { path = "${unionlabsPrimitivesSrc}" }
              ucs03-zkgm = { path = "${ucs03ZkgmSrc}" }
              ibc-union-spec = { path = "${ibcUnionSpecSrc}" }
              PATCH
              
              cargo build --release --lib --target wasm32-unknown-unknown -p cw4626-escher
              runHook postBuild
            '';

            # Skip tests
            doCheck = false;

            # Optimize the WASM output
            postBuild = ''
              mkdir -p artifacts
              if [ -f target/wasm32-unknown-unknown/release/cw4626_escher.wasm ]; then
                wasm-opt -Oz --signext-lowering --strip-debug --strip-producers \
                  target/wasm32-unknown-unknown/release/cw4626_escher.wasm \
                  -o artifacts/cw4626_escher.wasm
              else
                echo "Warning: WASM file not found, looking for alternatives..."
                find target -name "*.wasm" -type f
              fi
            '';

            installPhase = ''
              mkdir -p $out
              if [ -f artifacts/cw4626_escher.wasm ]; then
                cp artifacts/cw4626_escher.wasm $out/
              else
                echo "Error: Optimized WASM file not found"
                exit 1
              fi
            '';
          };
        };

        # Default package
        packages.default = self.packages.${system}.cw4626-escher;
      }
    );
}