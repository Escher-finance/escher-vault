use astroport::asset::{Asset, AssetInfo};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Decimal, Uint128};
use cw20_base::msg::InstantiateMarketingInfo;

use crate::state::{AccessControlRole, PricesMap, RedemptionRequest, TowerConfig};

#[cw_serde]
pub enum ReceiveMsg {
    /// Mints shares to receiver by depositing exact amount of underlying tokens
    Deposit { receiver: Addr },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub managers: Vec<Addr>,
    pub oracles: Vec<Addr>,
    pub underlying_token: AssetInfo,
    pub share_name: String,
    pub share_symbol: String,
    pub share_marketing: Option<InstantiateMarketingInfo>,
    pub tower_incentives: Addr,
    pub lp: Addr,
    pub slippage_tolerance: Decimal,
    pub incentives: Vec<AssetInfo>,
    pub staking_contract: Option<Addr>,
    // Entry fee configuration (applied on deposit/mint)
    pub entry_fee_rate: Option<Decimal>, // e.g., 0.1 (10%); None => 0
    pub entry_fee_recipient: Addr,       // If None, defaults to fee_recipient
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// Access control - add user to role
    AddToRole {
        role: AccessControlRole,
        address: Addr,
    },
    /// Access control - remove user from role
    RemoveFromRole {
        role: AccessControlRole,
        address: Addr,
    },
    /// Oracle update prices
    OracleUpdatePrices { prices: PricesMap },
    /// Manager bond
    Bond {
        amount: Uint128,
        salt: String,
        slippage: Option<Decimal>,
    },
    /// Manager unbond
    Unbond { amount: Uint128 },
    /// Manager add liquidity
    AddLiquidity { underlying_token_amount: Uint128 },
    /// Manager remove liquidity
    RemoveLiquidity { lp_token_amount: Uint128 },
    /// Manager claim incentives
    ClaimIncentives {},
    /// Manager swap
    Swap {
        amount: Uint128,
        asset_info: AssetInfo,
    },

    //
    // CW4626
    //
    /// Mints shares to receiver by depositing exact amount of underlying tokens
    Deposit { assets: Uint128, receiver: Addr },
    /// Request redemption with proper multi-asset distribution
    RequestRedeem {
        shares: Uint128,
        receiver: Addr,
        owner: Addr,
    },
    /// Complete redemption by burning shares AND distributing assets in one transaction
    CompleteRedemption { redemption_id: u64, tx_hash: String },
    /// CW20 receive
    Receive(cw20::Cw20ReceiveMsg),

    //
    // CW20
    //
    /// Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// Burn is a base message to destroy tokens forever
    Burn { amount: Uint128 },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<cw20::Expiration>,
    },
    /// Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<cw20::Expiration>,
    },
    /// Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Destroys tokens forever
    BurnFrom { owner: String, amount: Uint128 },
    /// If authorized, updates marketing metadata.
    /// Setting None/null for any of these will leave it unchanged.
    /// Setting Some("") will clear this field on the contract storage
    UpdateMarketing {
        /// A URL pointing to the project behind this token.
        project: Option<String>,
        /// A longer description of the token and it's utility. Designed for tooltips or such
        description: Option<String>,
        /// The address (if any) who can update this data structure
        marketing: Option<String>,
    },
    /// If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the token
    UploadLogo(cw20::Logo),
}

#[cw_serde]
pub struct AccessControlRoleResponse {
    pub addresses: Vec<Addr>,
}

#[cw_serde]
pub struct OracleTokensListResponse {
    pub tokens: Vec<String>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub staking_contract: Addr,
    pub tower_config: TowerConfig,
}

#[cw_serde]
pub struct OraclePricesResponse {
    pub prices: PricesMap,
}

#[cw_serde]
pub struct ExchangeRateResponse {
    pub exchange_rate: Decimal,
}

#[cw_serde]
pub struct GitInfoResponse {
    pub git: String,
}

#[cw_serde]
pub struct LpPositionResponse {
    pub lp_token_amount: Uint128,
}

#[cw_serde]
pub struct PendingIncentivesResponse {
    pub incentives: Vec<Asset>,
}

