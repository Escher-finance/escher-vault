# CW4626 Escher

Opinionated `CW4626` implementation adapted from Ethereum's
[ERC4626](https://eips.ethereum.org/EIPS/eip-4626) and its
[OpenZeppelin implementation](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/extensions/ERC4626.sol).

- Just like on Ethereum vault contract _is_ also the share token (fully
  implements `CW20`, via `cw20_base`)
- All other assets in the contract (including the underlying asset) can be
  either `CW20` or native (using `astroport::Asset` and `astroport::AssetInfo`)

<img width="2850" height="1789" alt="cw4626-escher-diagram" src="https://github.com/user-attachments/assets/c482689d-f51e-4049-81ce-32c4b64a7214" />
