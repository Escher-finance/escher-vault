use astroport::asset::AssetInfo;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Decimal, Uint128};

use cw4626::*;

use crate::state::{AccessControlRole, PricesMap};

#[cw_serde]
pub struct InstantiateMsg {
    pub manager: Addr,
    pub oracle: Addr,
    pub underlying_token: AssetInfo,
    pub share_name: String,
    pub share_symbol: String,
    pub share_marketing: Option<InstantiateMarketingInfo>,
    pub tower_incentives: Addr,
    pub lp: Addr,
    pub slippage_tolerance: Decimal,
    pub incentives: Vec<AssetInfo>,
    pub staking_contract: Option<Addr>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Access control: sets role
    UpdateRole {
        role: AccessControlRole,
        address: Addr,
    },

    /// Update prices
    OracleUpdatePrices { prices: PricesMap },

    //
    // CW4626
    //
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
    pub address: Addr,
}

#[cw_serde]
pub struct OracleTokensListResponse {
    pub tokens: Vec<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AccessControlRoleResponse)]
    Role { kind: AccessControlRole },

    #[returns(OracleTokensListResponse)]
    OracleTokensList {},

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
