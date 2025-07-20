use cosmwasm_std::{Addr, StdError, Uint128};
use cw20_base::ContractError as Cw20ContractError;
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("{0}")]
    ShareCw20Error(#[from] Cw20ContractError),

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
}
