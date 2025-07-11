# CW4626 Spec: Tokenized Vaults

Opinionated spec adapted from Ethereum's
[ERC4626](https://eips.ethereum.org/EIPS/eip-4626) and its
[OpenZeppelin implementation](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/extensions/ERC4626.sol).

Basic implementation can be found in
[cw4626-base](https://github.com/Escher-finance/cw-vault/tree/main/contracts/cw4626-base).

- This standard provides an API for ownable vaults with an underlying `CW20`
  (the `ASSET` token)
- Itself fully implements `CW20` (the `SHARE` token)
- It implements extra functionality to make it feel more native to Cosmos: just
  like `CW20` added `Send` in order to avoid the `IncreaseAllowance` +
  `TransferFrom` combo, `CW4626` does something similar with `Deposit`, allowing
  the user to deposit assets in a single transaction via `Send` using
  `Cw4626ReceiveMsg:Deposit`
