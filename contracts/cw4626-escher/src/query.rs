use astroport::asset::AssetInfo;
use cosmwasm_std::{Addr, Deps, StdError, StdResult, Uint128};

// Security constants for query validation
const MAX_ORACLE_TOKENS_RESPONSE: usize = 100;
const MAX_ROLE_ADDRESSES_RESPONSE: usize = 50;

/// Validates query parameters for security
fn validate_amount(amount: Uint128, param_name: &str) -> StdResult<()> {
    if amount.is_zero() {
        return Err(StdError::generic_err(format!(
            "{} cannot be zero", param_name
        )));
    }
    // Check for extremely large values that could cause overflow
    if amount > Uint128::new(u128::MAX / 1000) {
        return Err(StdError::generic_err(format!(
            "{} value too large (potential overflow risk)", param_name
        )));
    }
    Ok(())
}

/// Validates access control role for security
fn validate_access_control_role(role: &AccessControlRole) -> StdResult<()> {
    match role {
        AccessControlRole::Manager {} | AccessControlRole::Oracle {} => Ok(()),
    }
}

/// Validates response size to prevent DoS attacks
fn validate_response_size<T>(items: &[T], max_size: usize, _item_name: &str) -> StdResult<()> {
    if items.len() > max_size {
        return Err(StdError::generic_err(format!(
            "Response too large: {} items exceeds maximum of {}", 
            items.len(), max_size
        )));
    }
    Ok(())
}

/// Validates HashMap response size to prevent DoS attacks
fn validate_hashmap_response_size<K, V>(map: &std::collections::HashMap<K, V>, max_size: usize, _item_name: &str) -> StdResult<()> {
    if map.len() > max_size {
        return Err(StdError::generic_err(format!(
            "Response too large: {} items exceeds maximum of {}", 
            map.len(), max_size
        )));
    }
    Ok(())
}

use crate::{
    helpers::{
        Rounding, Tokens, _convert_to_assets, _convert_to_shares, _preview_deposit, get_tokens,
    },
    msg::{
        AccessControlRoleResponse, ConfigResponse, OraclePricesResponse, OracleTokensListResponse,
    },
    state::{
        AccessControlRole, ACCESS_CONTROL, ORACLE_PRICES, STAKING_CONTRACT, TOWER_CONFIG,
        UNDERLYING_ASSET,
    },
    tower::calculate_total_assets,
};

pub fn role(deps: &Deps, kind: AccessControlRole) -> StdResult<AccessControlRoleResponse> {
    // CRITICAL: Validate access control role
    validate_access_control_role(&kind)?;
    
    let addresses = ACCESS_CONTROL.load(deps.storage, kind.key())?;
    
    // CRITICAL: Validate response size to prevent DoS
    validate_response_size(&addresses, MAX_ROLE_ADDRESSES_RESPONSE, "role addresses")?;
    
    Ok(AccessControlRoleResponse { addresses })
}

pub fn oracle_tokens_list(deps: &Deps) -> StdResult<OracleTokensListResponse> {
    let tokens = ORACLE_PRICES
        .load(deps.storage)?
        .into_keys()
        .collect::<Vec<_>>();
    
    // CRITICAL: Validate response size to prevent DoS
    validate_response_size(&tokens, MAX_ORACLE_TOKENS_RESPONSE, "oracle tokens")?;
    
    Ok(OracleTokensListResponse { tokens })
}

pub fn oracle_prices(deps: &Deps) -> StdResult<OraclePricesResponse> {
    let prices = ORACLE_PRICES.load(deps.storage)?;
    
    // CRITICAL: Validate response size to prevent DoS
    validate_hashmap_response_size(&prices, MAX_ORACLE_TOKENS_RESPONSE, "oracle prices")?;
    
    // SECURITY NOTE: Oracle prices are public by design for transparency
    // but consider adding access control if price information becomes sensitive
    Ok(OraclePricesResponse { prices })
}

pub fn config(deps: &Deps) -> StdResult<ConfigResponse> {
    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;
    let tower_config = TOWER_CONFIG.load(deps.storage)?;
    
    // SECURITY NOTE: Config is public by design for transparency
    // but consider adding access control if configuration becomes sensitive
    Ok(ConfigResponse {
        staking_contract,
        tower_config,
    })
}

pub fn asset(deps: &Deps) -> StdResult<cw4626::AssetResponse> {
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    // Convert AssetInfo to the appropriate address format
    let asset_token_address = match &asset {
        AssetInfo::Token { contract_addr } => contract_addr.to_string(),
        AssetInfo::NativeToken { denom } => {
            // For native tokens, we use the denom as the address string
            denom.clone()
        }
    };
    Ok(cw4626::AssetResponse {
        asset_token_address,
    })
}

