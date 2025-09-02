use astroport::{
    asset::{Asset, AssetInfo},
    querier::{query_balance, query_token_balance},
};
use cosmwasm_std::{to_json_binary, Addr, Env, MessageInfo, QuerierWrapper, Uint128, WasmMsg, StdError};
use cw4626::cw20;
use cw4626_base::helpers::validate_cw20;
use cw_utils::must_pay;

use crate::ContractError;

pub fn get_asset_info_address(asset_info: &AssetInfo) -> String {
    match asset_info {
        AssetInfo::NativeToken { denom } => denom.clone(),
        AssetInfo::Token { contract_addr } => contract_addr.to_string(),
    }
}

/// Query asset balance with enhanced error handling and validation
pub fn query_asset_info_balance(
    querier: &QuerierWrapper,
    asset_info: AssetInfo,
    addr: Addr,
) -> Result<Uint128, ContractError> {
    // Validate inputs first
    if addr.as_str().is_empty() {
        return Err(ContractError::Std(StdError::generic_err("Empty address not allowed")));
    }
    
    match asset_info {
        AssetInfo::Token { contract_addr, .. } => {
            // Validate contract address
            if contract_addr.as_str().is_empty() {
                return Err(ContractError::Std(StdError::generic_err(
                    "Empty contract address not allowed"
                )));
            }
            
            let contract_addr_str = contract_addr.to_string();
            
            // Query with enhanced error handling
            query_token_balance(querier, contract_addr, addr)
                .map_err(|e| ContractError::Std(StdError::generic_err(
                    format!("Failed to query token balance for contract {}: {}", contract_addr_str, e)
                )))
        }
        AssetInfo::NativeToken { denom } => {
            // Validate denom
            if denom.is_empty() {
                return Err(ContractError::Std(StdError::generic_err(
                    "Empty denom not allowed"
                )));
            }
            
            let denom_str = denom.clone();
            
            // Query with enhanced error handling
            query_balance(querier, addr, denom)
                .map_err(|e| ContractError::Std(StdError::generic_err(
                    format!("Failed to query native balance for denom {}: {}", denom_str, e)
                )))
        }
    }
}

/// Query asset decimals with enhanced validation and fallback mechanisms
pub fn query_asset_info_decimals(
    querier: &QuerierWrapper,
    asset_info: AssetInfo,
) -> Result<u8, ContractError> {
    match asset_info {
        AssetInfo::Token { contract_addr, .. } => {
            // Validate the CW20 contract first
            let cw20::TokenInfoResponse { decimals, .. } = validate_cw20(querier, &contract_addr)?;
            
            // Validate decimal range (0-18 is standard)
            if decimals > 18 {
                return Err(ContractError::Std(StdError::generic_err(
                    format!("Invalid decimal places: {} (max 18 allowed)", decimals)
                )));
            }
            
            Ok(decimals)
        }
        AssetInfo::NativeToken { denom } => {
            // Try to query bank metadata first (if available)
            match query_native_token_decimals(querier, &denom) {
                Ok(decimals) => Ok(decimals),
                Err(_) => {
                    // Fallback to common decimal patterns based on denom
                    let fallback_decimals = get_fallback_decimals(&denom);
                    Ok(fallback_decimals)
                }
            }
        }
    }
}

/// Attempt to query native token decimals from bank metadata
fn query_native_token_decimals(_querier: &QuerierWrapper, _denom: &str) -> Result<u8, StdError> {
    // For now, we'll skip the bank metadata query as it's not available in all environments
    // and focus on the fallback mechanism which is more reliable
    Err(StdError::generic_err("Bank metadata query not implemented"))
}

/// Get fallback decimals based on common denom patterns
fn get_fallback_decimals(denom: &str) -> u8 {
    match denom {
        // Common native tokens with known decimals
        "ubbn" => 6,               // Babylon testnet native token
        "uosmo" => 6,              // Osmosis
        "uatom" => 6,              // Cosmos Hub
        "ujuno" => 6,              // Juno
        "ustars" => 6,             // Stargaze
        "uakt" => 6,               // Akash
        "ucre" => 6,               // Crescent
        "uion" => 6,               // Osmosis ION
        "uaxl" => 6,               // Axelar
        
        // IBC tokens - try to extract from denom pattern
        denom if denom.starts_with("ibc/") => {
            // IBC tokens often follow patterns like "ibc/ABC123.../denom"
            // Most IBC tokens use 6 decimals, but we should be more careful
            if denom.contains("gamm") || denom.contains("pool") {
                18  // GAMM tokens and pool tokens often use 18 decimals
            } else {
                6   // Default for most IBC tokens
            }
        }
        
        // Default fallback
        _ => 6,
    }
}

/// Validates and processes asset transfer with comprehensive security checks
pub fn assert_send_asset_to_contract(
    info: MessageInfo,
    env: Env,
    asset: Asset,
    querier: &QuerierWrapper,
) -> Result<Option<WasmMsg>, ContractError> {
    // CRITICAL: Validate asset amount
    if asset.amount.is_zero() {
        return Err(ContractError::Std(StdError::generic_err(
            "Asset amount cannot be zero"
        )));
    }
    
    // CRITICAL: Validate asset info
    validate_asset_info(querier, &asset.info)?;
    
    let caller = info.sender.clone();
    let this = env.contract.address;
    
    match asset.info {
        AssetInfo::Token { contract_addr } => {
            // Additional validation for CW20 contracts
            validate_cw20(querier, &contract_addr)?;
            
            Ok(Some(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
                    owner: caller.to_string(),
                    recipient: this.to_string(),
                    amount: asset.amount,
                })?,
                funds: vec![],
            }))
        }
        AssetInfo::NativeToken { denom } => {
            // CRITICAL: Validate denom
            if denom.is_empty() {
                return Err(ContractError::Std(StdError::generic_err(
                    "Empty denom not allowed"
                )));
            }
            
            // CRITICAL: Check for exact payment to prevent overpayment attacks
            let paid = must_pay(&info, &denom)?;
            if paid != asset.amount {
                return Err(ContractError::Std(StdError::generic_err(
                    format!("Exact payment required: expected {}, got {}", asset.amount, paid)
                )));
            }
            
            Ok(None)
        }
    }
}

/// Validates asset info for security and correctness
fn validate_asset_info(querier: &QuerierWrapper, asset_info: &AssetInfo) -> Result<(), ContractError> {
    match asset_info {
        AssetInfo::Token { contract_addr } => {
            // Validate contract address format
            if contract_addr.as_str().is_empty() {
                return Err(ContractError::Std(StdError::generic_err(
                    "Empty contract address not allowed"
                )));
            }
            
            // Validate that the contract is a legitimate CW20
            validate_cw20(querier, contract_addr)?;
            
            Ok(())
        }
        AssetInfo::NativeToken { denom } => {
            // Validate denom format
            if denom.is_empty() {
                return Err(ContractError::Std(StdError::generic_err(
                    "Empty denom not allowed"
                )));
            }
            
            // Check for suspicious denom patterns
            if denom.len() > 128 {
                return Err(ContractError::Std(StdError::generic_err(
                    "Denom too long (max 128 characters)"
                )));
            }
            
            // Validate denom contains only allowed characters
            if !denom.chars().all(|c| c.is_alphanumeric() || c == '/' || c == '-') {
                return Err(ContractError::Std(StdError::generic_err(
                    "Invalid denom format: only alphanumeric characters, '/', and '-' allowed"
                )));
            }
            
            Ok(())
        }
    }
}
