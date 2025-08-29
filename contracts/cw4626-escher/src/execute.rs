use std::collections::HashMap;

use cosmwasm_std::{Addr, Decimal, DepsMut, Response};

use crate::{
    access_control::only_role,
    state::{AccessControlRole, ACCESS_CONTROL},
    tower::update_and_validate_prices,
    ContractError,
};

pub fn update_role(
    deps: DepsMut,
    sender: Addr,
    role: AccessControlRole,
    address: Addr,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &sender, AccessControlRole::Manager {})?;
    ACCESS_CONTROL.save(deps.storage, role.key(), &address)?;
    Ok(Response::new())
}

pub fn oracle_update_prices(
    deps: DepsMut,
    sender: Addr,
    prices: HashMap<String, Decimal>,
) -> Result<Response, ContractError> {
    only_role(deps.storage, &sender, AccessControlRole::Oracle {})?;
    update_and_validate_prices(deps, prices)?;
    Ok(Response::new())
}
