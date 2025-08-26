use cosmwasm_std::{Addr, DepsMut, Response};

use crate::{
    access_control::only_role,
    state::{AccessControlRole, ACCESS_CONTROL},
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
