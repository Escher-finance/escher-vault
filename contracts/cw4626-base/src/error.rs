use cosmwasm_std::StdError;
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
    InvalidCw20 { addr: String },

    #[error("Share must have this contract as the minter")]
    InvalidSharesMinter {},

    #[error("Shares minter must have a nonzero cap")]
    SharesMinterCapTooSmall {},

    #[error("The share token has not been set")]
    ShareTokenNotConnected {},

    #[error("{receiver} deposit of {assets} assets exceeds the max {max_assets}")]
    ExceededMaxDeposit {
        receiver: String,
        assets: u128,
        max_assets: u128,
    },

    #[error("{receiver} mint of {shares} shares exceeds the max {max_shares}")]
    ExceededMaxMint {
        receiver: String,
        shares: u128,
        max_shares: u128,
    },

    #[error("{owner} withdraw of {assets} assets exceeds the max {max_assets}")]
    ExceededMaxWithdraw {
        owner: String,
        assets: u128,
        max_assets: u128,
    },

    #[error("{owner} withdraw of {shares} shares exceeds the max {max_shares}")]
    ExceededMaxRedeem {
        owner: String,
        shares: u128,
        max_shares: u128,
    },

    #[error("Cannot set withdrawal share allowance to own account")]
    CannotSetWithdrawalShareAllowanceToOwnAccount {},

    #[error("Invalid withdrawal share allowance expiration")]
    InvalidWithdrawalShareAllowanceExpiration {},

    #[error("Insufficient withdrawal share allowance")]
    InsufficientWithdrawalShareAllowance {},

    #[error("Withdrawal share allowance has expired")]
    ExpiredWithdrawalShareAllowance {},
}
