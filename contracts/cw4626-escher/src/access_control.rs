use cosmwasm_std::{Addr, Storage, StdError};

use crate::{
    state::{AccessControlRole, ACCESS_CONTROL},
    ContractError,
};

/// Maximum number of addresses allowed in a role to prevent DoS attacks
const MAX_ROLE_SIZE: usize = 20;

/// Validates that an address is not empty and properly formatted
pub fn validate_address(address: &Addr) -> Result<(), ContractError> {
    if address.as_str().is_empty() {
        return Err(ContractError::Std(StdError::generic_err("Empty address not allowed")));
    }
    
    // Additional validation could be added here (e.g., bech32 format check)
    // For now, we rely on CosmWasm's Addr type which already validates bech32 format
    
    Ok(())
}

/// Validates that a role list doesn't exceed size limits
pub fn validate_role_size(addresses: &[Addr]) -> Result<(), ContractError> {
    if addresses.len() > MAX_ROLE_SIZE {
        return Err(ContractError::Std(StdError::generic_err(
            format!("Role size limit exceeded: max {} addresses allowed", MAX_ROLE_SIZE)
        )));
    }
    Ok(())
}

/// Validates that all addresses in a role list are valid
pub fn validate_role_addresses(addresses: &[Addr]) -> Result<(), ContractError> {
    for addr in addresses {
        validate_address(addr)?;
    }
    
    // Check for duplicates
    let mut unique_addresses = std::collections::HashSet::new();
    for addr in addresses {
        if !unique_addresses.insert(addr) {
            return Err(ContractError::Std(StdError::generic_err(
                format!("Duplicate address found in role: {}", addr)
            )));
        }
    }
    
    Ok(())
}

/// Checks if the sender has the required role
pub fn only_role(
    storage: &dyn Storage,
    sender: &Addr,
    role: AccessControlRole,
) -> Result<(), ContractError> {
    // Validate sender address
    validate_address(sender)?;
    
    let unauthorized_err = Err(ContractError::Unauthorized(role.clone()));
    let Ok(Some(addresses)) = ACCESS_CONTROL.may_load(storage, role.key()) else {
        return unauthorized_err;
    };
    
    if !addresses.contains(sender) {
        return unauthorized_err;
    }
    
    Ok(())
}

/// Validates a complete role configuration
pub fn validate_role_config(addresses: &[Addr]) -> Result<(), ContractError> {
    validate_role_size(addresses)?;
    validate_role_addresses(addresses)?;
    Ok(())
}
