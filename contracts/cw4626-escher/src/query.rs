use cosmwasm_std::{Deps, StdResult};

use crate::{
    msg::{AccessControlRoleResponse, OracleTokensListResponse},
    state::{AccessControlRole, ACCESS_CONTROL, ORACLE_PRICES},
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
