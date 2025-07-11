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

## Attributes emitted

`Cw4626ExecuteMsg:Deposit`, `Cw4626ExecuteMsg:Mint` and
`Cw4626ReceiveMsg:Deposit` emit:

|         Key          |       Value        |
| :------------------: | :----------------: |
|       `action`       |     `deposit`      |
|     `depositor`      | `{depositor addr}` |
|      `receiver`      | `{receiver addr}`  |
| `assets_transferred` |     `{assets}`     |
|   `shares_minted`    |     `{shares}`     |

`Cw4626ExecuteMsg:Withdraw` and `Cw4626ExecuteMsg:Redeem` emit :

|        Key        |        Value        |
| :---------------: | :-----------------: |
|     `action`      |     `withdraw`      |
|   `withdrawer`    | `{withdrawer addr}` |
|    `receiver`     |  `{receiver addr}`  |
| `assets_received` |     `{assets}`      |
|  `shares_burned`  |     `{shares}`      |

## Execute Messages `Cw4626ExecuteMsg`

`Deposit { assets: Uint128, receiver: Addr }` - Mints shares to receiver by
depositing exact amount of underlying tokens

`Mint { shares: Uint128, receiver: Addr }` - Mints exact shares to receiver by
depositing amount of underlying tokens

`Withdraw { assets: Uint128, receiver: Addr, owner: Addr }` - Burns shares from
owner and sends exact assets of underlying tokens to receiver

`Redeem { shares: Uint128, receiver: Addr, owner: Addr }` - Burns exact shares
from owner and sends assets of underlying tokens to receiver

`Receive(Cw20ReceiveMsg)` - CW20 receive

## Receive Messages `Cw4626ReceiveMsg`

`Deposit { receiver: Addr }` - Mints shares to receiver by depositing exact
amount of underlying tokens
