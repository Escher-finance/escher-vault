use cosmwasm_std::{Addr, Decimal, Deps, StdError, StdResult, Uint128};

use crate::{
    asset::get_asset_info_address,
    helpers::{
        Rounding, Tokens, _convert_to_assets, _convert_to_shares, _preview_deposit, get_tokens,
    },
    msg::{
        AccessControlRoleResponse, ConfigResponse, ExchangeRateResponse, GitInfoResponse,
        OraclePricesResponse, OracleTokensListResponse,
    },
    state::{
        AccessControlRole, ACCESS_CONTROL, ORACLE_PRICES, STAKING_CONTRACT, TOWER_CONFIG,
        UNDERLYING_ASSET,
    },
    tower::calculate_total_assets,
};

pub fn git_info() -> StdResult<GitInfoResponse> {
    let git = format!("{}:{}", env!("VERGEN_GIT_BRANCH"), env!("VERGEN_GIT_SHA"));
    Ok(GitInfoResponse { git })
}

pub fn role(deps: &Deps, kind: AccessControlRole) -> StdResult<AccessControlRoleResponse> {
    let addresses = ACCESS_CONTROL.load(deps.storage, kind.key())?;
    Ok(AccessControlRoleResponse { addresses })
}

pub fn oracle_tokens_list(deps: &Deps) -> StdResult<OracleTokensListResponse> {
    let tokens = ORACLE_PRICES
        .load(deps.storage)?
        .into_keys()
        .collect::<Vec<_>>();
    Ok(OracleTokensListResponse { tokens })
}

pub fn oracle_prices(deps: &Deps) -> StdResult<OraclePricesResponse> {
    let prices = ORACLE_PRICES.load(deps.storage)?;
    Ok(OraclePricesResponse { prices })
}

pub fn config(deps: &Deps) -> StdResult<ConfigResponse> {
    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;
    let tower_config = TOWER_CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        staking_contract,
        tower_config,
    })
}

pub fn asset(deps: &Deps) -> StdResult<cw4626::AssetResponse> {
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    Ok(cw4626::AssetResponse {
        asset_token_address: get_asset_info_address(&asset),
    })
}

pub fn total_assets(deps: &Deps, this: Addr) -> StdResult<cw4626::TotalAssetsResponse> {
    let total_managed_assets = calculate_total_assets(&deps.querier, deps.storage, this)
        .map_err(|err| StdError::generic_err(err.to_string()))?;
    Ok(cw4626::TotalAssetsResponse {
        total_managed_assets,
    })
}

pub fn convert_to_shares(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
) -> StdResult<cw4626::ConvertToSharesResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)?;
    Ok(cw4626::ConvertToSharesResponse { shares })
}

pub fn convert_to_assets(
    this: &Addr,
    deps: &Deps,
    shares: Uint128,
) -> StdResult<cw4626::ConvertToAssetsResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let assets = _convert_to_assets(total_shares, total_assets, shares, Rounding::Floor)?;
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
    _preview_deposit(this, deps, assets)
}

pub fn preview_mint(
    this: &Addr,
    deps: &Deps,
    shares: Uint128,
) -> StdResult<cw4626::PreviewMintResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let assets = _convert_to_assets(total_shares, total_assets, shares, Rounding::Ceil)?;
    Ok(cw4626::PreviewMintResponse { assets })
}

pub fn exchange_rate(this: &Addr, deps: &Deps) -> StdResult<ExchangeRateResponse> {
    // Check total supply first to avoid requiring oracle prices for zero-state
    let token_info = cw20_base::contract::query_token_info(*deps)?;
    if token_info.total_supply.is_zero() {
        return Ok(ExchangeRateResponse {
            exchange_rate: Decimal::one(),
        });
    }

    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;

    let assets_dec = Decimal::from_ratio(total_assets, Uint128::one());
    let shares_dec = Decimal::from_ratio(total_shares, Uint128::one());
    let exchange_rate = assets_dec / shares_dec;
    Ok(ExchangeRateResponse { exchange_rate })
}