#[cw_serde]
pub struct RedemptionRequestResponse {
    pub request: Option<RedemptionRequest>,
}

#[cw_serde]
pub struct UserRedemptionRequestsResponse {
    pub requests: Vec<RedemptionRequest>,
}

#[cw_serde]
pub struct PreviewRedeemMultiAssetResponse {
    pub expected_assets: Vec<Asset>,
    pub total_value_in_underlying: Uint128,
}

#[cw_serde]
pub struct RedemptionStatsResponse {
    pub total_redemptions: u64,
    pub pending_redemptions: u64,
    pub completed_redemptions: u64,
    pub total_shares_burned: Uint128,
    pub total_assets_distributed: Vec<Asset>,
    pub total_value_distributed: Uint128,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GitInfoResponse)]
    GitInfo {},
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AccessControlRoleResponse)]
    Role { kind: AccessControlRole },
    #[returns(OracleTokensListResponse)]
    OracleTokensList {},
    /// NOTE: We might want to keep prices "private"
    #[returns(OracleTokensListResponse)]
    OraclePrices {},
    /// Returns vault underlying asset exchange rate (total_assets / total_shares) as string
    #[returns(ExchangeRateResponse)]
    ExchangeRate {},
    #[returns(LpPositionResponse)]
    LpPosition {},
    #[returns(PendingIncentivesResponse)]
    AllPendingIncentives {},

    //
    // Redemption System
    //
    /// Get redemption request details
    #[returns(RedemptionRequestResponse)]
    RedemptionRequest { id: u64 },
    /// Get all redemption requests for a user
    #[returns(UserRedemptionRequestsResponse)]
    UserRedemptionRequests { user: Addr },
    /// Preview redemption with multi-asset distribution
    #[returns(PreviewRedeemMultiAssetResponse)]
    PreviewRedeemMultiAsset { shares: Uint128 },
    /// Get redemption statistics and summary
    #[returns(RedemptionStatsResponse)]
    RedemptionStats,

    //
    // CW4626
    //
    /// Returns the address of the underlying cw20 token used for the Vault for accounting, depositing, and withdrawing
    #[returns(AssetResponse)]
    Asset {},
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
    /// Returns the maximum amount of Vault shares that can be redeemed from the owner balance in the Vault,
    /// through a redeem call
    #[returns(MaxRedeemResponse)]
    MaxRedeem { owner: Addr },
    /// Allows an on-chain or off-chain user to simulate the effects of their redemption at the current block,
    /// given current on-chain conditions
    #[returns(PreviewRedeemResponse)]
    PreviewRedeem { shares: Uint128 },

    //
    // CW20
    //
    /// Returns the current balance of the given address, 0 if unset.
    #[returns(cw20::BalanceResponse)]
    Balance { address: String },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    #[returns(cw20::TokenInfoResponse)]
    TokenInfo {},
    /// Returns how much spender can use from owner account, 0 if unset.
    #[returns(cw20::AllowanceResponse)]
    Allowance { owner: String, spender: String },
    /// Returns all allowances this owner has approved. Supports pagination.
    #[returns(cw20::AllAllowancesResponse)]
    AllAllowances {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns all allowances this spender has been granted. Supports pagination.
    #[returns(cw20::AllSpenderAllowancesResponse)]
    AllSpenderAllowances {
        spender: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns all accounts that have balances. Supports pagination.
    #[returns(cw20::AllAccountsResponse)]
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns more metadata on the contract to display in the client:
    /// - description, logo, project url, etc.
    #[returns(cw20::MarketingInfoResponse)]
    MarketingInfo {},
    /// Downloads the embedded logo data (if stored on chain). Errors if no logo data is stored for this
    /// contract.
    #[returns(cw20::DownloadLogoResponse)]
    DownloadLogo {},
}

#[cw_serde]
pub struct AssetResponse {
    pub asset_token_address: String,
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
pub struct MaxRedeemResponse {
    pub max_shares: Uint128,
}

#[cw_serde]
pub struct PreviewRedeemResponse {
    pub assets: Uint128,
}
