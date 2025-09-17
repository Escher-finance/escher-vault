use cosmwasm_std::{Addr, Storage};

use crate::{
    ContractError,
    error::ContractResult,
    state::{ACCESS_CONTROL, AccessControlRole, PAUSED_STATUS, PausedStatus},
};

/// Validates that the `sender` has to have the specified `role`
///
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

/// # Errors
/// Will return error if validation fails
pub fn validate_only_not_paused(storage: &dyn Storage, sender: &Addr) -> ContractResult<()> {
    let managers = ACCESS_CONTROL.load(storage, AccessControlRole::Manager {}.key())?;
    let is_manager = managers.contains(sender);
    let paused_status = PAUSED_STATUS.load(storage)?;
    match paused_status {
        PausedStatus::NotPaused {} => return Ok(()),
        PausedStatus::PausedMaintenance {} => {
            if is_manager {
                return Ok(());
            }
        }
        PausedStatus::PausedOngoingBonding {} => {}
    }
    Err(ContractError::Paused(paused_status))
}

pub fn internal_toggle_paused_status(storage: &mut dyn Storage) -> ContractResult<()> {
    PAUSED_STATUS.update::<_, ContractError>(storage, |status| match status {
        PausedStatus::NotPaused {} => Ok(PausedStatus::PausedMaintenance {}),
        PausedStatus::PausedMaintenance {} | PausedStatus::PausedOngoingBonding {} => {
            Ok(PausedStatus::NotPaused {})
        }
    })?;
    Ok(())
}
