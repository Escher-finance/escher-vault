use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("{addr} is not a cw20")]
    InvalidCw20 { addr: String },

    #[error("Share and asset must have the same decimals")]
    DecimalsMismatch {},
}
