use cosmwasm_std::StdError;
use cw20_base::ContractError as Cw20ContractError;
use cw4626_base::ContractError as Cw4626BaseContractError;
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

    #[error("only {0} role")]
    Unauthorized(AccessControlRole),

    #[error("tower config is not valid")]
    InvalidTowerConfig {},

    #[error("oracle prices must be greater than zero")]
    OracleZeroPrice {},

    #[error("oracle prices are not valid")]
    OracleInvalidPrices {},
}
