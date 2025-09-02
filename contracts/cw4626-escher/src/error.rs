use cosmwasm_std::{Decimal, StdError, Uint128};
use cw20_base::ContractError as Cw20ContractError;
use cw4626_base::ContractError as Cw4626BaseContractError;
use cw_utils::PaymentError;
use thiserror::Error;

use crate::state::AccessControlRole;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ShareCw20Error(#[from] Cw20ContractError),

    #[error("{0}")]
    Cw4626Base(#[from] Cw4626BaseContractError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    // Access Control Errors
    #[error("only {0} role")]
    Unauthorized(AccessControlRole),

    // Configuration Errors
    #[error("tower config is not valid")]
    InvalidTowerConfig {},

    #[error("oracle prices must be greater than zero")]
    OracleZeroPrice {},

    #[error("oracle prices are not valid")]
    OracleInvalidPrices {},

    #[error("Invalid staking contract provided")]
    InvalidStakingContract {},

    // Business Logic Errors
    #[error("Insufficient funds for operation")]
    InsufficientFunds {},

    #[error("Invalid token type for this operation")]
    InvalidTokenType {},

    // NEW: Validation Errors
    #[error("Validation failed for field '{field}': {reason}")]
    ValidationError { field: String, reason: String },

    #[error("Empty input not allowed for field '{field}'")]
    EmptyInput { field: String },

    #[error("Invalid length for field '{field}': expected {min}-{max} characters, got {actual}")]
    InvalidLength { field: String, min: u32, max: u32, actual: u32 },

    #[error("Invalid characters in field '{field}': {invalid_chars:?}")]
    InvalidCharacters { field: String, invalid_chars: Vec<char> },

    #[error("Invalid range for field '{field}': expected {min}-{max}, got {actual}")]
    InvalidRange { field: String, min: String, max: String, actual: String },

    // NEW: Security Errors
    #[error("Security validation failed: {reason}")]
    SecurityError { reason: String },

    #[error("Invalid amount: {amount} - {reason}")]
    InvalidAmount { amount: String, reason: String },

    #[error("Invalid slippage tolerance: {slippage}% - must be between {min}% and {max}%")]
    InvalidSlippage { slippage: String, min: String, max: String },

    // NEW: Math Errors
    #[error("Mathematical operation failed: {operation} - {reason}")]
    MathError { operation: String, reason: String },

    #[error("Overflow detected in {operation}")]
    OverflowError { operation: String },

    #[error("Underflow detected in {operation}")]
    UnderflowError { operation: String },
}

// Helper functions for creating specific error types
impl ContractError {
    /// Creates a validation error with field and reason
    pub fn validation_error(field: &str, reason: &str) -> Self {
        ContractError::ValidationError {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates an empty input error
    pub fn empty_input(field: &str) -> Self {
        ContractError::EmptyInput {
            field: field.to_string(),
        }
    }

    /// Creates an invalid length error
    pub fn invalid_length(field: &str, min: u32, max: u32, actual: u32) -> Self {
        ContractError::InvalidLength {
            field: field.to_string(),
            min,
            max,
            actual,
        }
    }

    /// Creates an invalid characters error
    pub fn invalid_characters(field: &str, invalid_chars: Vec<char>) -> Self {
        ContractError::InvalidCharacters {
            field: field.to_string(),
            invalid_chars,
        }
    }

    /// Creates an invalid range error
    pub fn invalid_range(field: &str, min: &str, max: &str, actual: &str) -> Self {
        ContractError::InvalidRange {
            field: field.to_string(),
            min: min.to_string(),
            max: max.to_string(),
            actual: actual.to_string(),
        }
    }

    /// Creates a security error
    pub fn security_error(reason: &str) -> Self {
        ContractError::SecurityError {
            reason: reason.to_string(),
        }
    }

    /// Creates an invalid amount error
    pub fn invalid_amount(amount: Uint128, reason: &str) -> Self {
        ContractError::InvalidAmount {
            amount: amount.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates an invalid slippage error
    pub fn invalid_slippage(slippage: Decimal, min: Decimal, max: Decimal) -> Self {
        ContractError::InvalidSlippage {
            slippage: slippage.to_string(),
            min: min.to_string(),
            max: max.to_string(),
        }
    }

    /// Creates a math error
    pub fn math_error(operation: &str, reason: &str) -> Self {
        ContractError::MathError {
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates an overflow error
    pub fn overflow_error(operation: &str) -> Self {
        ContractError::OverflowError {
            operation: operation.to_string(),
        }
    }

    /// Creates an underflow error
    pub fn underflow_error(operation: &str) -> Self {
        ContractError::UnderflowError {
            operation: operation.to_string(),
        }
    }
}
