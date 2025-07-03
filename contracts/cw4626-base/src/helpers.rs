use cosmwasm_std::{Addr, Deps};

use crate::ContractError;

pub fn validate_cw20(
    deps: &Deps,
    token_address: &Addr,
) -> Result<cw20::TokenInfoResponse, ContractError> {
    deps.querier
        .query_wasm_smart::<cw20::TokenInfoResponse>(
            token_address,
            &cw20::Cw20QueryMsg::TokenInfo {},
        )
        .map_err(|_| ContractError::InvalidCw20 {
            addr: token_address.to_string(),
        })
}
