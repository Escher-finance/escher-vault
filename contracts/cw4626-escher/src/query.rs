use cosmwasm_std::{Deps, StdResult};

use crate::{
    msg::{AccessControlRoleResponse, ConfigResponse, OracleTokensListResponse},
    state::{
        AccessControlRole, ACCESS_CONTROL, ORACLE_PRICES, STAKING_CONTRACT, TOWER_CONFIG,
        UNDERLYING_ASSET, UNDERLYING_DECIMALS,
    },
};

pub fn role(deps: &Deps, kind: AccessControlRole) -> StdResult<AccessControlRoleResponse> {
    let address = ACCESS_CONTROL.load(deps.storage, kind.key())?;
    Ok(AccessControlRoleResponse { address })
}

pub fn oracle_tokens_list(deps: &Deps) -> StdResult<OracleTokensListResponse> {
    let tokens = ORACLE_PRICES
        .load(deps.storage)?
        .into_keys()
        .collect::<Vec<_>>();
    Ok(OracleTokensListResponse { tokens })
}

pub fn config(deps: &Deps) -> StdResult<ConfigResponse> {
    let underlying_asset = UNDERLYING_ASSET.load(deps.storage)?;
    let underlying_decimals = UNDERLYING_DECIMALS.load(deps.storage)?;
    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;
    let tower_config = TOWER_CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        underlying_asset,
        underlying_decimals,
        staking_contract,
        tower_config,
    })
}