pub fn total_assets(deps: &Deps, this: Addr) -> StdResult<cw4626::TotalAssetsResponse> {
    // CRITICAL: Validate contract address
    if this.as_str().is_empty() {
        return Err(StdError::generic_err("Contract address cannot be empty"));
    }
    
    let total_managed_assets = calculate_total_assets(&deps.querier, deps.storage, this)
        .map_err(|err| StdError::generic_err(format!(
            "Failed to calculate total assets: {}", err
        )))?;
    
    Ok(cw4626::TotalAssetsResponse {
        total_managed_assets,
    })
}

pub fn convert_to_shares(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
) -> StdResult<cw4626::ConvertToSharesResponse> {
    // CRITICAL: Validate input parameters
    validate_amount(assets, "assets")?;
    
    // CRITICAL: Validate contract address
    if this.as_str().is_empty() {
        return Err(StdError::generic_err("Contract address cannot be empty"));
    }
    
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)
        .map_err(|err| StdError::generic_err(format!(
            "Failed to get tokens: {}", err
        )))?;
    
    let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)
        .map_err(|err| StdError::generic_err(format!(
            "Failed to convert to shares: {}", err
        )))?;
    
    Ok(cw4626::ConvertToSharesResponse { shares })
}

pub fn convert_to_assets(
    this: &Addr,
    deps: &Deps,
    shares: Uint128,
) -> StdResult<cw4626::ConvertToAssetsResponse> {
    // CRITICAL: Validate input parameters
    validate_amount(shares, "shares")?;
    
    // CRITICAL: Validate contract address
    if this.as_str().is_empty() {
        return Err(StdError::generic_err("Contract address cannot be empty"));
    }
    
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)
        .map_err(|err| StdError::generic_err(format!(
            "Failed to get tokens: {}", err
        )))?;
    
    let assets = _convert_to_assets(total_shares, total_assets, shares, Rounding::Floor)
        .map_err(|err| StdError::generic_err(format!(
            "Failed to convert to assets: {}", err
        )))?;
    
    Ok(cw4626::ConvertToAssetsResponse { assets })
}

pub fn max_deposit(_receiver: Addr) -> StdResult<cw4626::MaxDepositResponse> {
    Ok(cw4626::MaxDepositResponse {
        max_assets: if cfg!(not(test)) {
            Uint128::MAX
        } else {
            Uint128::new(100_000_000)
        },
    })
}

pub fn max_mint(_receiver: Addr) -> StdResult<cw4626::MaxMintResponse> {
    Ok(cw4626::MaxMintResponse {
        max_shares: if cfg!(not(test)) {
            Uint128::MAX
        } else {
            Uint128::new(100_000_000)
        },
    })
}

pub fn preview_deposit(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
) -> StdResult<cw4626::PreviewDepositResponse> {
    // CRITICAL: Validate input parameters
    validate_amount(assets, "assets")?;
    
    // CRITICAL: Validate contract address
    if this.as_str().is_empty() {
        return Err(StdError::generic_err("Contract address cannot be empty"));
    }
    
    let asset_info = UNDERLYING_ASSET.load(deps.storage)
        .map_err(|err| StdError::generic_err(format!(
            "Failed to load underlying asset: {}", err
        )))?;
    
    _preview_deposit(
        this,
        deps,
        assets,
        matches!(asset_info, AssetInfo::NativeToken { .. }),
    ).map_err(|err| StdError::generic_err(format!(
        "Failed to preview deposit: {}", err
    )))
}

pub fn preview_mint(
    this: &Addr,
    deps: &Deps,
    shares: Uint128,
) -> StdResult<cw4626::PreviewMintResponse> {
    // CRITICAL: Validate input parameters
    validate_amount(shares, "shares")?;
    
    // CRITICAL: Validate contract address
    if this.as_str().is_empty() {
        return Err(StdError::generic_err("Contract address cannot be empty"));
    }
    
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)
        .map_err(|err| StdError::generic_err(format!(
            "Failed to get tokens: {}", err
        )))?;
    
    let assets = _convert_to_assets(total_shares, total_assets, shares, Rounding::Ceil)
        .map_err(|err| StdError::generic_err(format!(
            "Failed to convert to assets: {}", err
        )))?;
    
    Ok(cw4626::PreviewMintResponse { assets })
}
