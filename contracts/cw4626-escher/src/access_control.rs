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
    let unauthorized_err = Err(ContractError::Unauthorized(role));
    let Ok(Some(addresses)) = ACCESS_CONTROL.may_load(storage, role.key()) else {
        return unauthorized_err;
    };
    if !addresses.contains(sender) {
        return unauthorized_err;
    }
    Ok(())
}
