# CW4626 Escher

Opinionated `CW4626` implementation adapted from Ethereum's
[ERC4626](https://eips.ethereum.org/EIPS/eip-4626) and its
[OpenZeppelin implementation](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/extensions/ERC4626.sol).

- Just like on Ethereum, the vault contract _is_ also the share token (fully
  implements `CW20`, via `cw20_base`)
- All other assets in the contract (including the underlying asset) can be
  either `CW20` or native (using `astroport::Asset` and `astroport::AssetInfo`)
- It integrates with:
  - A concentrated liquidity pool on TowerFi (Astroport)
  - An LST contract
    - `NonZkgm`: In the same network (Babylon vault with Babylon's Escher Hub)
    - `Zkgm`: In a different network (Babylon vault with Union's LST)

<img width="2850" height="1789" alt="cw4626-escher-diagram" src="https://github.com/user-attachments/assets/c482689d-f51e-4049-81ce-32c4b64a7214" />

## Spec

### Instantiate

`InstantiateMsg` initializes the vault with the following fields:

- `managers` - Addresses granted the `Manager` role
- `oracles` - Addresses granted the `Oracle` role
- `underlying_token` - The asset (CW20 or native) used for deposits
- `share_name` - Name of the share token (CW20)
- `share_symbol` - Symbol of the share token (CW20)
- `share_marketing` - Optional CW20 marketing metadata
- `tower_incentives` - TowerFi incentives contract address
- `lp` - TowerFi concentrated liquidity pool contract address
- `slippage_tolerance` - Slippage tolerance for LP operations
- `incentives` - List of LP incentive token assets
- `lst_config` - Optional LST configuration (`NonZkgm` for same-chain, `Zkgm`
  for cross-chain)
- `minimum_deposit` - Optional minimum deposit amount (in underlying token
  units)
- `entry_fee_rate` - Optional entry fee applied on deposit (e.g., `0.1` = 10%);
  defaults to 0
- `entry_fee_recipient` - Address that receives the entry fee shares

## Access Control

Two roles exist: `Manager` and `Oracle`. Managers administer the vault and LP
operations. Oracles submit asset prices.

### Messages

`AddToRole { role, address }` - Callable only by a Manager. Grants `address` the
given `role`.

Attributes emitted:

| Key       | Value   |
| --------- | ------- |
| "sender"  | caller  |
| "role"    | role    |
| "address" | address |

`RemoveFromRole { role, address }` - Callable only by a Manager. Revokes `role`
from `address`.

Attributes emitted:

| Key       | Value   |
| --------- | ------- |
| "sender"  | caller  |
| "role"    | role    |
| "address" | address |

### Queries

`Role { kind }` - Returns all addresses currently holding the given role. Return
type is `AccessControlRoleResponse { addresses }`.

## Oracle

### Messages

`OracleUpdatePrices { prices }` - Callable only by an Oracle. Submits a map of
asset denom/address → price (`Decimal`) used to value vault assets for
multi-asset redemption calculations.

Attributes emitted:

| Key      | Value                               |
| -------- | ----------------------------------- |
| "sender" | caller                              |
| "prices" | comma-separated `denom=price` pairs |

### Queries

`OracleTokensList {}` - Returns the list of token denoms/addresses that have a
recorded price. Return type is `OracleTokensListResponse { tokens }`.

`OraclePrices {}` - Returns the full price map. Return type is
`OraclePricesResponse { prices }`.

## Configuration

### Messages

`UpdateLstConfig { config }` - Callable only by a Manager. Replaces the active
LST configuration with a new `NonZkgm` or `Zkgm` variant.

`UpdateMinimumDeposit { amount }` - Callable only by a Manager. Sets the minimum
amount of underlying tokens required per deposit.

`TogglePausedStatus {}` - Callable only by a Manager. Cycles the vault pause
status between `NotPaused`, `PausedMaintenance`, and `PausedOngoingBonding`.
Most operations are blocked while paused.

### Queries

`Config {}` - Returns current LST and tower (LP) configuration. Return type is
`ConfigResponse { lst_config, tower_config }`.

`Paused {}` - Returns the current pause status. Return type is
`PausedResponse { status }`.

## CW4626 Core

The vault contract is also the share token. Depositing underlying tokens mints
shares; redeeming shares distributes vault assets back to the user. Redemption
is a two-phase process: the user requests redemption (locking shares), then a
Manager completes it (burning shares and distributing assets).

### Messages

`Deposit { assets, receiver }` - Transfers exactly `assets` units of the
underlying token from `info.sender` to the vault and mints the corresponding
shares to `receiver`. If an entry fee is configured, a portion of the minted
shares goes to the fee recipient instead.

Attributes emitted (no fee):

| Key                  | Value         |
| -------------------- | ------------- |
| "depositor"          | sender        |
| "receiver"           | receiver      |
| "assets_transferred" | assets amount |
| "shares_minted"      | shares minted |

Attributes emitted (with entry fee):

| Key                  | Value                     |
| -------------------- | ------------------------- |
| "depositor"          | sender                    |
| "receiver"           | receiver                  |
| "assets_transferred" | assets amount             |
| "user_shares_minted" | shares minted to receiver |
| "fee_shares_minted"  | shares minted as fee      |
| "entry_fee_rate"     | fee rate applied          |

`Receive(Cw20ReceiveMsg)` - CW20 callback entrypoint. When the vault receives
CW20 tokens via a `Send`, the inner `msg` must decode to
`ReceiveMsg::Deposit { receiver }`, which performs the same deposit logic as the
`Deposit` message above.

`RequestRedeem { shares, receiver, owner }` - Locks `shares` from `owner`'s
balance and creates a pending redemption request. The expected multi-asset
distribution is calculated at this point based on current oracle prices. The
vault is _not_ paused required for `owner` to equal `info.sender` (standard
path) or for a pre-approved allowance.

Attributes emitted:

| Key                     | Value                                   |
| ----------------------- | --------------------------------------- |
| "redemption_id"         | unique redemption request ID            |
| "owner"                 | share owner                             |
| "receiver"              | asset recipient                         |
| "shares_locked"         | shares locked                           |
| "expected_assets_count" | number of asset types to be distributed |
| "expected_assets"       | comma-separated `asset=amount` pairs    |
| "created_at"            | request creation timestamp              |
| "total_expected_value"  | sum of expected asset amounts           |

`CompleteRedemption { redemption_id, tx_hash }` - Callable only by a Manager.
Burns the locked shares for the given `redemption_id` and distributes the vault
assets to the original `receiver`. The `tx_hash` records the cross-chain
transaction that triggered completion.

Attributes emitted:

| Key                        | Value                                |
| -------------------------- | ------------------------------------ |
| "redemption_id"            | redemption request ID                |
| "receiver"                 | asset recipient                      |
| "shares_burned"            | shares burned                        |
| "completed_at"             | completion timestamp                 |
| "tx_hash"                  | completion transaction hash          |
| "distributed_assets"       | comma-separated `asset=amount` pairs |
| "distributed_assets_count" | number of asset types distributed    |

### Queries

`Asset {}` - Returns the address/denom of the underlying asset. Return type is
`AssetResponse { asset_token_address }`.

`TotalAssets {}` - Returns the total amount of underlying tokens managed by the
vault (including LP and staked portions). Return type is
`TotalAssetsResponse { total_managed_assets }`.

`ConvertToShares { assets }` - Returns the number of shares that would be minted
for `assets` underlying tokens at the current exchange rate. Return type is
`ConvertToSharesResponse { shares }`.

`ConvertToAssets { shares }` - Returns the number of underlying tokens that
`shares` would convert to at the current exchange rate. Return type is
`ConvertToAssetsResponse { assets }`.

`MaxDeposit { receiver }` - Returns the maximum underlying token amount the
vault currently accepts from `receiver`. Return type is
`MaxDepositResponse { max_assets }`.

`PreviewDeposit { assets }` - Simulates a deposit of `assets` and returns the
shares that would be minted. Return type is `PreviewDepositResponse { shares }`.

`MaxRedeem { owner }` - Returns the maximum share amount `owner` can submit for
redemption. Return type is `MaxRedeemResponse { max_shares }`.

`PreviewRedeem { shares }` - Simulates redeeming `shares` and returns the
underlying token amount that would be received. Return type is
`PreviewRedeemResponse { assets }`.

`ExchangeRate {}` - Returns the current `total_assets / total_shares` ratio.
Return type is `ExchangeRateResponse { exchange_rate }`.

## Redemption System

### Queries

`RedemptionRequest { id }` - Returns the details of a single redemption request
by its ID. Return type is `RedemptionRequestResponse { request }`.

`UserRedemptionRequests { user }` - Returns all redemption requests (pending and
completed) for a given user. Return type is
`UserRedemptionRequestsResponse { requests }`.

`PreviewRedeemMultiAsset { shares }` - Returns the expected multi-asset
distribution for redeeming `shares` at current prices. Return type is
`PreviewRedeemMultiAssetResponse { expected_assets, total_value_in_underlying }`.

`RedemptionStats` - Returns aggregate redemption statistics. Return type is
`RedemptionStatsResponse { total_redemptions, pending_redemptions, completed_redemptions, total_shares_burned, total_assets_distributed, total_value_distributed }`.

`AllRedemptionRequests { start_after, limit }` - Lists all redemption requests
with pagination. Return type is
`AllRedemptionRequestsResponse { requests, next_start_after }`.

## LP Management

Managers control the vault's liquidity position on TowerFi and can bond
underlying tokens to the configured LST contract.

### Messages

`Bond(ExecuteBondPayload)` - Callable only by a Manager (vault must not be
paused). Bonds underlying tokens to the LST contract. Two variants:

- `NonZkgm { amount, salt, slippage }` — same-chain LST bond
- `Zkgm { amount, salt, min_mint_amount }` — cross-chain LST bond via IBC/ZKGM

Attributes emitted:

| Key                | Value                    |
| ------------------ | ------------------------ |
| "sender"           | caller                   |
| "amount"           | amount bonded            |
| "expected"         | expected LST mint amount |
| "staking_contract" | LST contract address     |

`AddLiquidity { underlying_token_amount }` - Callable only by a Manager. Adds
`underlying_token_amount` of the underlying token (plus a proportional amount of
the paired asset) as liquidity to the TowerFi LP pool.

Attributes emitted:

| Key                       | Value                        |
| ------------------------- | ---------------------------- |
| "sender"                  | caller                       |
| "underlying_token_amount" | underlying tokens provided   |
| "other_lp_token_amount"   | paired asset tokens provided |
| "lp_contract"             | LP pool contract address     |

`RemoveLiquidity { lp_token_amount }` - Callable only by a Manager. Removes
`lp_token_amount` LP tokens from the TowerFi pool, receiving the underlying
assets back.

Attributes emitted:

| Key               | Value                    |
| ----------------- | ------------------------ |
| "sender"          | caller                   |
| "lp_token_amount" | LP tokens withdrawn      |
| "lp_contract"     | LP pool contract address |

`ClaimIncentives {}` - Callable only by a Manager. Claims all pending LP
incentive rewards from TowerFi.

Attributes emitted:

| Key           | Value                    |
| ------------- | ------------------------ |
| "sender"      | caller                   |
| "lp_contract" | LP pool contract address |

`Swap { amount, asset_info }` - Callable only by a Manager. Swaps `amount` of
`asset_info` through the TowerFi LP pool.

Attributes emitted:

| Key          | Value               |
| ------------ | ------------------- |
| "sender"     | caller              |
| "amount"     | amount swapped      |
| "asset_info" | asset being swapped |

### Queries

`LpPosition {}` - Returns the vault's current LP token balance in the TowerFi
pool. Return type is `LpPositionResponse { lp_token_amount }`.

`AllPendingIncentives {}` - Returns all unclaimed LP incentive rewards. Return
type is `PendingIncentivesResponse { incentives }`.

## CW20

The vault share token fully implements the CW20 standard (base, allowances,
enumerable, marketing) via `cw20_base`. For the full CW20 spec see the
[CW20 specification](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw20/README.md).

### Messages

`Transfer { recipient, amount }` - Moves `amount` shares from `info.sender` to
`recipient`.

`Burn { amount }` - Destroys `amount` shares from `info.sender`'s balance.

`Send { contract, amount, msg }` - Transfers `amount` shares to `contract` and
invokes the CW20 `Receive` hook on it with `msg`.

`IncreaseAllowance { spender, amount, expires }` - Increases the allowance for
`spender` to spend `info.sender`'s shares.

`DecreaseAllowance { spender, amount, expires }` - Decreases the allowance for
`spender`. Rounds down to 0 if `amount` exceeds the current allowance.

`TransferFrom { owner, recipient, amount }` - Transfers `amount` shares from
`owner` to `recipient` using a pre-approved allowance.

`SendFrom { owner, contract, amount, msg }` - Like `TransferFrom`, but sends to
a contract and triggers its `Receive` hook.

`BurnFrom { owner, amount }` - Burns `amount` shares from `owner` using a
pre-approved allowance.

`UpdateMarketing { project, description, marketing }` - Updates marketing
metadata. Callable by the marketing role address.

`UploadLogo(Logo)` - Uploads a new logo (URL or embedded SVG/PNG). Callable by
the marketing role address.

### Queries

`Balance { address }` - Returns the share balance of `address`. Return type is
`BalanceResponse { balance }`.

`TokenInfo {}` - Returns share token metadata. Return type is
`TokenInfoResponse { name, symbol, decimals, total_supply }`.

`Allowance { owner, spender }` - Returns the current allowance and expiration.
Return type is `AllowanceResponse { allowance, expires }`.

`AllAllowances { owner, start_after, limit }` - Lists all allowances granted by
`owner`. Supports pagination.

`AllSpenderAllowances { spender, start_after, limit }` - Lists all allowances
granted to `spender`. Supports pagination.

`AllAccounts { start_after, limit }` - Lists all accounts with a share balance.
Supports pagination.

`MarketingInfo {}` - Returns marketing metadata. Return type is
`MarketingInfoResponse { project, description, logo, marketing }`.

`DownloadLogo {}` - Returns embedded logo data. Return type is
`DownloadLogoResponse { mime_type, data }`.

## Misc

### Queries

`GitInfo {}` - Returns the git branch and commit SHA the contract was built
from. Return type is `GitInfoResponse { git }`.
