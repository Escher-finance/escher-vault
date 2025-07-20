use cosmwasm_std::StdError;
use cw20_base::ContractError as Cw20ContractError;
use cw4626_base::ContractError as Cw4626BaseContractError;
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

    #[error("{0}")]
    Cw4626Base(#[from] Cw4626BaseContractError),
}
