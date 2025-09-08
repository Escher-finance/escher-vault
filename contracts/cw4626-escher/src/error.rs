use astroport::asset::AssetInfo;
use cosmwasm_std::{StdError, Uint128};
use cw20_base::ContractError as Cw20ContractError;
use cw4626_base::ContractError as Cw4626BaseContractError;
use cw_utils::PaymentError;
use thiserror::Error;

use crate::state::AccessControlRole;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ShareCw20Error(#[from] Cw20ContractError),

    #[error("{0}")]
    Cw4626Base(#[from] Cw4626BaseContractError),

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
}
