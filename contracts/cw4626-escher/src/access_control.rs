use cosmwasm_std::{Addr, Storage};

use crate::{
    error::ContractResult,
    state::{AccessControlRole, ACCESS_CONTROL},
    ContractError,
};

/// # Errors
/// Will return error if validation fails
pub fn validate_only_role(
    storage: &dyn Storage,
    sender: &Addr,
    role: AccessControlRole,
) -> ContractResult<()> {
    let unauthorized_err = Err(ContractError::Unauthorized(role));
    let Ok(Some(addresses)) = ACCESS_CONTROL.may_load(storage, role.key()) else {
        return unauthorized_err;
    };
    if !addresses.contains(sender) {
        return unauthorized_err;
    }
    Ok(())
}
