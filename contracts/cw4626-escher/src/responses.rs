use cosmwasm_std::{Addr, Response, Uint128, StdError, StdResult};

// Security constants for response validation
const MAX_ATTRIBUTE_VALUE_LENGTH: usize = 200;
const MAX_ADDRESS_LENGTH: usize = 100;

/// Validates address for security
fn validate_address(addr: &Addr, field_name: &str) -> StdResult<()> {
    if addr.as_str().is_empty() {
        return Err(StdError::generic_err(format!("{} address cannot be empty", field_name)));
    }
    if addr.as_str().len() > MAX_ADDRESS_LENGTH {
        return Err(StdError::generic_err(format!(
            "{} address too long (max {} characters)", field_name, MAX_ADDRESS_LENGTH
        )));
    }
    Ok(())
}

/// Validates amount for security
fn validate_amount(amount: Uint128, field_name: &str) -> StdResult<()> {
    if amount.is_zero() {
        return Err(StdError::generic_err(format!("{} amount cannot be zero", field_name)));
    }
    // Check for extremely large values that could cause overflow
    if amount > Uint128::new(u128::MAX / 1000) {
        return Err(StdError::generic_err(format!(
            "{} amount too large (potential overflow risk)", field_name
        )));
    }
    Ok(())
}

/// Validates attribute value for security
fn validate_attribute_value(value: &str, field_name: &str) -> StdResult<()> {
    if value.len() > MAX_ATTRIBUTE_VALUE_LENGTH {
        return Err(StdError::generic_err(format!(
            "{} value too long (max {} characters)", field_name, MAX_ATTRIBUTE_VALUE_LENGTH
        )));
    }
    // Check for dangerous characters that could break parsing
    let invalid_chars: Vec<char> = value.chars()
        .filter(|&c| c == '\x00' || c == '\n' || c == '\r')
        .collect();
    if !invalid_chars.is_empty() {
        return Err(StdError::generic_err(format!(
            "{} contains invalid characters: {:?}", field_name, invalid_chars
        )));
    }
    Ok(())
}

pub fn generate_withdraw_response(
    caller: &Addr,
    receiver: &Addr,
    assets: Uint128,
    shares: Uint128,
) -> StdResult<Response> {
    // CRITICAL: Validate all input parameters
    validate_address(caller, "caller")?;
    validate_address(receiver, "receiver")?;
    validate_amount(assets, "assets")?;
    validate_amount(shares, "shares")?;
    
    // CRITICAL: Validate attribute values to prevent injection attacks
    validate_attribute_value(caller.as_str(), "caller")?;
    validate_attribute_value(receiver.as_str(), "receiver")?;
    validate_attribute_value(&assets.to_string(), "assets")?;
    validate_attribute_value(&shares.to_string(), "shares")?;
    
    Ok(Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("withdrawer", caller)
        .add_attribute("receiver", receiver)
        .add_attribute("assets_received", assets)
        .add_attribute("shares_burned", shares))
}

pub fn generate_bond_response(
    sender: &Addr,
    expected: Uint128,
    staking_contract: &Addr,
) -> StdResult<Response> {
    // CRITICAL: Validate all input parameters
    validate_address(sender, "sender")?;
    validate_address(staking_contract, "staking_contract")?;
    validate_amount(expected, "expected")?;
    
    // CRITICAL: Validate attribute values to prevent injection attacks
    validate_attribute_value(sender.as_str(), "sender")?;
    validate_attribute_value(staking_contract.as_str(), "staking_contract")?;
    validate_attribute_value(&expected.to_string(), "expected")?;
    
    Ok(Response::new()
        .add_attribute("action", "bond")
        .add_attribute("sender", sender)
        .add_attribute("expected", expected)
        .add_attribute("staking_contract", staking_contract))
}
