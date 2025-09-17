{
  description = "CW4626 Vault - LP Staking on Astroport";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        lib = pkgs.lib;

        # Crane library for offline Rust builds with vendoring
        craneLib = (crane.mkLib pkgs);

        astroportSrc = pkgs.fetchFromGitHub {
          owner = "quasar-finance";
          repo = "babydex";
          rev = "8fce1b955a1769a1f4286c73cbfd36701753ac1e";
          sha256 = "sha256-2MkxcBG9rd3B8aivY4bXdByd+fnuqJ8zuwVIk+RdHZU=";
        };

        # Use the main Union branch
        unionSrc = pkgs.fetchFromGitHub {
          owner = "unionlabs";
          repo = "union";
          rev = "8768bb1f3a7e4c73901fdcd356789c4fb29b051f";
          sha256 = "sha256-kkVQlO4zqOhrWCtpbpqCe/SMu2EBqjfByd6uUm8DSrY=";
        };

        # Create individual source packages for Union crates to avoid dependency conflicts
        unionlabsPrimitivesSrc = unionSrc + "/lib/unionlabs-primitives";
        ucs03ZkgmSrc = unionSrc + "/cosmwasm/ibc-union/app/ucs03-zkgm";
        ibcUnionSpecSrc = unionSrc + "/lib/ibc-union-spec";

        # Rust toolchain with latest nightly that supports Rust 2024 features
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

            # Set up Cargo configuration with patches
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

            echo "🔧 Cargo patches applied for development"
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
          # Build the WASM contract using crane with vendored dependencies (offline)
          cw4626-escher = let
            # Clean source for reproducibility
            src = craneLib.cleanCargoSource ./.;
            # Vendor all Cargo dependencies; set a fake hash first, build once to get the correct hash
            cargoVendorDir = craneLib.vendorCargoDeps {
              inherit src;
              cargoLock = ./Cargo.lock;
              cargoVendorHash = "sha256-65ouUH7OfQF2r2XOFugM9KxLHTitrSxnt57ghS8qruk=";
            };

            commonArgs = {
              pname = "cw4626-escher";
              version = "0.1.0";
              inherit src cargoVendorDir;
              nativeBuildInputs = [
                rustToolchain
                pkgs.binaryen
                pkgs.pkg-config
                pkgs.lld_18
                pkgs.cacert
              ];
              CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
              RUSTFLAGS = "-C target-feature=-reference-types,-bulk-memory";
            };

            # Build dependencies only (avoids re-building them on every change)
            deps = craneLib.buildDepsOnly (commonArgs // {
              cargoExtraArgs = "-p cw4626-escher --lib --target wasm32-unknown-unknown --offline";
              doCheck = false;
            });
          in craneLib.buildPackage (commonArgs // {
            pname = "cw4626-escher";
            version = "0.1.0";
            inherit src cargoVendorDir;
            cargoArtifacts = deps;

            # Use our custom toolchain
            nativeBuildInputs = [
              rustToolchain
              pkgs.binaryen
              pkgs.pkg-config
              pkgs.lld_18
              pkgs.cacert
            ];

            # Ensure we build the desired crate for wasm target
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            RUSTFLAGS = "-C target-feature=-reference-types,-bulk-memory";
            cargoExtraArgs = "-p cw4626-escher --lib --target wasm32-unknown-unknown --offline";

            # Patch git dependencies before building
            preBuild = ''
              export CARGO_HOME=$(pwd)/.cargo-home
              mkdir -p $CARGO_HOME

              # Append our patches without overwriting vendored source config
              cat >> $CARGO_HOME/config.toml <<'CFG'
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

              # Also apply crates-io patches for workspace resolution
              cat >> Cargo.toml <<'PATCH'

              [patch.crates-io]
              unionlabs-primitives = { path = "${unionlabsPrimitivesSrc}" }
              ucs03-zkgm = { path = "${ucs03ZkgmSrc}" }
              ibc-union-spec = { path = "${ibcUnionSpecSrc}" }
              PATCH
            '';

            doCheck = false;

            installPhase = ''
              set -euo pipefail
              mkdir -p "$out"
              # Prefer the direct build artifact
              in_wasm="target/wasm32-unknown-unknown/release/cw4626_escher.wasm"
              if [ ! -f "''${in_wasm}" ]; then
                # Fallback: search for any wasm in target tree
                in_wasm=$(find target -type f -name "*.wasm" | head -n1 || true)
              fi
              if [ -n "''${in_wasm}" ] && [ -f "''${in_wasm}" ]; then
                wasm-opt -Oz --signext-lowering --strip-debug --strip-producers \
                  "''${in_wasm}" -o "$out/cw4626_escher.wasm"
              else
                echo "Error: WASM artifact not found under target" >&2
                exit 1
              fi
            '';
          });
        };

        # Default package
        packages.default = self.packages.${system}.cw4626-escher;
      }
    );
}

