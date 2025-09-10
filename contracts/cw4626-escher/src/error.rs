use astroport::asset::AssetInfo;
use cosmwasm_std::{Addr, StdError, Uint128};
use cw20_base::ContractError as Cw20ContractError;
use cw_utils::PaymentError;
use thiserror::Error;

use crate::state::AccessControlRole;

pub type ContractResult<T> = Result<T, ContractError>;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ShareCw20Error(#[from] Cw20ContractError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("only {0} role")]
    Unauthorized(AccessControlRole),

    #[error("tower config is not valid")]
    InvalidTowerConfig {},

    #[error("oracle prices must be greater than zero")]
    OracleZeroPrice {},

    #[error("oracle prices are not valid")]
    OracleInvalidPrices {},

    #[error("insufficient funds for operation")]
    InsufficientFunds {},

    #[error("wrong fund amount provided")]
    WrongFundAmountProvided {},

    #[error("{addr} is not a cw20")]
    InvalidCw20 { addr: Addr },

    #[error("{addr} cw20 is not supported for this receive msg")]
    UnsupportedCw20Received { addr: Addr },

    #[error("{receiver} deposit of {assets} assets exceeds the max {max_assets}")]
    ExceededMaxDeposit {
        receiver: Addr,
        assets: Uint128,
        max_assets: Uint128,
    },

    #[error("{receiver} mint of {shares} shares exceeds the max {max_shares}")]
    ExceededMaxMint {
        receiver: Addr,
        shares: Uint128,
        max_shares: Uint128,
    },

    #[error("{owner} withdraw of {assets} assets exceeds the max {max_assets}")]
    ExceededMaxWithdraw {
        owner: Addr,
        assets: Uint128,
        max_assets: Uint128,
    },

    #[error("{owner} withdraw of {shares} shares exceeds the max {max_shares}")]
    ExceededMaxRedeem {
        owner: Addr,
        shares: Uint128,
        max_shares: Uint128,
    },

    #[error("invalid token type for this operation")]
    InvalidTokenType {},

    #[error("invalid staking contract provided")]
    InvalidStakingContract {},

    #[error("insufficient {asset_info} for swap funds")]
    InsufficientSwapFunds { asset_info: AssetInfo },

    #[error("addrs list cannot be empty")]
    EmptyAddrsList {},

    #[error("addrs list exceeds maximum length")]
    MaxedAddrsList {},

    #[error("salt is not valid")]
    InvalidSalt {},

    #[error("share amount cannot be zero")]
    ZeroShareAmount {},

    #[error("asset amount cannot be zero")]
    ZeroAssetAmount {},

    #[error("wrong cw20 received")]
    WrongCw20Received {},

    #[error("insufficient shares: requested {requested}, available {available}")]
    InsufficientShares {
        requested: Uint128,
        available: Uint128,
    },

    #[error("insufficient locked shares: requested {requested}, available {available}")]
    InsufficientLockedShares {
        requested: Uint128,
        available: Uint128,
    },

    #[error("redemption request {id} not found")]
    RedemptionNotFound { id: u64 },

    #[error("redemption request {id} already completed")]
    RedemptionAlreadyCompleted { id: u64 },

    #[error(
        "fee calculation too early: current block {current_block}, required block {required_block}"
    )]
    FeeCalculationTooEarly {
        current_block: u64,
        required_block: u64,
    },

    #[error("deposit amount must be greater than or equal to {minimum_deposit}")]
    DepositTooSmall { minimum_deposit: Uint128 },
}
