use cosmwasm_std::{Deps, StdResult};

use crate::{
    msg::AccessControlRoleResponse,
    state::{AccessControlRole, ACCESS_CONTROL},
};

pub fn role(deps: &Deps, kind: AccessControlRole) -> StdResult<AccessControlRoleResponse> {
    let address = ACCESS_CONTROL.load(deps.storage, kind.key())?;
    Ok(AccessControlRoleResponse { address })
}
