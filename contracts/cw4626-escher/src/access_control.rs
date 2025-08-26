use cosmwasm_std::{Addr, Storage};

use crate::{
    state::{AccessControlRole, ACCESS_CONTROL},
    ContractError,
};

pub fn only_role(
    storage: &dyn Storage,
    sender: &Addr,
    role: AccessControlRole,
) -> Result<(), ContractError> {
    let unauthorized_err = Err(ContractError::Unauthorized(role.clone()));
    let Ok(Some(address)) = ACCESS_CONTROL.may_load(storage, role.key()) else {
        return unauthorized_err;
    };
    if sender != address {
        return unauthorized_err;
    }
    Ok(())
}
