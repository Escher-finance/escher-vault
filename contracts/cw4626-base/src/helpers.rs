use cosmwasm_std::{Addr, QuerierWrapper, StdError, Uint128};

use crate::ContractError;

pub fn validate_cw20(
    querier: &QuerierWrapper,
    token_address: &Addr,
) -> Result<cw20::TokenInfoResponse, ContractError> {
    querier
        .query_wasm_smart::<cw20::TokenInfoResponse>(
            token_address,
            &cw20::Cw20QueryMsg::TokenInfo {},
        )
        .map_err(|_| ContractError::InvalidCw20 {
            addr: token_address.to_string(),
        })
}

pub fn query_cw20_balance(
    querier: &QuerierWrapper,
    token: &Addr,
    user: &Addr,
) -> Result<Uint128, StdError> {
    let cw20::BalanceResponse { balance } = querier.query_wasm_smart(
        token,
        &cw20::Cw20QueryMsg::Balance {
            address: user.to_string(),
        },
    )?;
    Ok(balance)
}

#[derive(Debug)]
pub enum Rounding {
    Floor,
    Ceil,
}

pub fn _convert_to_shares(
    total_shares: Uint128,
    total_assets: Uint128,
    assets: Uint128,
    rounding: Rounding,
) -> Result<Uint128, StdError> {
    let frac = (total_shares, total_assets + Uint128::one());
    match rounding {
        Rounding::Ceil => assets.checked_mul_ceil(frac),
        Rounding::Floor => assets.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(e.to_string()))
}

pub fn _convert_to_assets(
    total_shares: Uint128,
    total_assets: Uint128,
    shares: Uint128,
    rounding: Rounding,
) -> Result<Uint128, StdError> {
    let frac = (total_assets + Uint128::one(), total_shares);
    match rounding {
        Rounding::Ceil => shares.checked_mul_ceil(frac),
        Rounding::Floor => shares.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(e.to_string()))
}
