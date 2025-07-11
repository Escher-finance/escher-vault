# cw-vault

| Specification | Path                                                                                        |
| ------------- | ------------------------------------------------------------------------------------------- |
| cw4626        | [`./packages/cw4626`](https://github.com/Escher-finance/cw-vault/tree/main/packages/cw4626) |

| Contracts     | Path                                                                                                    |
| ------------- | ------------------------------------------------------------------------------------------------------- |
| cw4626-base   | [`./contracts/cw4626-base`](https://github.com/Escher-finance/cw-vault/tree/main/contracts/cw4626-base) |
| cw4626-escher | `TODO`                                                                                                  |

<!-- | cw4626-escher | [`./contracts/cw4626-escher`](https://github.com/Escher-finance/cw-vault/tree/main/contracts/cw4626-escher) | -->

## Build

```bash
cargo wasm
```

## Test

```bash
cargo test
```

## Test Coverage

Requires `cargo-llvm-cov` (`cargo install cargo-llvm-cov`)

CLI output:

```bash
cargo cov
```

Web output:

```bash
cargo cov --open
```

## Generate JSON schema

```bash
cargo schema
```
