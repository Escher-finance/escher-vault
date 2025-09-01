use cosmwasm_std::{Deps, StdResult};

use crate::{
    asset_info::get_asset_info_address,
    msg::{AccessControlRoleResponse, ConfigResponse, OracleTokensListResponse},
    state::{
        AccessControlRole, ACCESS_CONTROL, ORACLE_PRICES, STAKING_CONTRACT, TOWER_CONFIG,
        UNDERLYING_ASSET,
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
    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;
    let tower_config = TOWER_CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        staking_contract,
        tower_config,
    })
}

pub fn asset(deps: &Deps) -> StdResult<cw4626::AssetResponse> {
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    Ok(cw4626::AssetResponse {
        asset_token_address: get_asset_info_address(&asset),
    })
}
