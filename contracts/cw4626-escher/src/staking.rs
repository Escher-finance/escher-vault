use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Timestamp, Uint128, StdError, StdResult};

// Security constants for staking validation
const MAX_RECIPIENT_LENGTH: usize = 100;
const MAX_SALT_LENGTH: usize = 100;
const MAX_CHANNEL_ID: u32 = 10000;
const MIN_SLIPPAGE_PERCENT: u64 = 1; // 1%
const MAX_SLIPPAGE_PERCENT: u64 = 50; // 50%

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

/// Validates slippage for security
fn validate_slippage(slippage: Option<Decimal>) -> StdResult<()> {
    if let Some(slippage) = slippage {
        let slippage_percent = slippage.atomics() / Decimal::percent(1).atomics();
        if slippage_percent < Uint128::from(MIN_SLIPPAGE_PERCENT) {
            return Err(StdError::generic_err(format!(
                "Slippage too low (min {}%)", MIN_SLIPPAGE_PERCENT
            )));
        }
        if slippage_percent > Uint128::from(MAX_SLIPPAGE_PERCENT) {
            return Err(StdError::generic_err(format!(
                "Slippage too high (max {}%)", MAX_SLIPPAGE_PERCENT
            )));
        }
    }
    Ok(())
}

/// Validates string parameter for security
fn validate_string_param(value: &Option<String>, field_name: &str, max_length: usize) -> StdResult<()> {
    if let Some(value) = value {
        if value.is_empty() {
            return Err(StdError::generic_err(format!("{} cannot be empty", field_name)));
        }
        if value.len() > max_length {
            return Err(StdError::generic_err(format!(
                "{} too long (max {} characters)", field_name, max_length
            )));
        }
        // Check for dangerous characters
        let invalid_chars: Vec<char> = value.chars()
            .filter(|&c| c == '\x00' || c == '\n' || c == '\r')
            .collect();
        if !invalid_chars.is_empty() {
            return Err(StdError::generic_err(format!(
                "{} contains invalid characters: {:?}", field_name, invalid_chars
            )));
        }
    }
    Ok(())
}

/// Validates channel ID for security
fn validate_channel_id(channel_id: &Option<u32>) -> StdResult<()> {
    if let Some(channel_id) = channel_id {
        if *channel_id == 0 {
            return Err(StdError::generic_err("Channel ID cannot be zero"));
        }
        if *channel_id > MAX_CHANNEL_ID {
            return Err(StdError::generic_err(format!(
                "Channel ID too large (max {})", MAX_CHANNEL_ID
            )));
        }
    }
    Ok(())
}

#[cw_serde]
#[derive(Default)]
pub struct EscherHubStakingLiquidity {
    pub amount: Uint128,
    pub delegated: Uint128,
    pub reward: Uint128,
    pub unclaimed_reward: Uint128,
    pub exchange_rate: Decimal,
    pub time: Timestamp,
    pub total_supply: Uint128,
    pub adjusted_supply: Uint128,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum EscherHubQueryMsg {
    #[returns(EscherHubStakingLiquidity)]
    StakingLiquidity {},
}

#[cw_serde]
pub enum EscherHubExecuteMsg {
    Bond {
        slippage: Option<Decimal>,
        expected: Uint128,
        recipient: Option<String>,
        recipient_channel_id: Option<u32>,
        salt: Option<String>,
    },
}

impl EscherHubExecuteMsg {
    /// Validates the Bond message for security
    pub fn validate_bond(&self) -> StdResult<()> {
        match self {
            EscherHubExecuteMsg::Bond {
                slippage,
                expected,
                recipient,
                recipient_channel_id,
                salt,
            } => {
                // CRITICAL: Validate all input parameters
                validate_amount(*expected, "expected")?;
                validate_slippage(*slippage)?;
                validate_string_param(recipient, "recipient", MAX_RECIPIENT_LENGTH)?;
                validate_channel_id(recipient_channel_id)?;
                validate_string_param(salt, "salt", MAX_SALT_LENGTH)?;
                
                Ok(())
            }
        }
    }
}
