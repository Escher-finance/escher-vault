use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct Cw4626InstantiateMsg {
    pub owner: Option<Addr>,
    pub share_token_address: Addr,
    pub underlying_token_address: Addr,
}

#[cw_serde]
pub enum Cw4626ExecuteMsg {
    /// Mints shares to receiver by depositing exact amount of underlying tokens
    Deposit { assets: Uint128, receiver: Addr },
    /// Mints exact shares to receiver by depositing amount of underlying tokens
    Mint { shares: Uint128, receiver: Addr },
    /// Burns shares from owner and sends exact assets of underlying tokens to receiver
    Withdraw {
        assets: Uint128,
        receiver: Addr,
        owner: Addr,
    },
    /// Burns exact shares from owner and sends assets of underlying tokens to receiver
    Redeem {
        shares: Uint128,
        receiver: Addr,
        owner: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum Cw4626QueryMsg {
    /// Since the share token is separated from the vault, added a getter to the share
    /// Returns the address of the share cw20 token used for the Vault for accounting, depositing, and withdrawing
    #[returns(ShareResponse)]
    Share {},
    /// Returns the address of the underlying cw20 token used for the Vault for accounting, depositing, and withdrawing
    #[returns(AssetResponse)]
    Asset {},
    /// Since the share token is separated from the vault, added a getter to the total shares
    /// Returns the total amount of the share asset that is managed by Vault
    #[returns(TotalSharesResponse)]
    TotalShares {},
    /// Returns the total amount of the underlying asset that is managed by Vault
    #[returns(TotalAssetsResponse)]
    TotalAssets {},
    /// Returns the amount of shares that the Vault would exchange for the amount of assets provided, in an ideal
    /// scenario where all the conditions are met
    #[returns(ConvertToSharesResponse)]
    ConvertToShares { assets: Uint128 },
    /// Returns the amount of assets that the Vault would exchange for the amount of shares provided, in an ideal
    /// scenario where all the conditions are met
    #[returns(ConvertToAssetsResponse)]
    ConvertToAssets { shares: Uint128 },
    /// Returns the maximum amount of the underlying asset that can be deposited into the Vault for the receiver,
    /// through a deposit call
    #[returns(MaxDepositResponse)]
    MaxDeposit { receiver: Addr },
    /// Allows an on-chain or off-chain user to simulate the effects of their deposit at the current block, given
    /// current on-chain conditions
    #[returns(PreviewDepositResponse)]
    PreviewDeposit { assets: Uint128 },
    /// Returns the maximum amount of the Vault shares that can be minted for the receiver, through a mint call
    #[returns(MaxMintResponse)]
    MaxMint { receiver: Addr },
    /// Allows an on-chain or off-chain user to simulate the effects of their mint at the current block, given
    /// current on-chain conditions
    #[returns(PreviewMintResponse)]
    PreviewMint { shares: Uint128 },
    /// Returns the maximum amount of the underlying asset that can be withdrawn from the owner balance in the
    /// Vault, through a withdraw call
    #[returns(MaxWithdrawResponse)]
    MaxWithdraw { owner: Addr },
    /// Allows an on-chain or off-chain user to simulate the effects of their withdrawal at the current block,
    /// given current on-chain conditions
    #[returns(PreviewWithdrawResponse)]
    PreviewWithdraw { assets: Uint128 },
    /// Returns the maximum amount of Vault shares that can be redeemed from the owner balance in the Vault,
    /// through a redeem call
    #[returns(MaxRedeemResponse)]
    MaxRedeem { owner: Addr },
    /// Allows an on-chain or off-chain user to simulate the effects of their redemption at the current block,
    /// given current on-chain conditions
    #[returns(PreviewRedeemResponse)]
    PreviewRedeem { shares: Uint128 },
}

#[cw_serde]
pub struct ShareResponse {
    pub share_token_address: Addr,
}

#[cw_serde]
pub struct AssetResponse {
    pub asset_token_address: Addr,
}

#[cw_serde]
pub struct TotalSharesResponse {
    pub total_managed_shares: Uint128,
}

#[cw_serde]
pub struct TotalAssetsResponse {
    pub total_managed_assets: Uint128,
}

#[cw_serde]
pub struct ConvertToSharesResponse {
    pub shares: Uint128,
}

#[cw_serde]
pub struct ConvertToAssetsResponse {
    pub assets: Uint128,
}

#[cw_serde]
pub struct MaxDepositResponse {
    pub max_assets: Uint128,
}

#[cw_serde]
pub struct PreviewDepositResponse {
    pub shares: Uint128,
}

#[cw_serde]
pub struct MaxMintResponse {
    pub max_shares: Uint128,
}

#[cw_serde]
pub struct PreviewMintResponse {
    pub assets: Uint128,
}

#[cw_serde]
pub struct MaxWithdrawResponse {
    pub max_assets: Uint128,
}

#[cw_serde]
pub struct PreviewWithdrawResponse {
    pub shares: Uint128,
}

#[cw_serde]
pub struct MaxRedeemResponse {
    max_shares: Uint128,
}

#[cw_serde]
pub struct PreviewRedeemResponse {
    assets: Uint128,
}
