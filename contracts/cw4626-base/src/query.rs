use crate::{
    helpers::{
        _convert_to_shares, query_cw20_balance, Rounding, Tokens, _convert_to_assets, get_tokens,
    },
    state::{SHARE, UNDERLYING_ASSET},
};

use cosmwasm_std::{Addr, Deps, StdResult, Storage, Uint128};
use cw4626::*;

pub fn asset(storage: &dyn Storage) -> StdResult<AssetResponse> {
    let asset = UNDERLYING_ASSET.load(storage)?;
    Ok(AssetResponse {
        asset_token_address: asset,
    })
}

pub fn total_assets(this: &Addr, deps: &Deps) -> StdResult<TotalAssetsResponse> {
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    let balance = query_cw20_balance(&deps.querier, &asset, this)?;
    Ok(TotalAssetsResponse {
        total_managed_assets: balance,
    })
}

pub fn convert_to_shares(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
) -> StdResult<ConvertToSharesResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)?;
    Ok(ConvertToSharesResponse { shares })
}

pub fn convert_to_assets(
    this: &Addr,
    deps: &Deps,
    shares: Uint128,
) -> StdResult<ConvertToAssetsResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let assets = _convert_to_shares(total_shares, total_assets, shares, Rounding::Floor)?;
    Ok(ConvertToAssetsResponse { assets })
}

pub fn max_deposit(_receiver: Addr) -> StdResult<MaxDepositResponse> {
    Ok(MaxDepositResponse {
        max_assets: Uint128::MAX,
    })
}

pub fn preview_deposit(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
) -> StdResult<PreviewDepositResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)?;
    Ok(PreviewDepositResponse { shares })
}

pub fn max_mint(deps: &Deps, _receiver: Addr) -> StdResult<MaxMintResponse> {
    let share = SHARE.load(deps.storage)?;
    let cw20::MinterResponse { minter: _, cap } = deps
        .querier
        .query_wasm_smart(&share, &cw20::Cw20QueryMsg::Minter {})?;
    Ok(MaxMintResponse {
        max_shares: cap.unwrap_or(Uint128::MAX),
    })
}

pub fn preview_mint(this: &Addr, deps: &Deps, shares: Uint128) -> StdResult<PreviewMintResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let assets = _convert_to_assets(total_shares, total_assets, shares, Rounding::Ceil)?;
    Ok(PreviewMintResponse { assets })
}

pub fn max_withdraw(this: &Addr, deps: &Deps, owner: Addr) -> StdResult<MaxWithdrawResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let owner_shares_balance =
        cw20_base::contract::query_balance(*deps, owner.to_string())?.balance;
    let assets = _convert_to_assets(
        total_shares,
        total_assets,
        owner_shares_balance,
        Rounding::Floor,
    )?;
    Ok(MaxWithdrawResponse { max_assets: assets })
}

pub fn preview_withdraw(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
) -> StdResult<PreviewWithdrawResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Ceil)?;
    Ok(PreviewWithdrawResponse { shares })
}

pub fn max_redeem(deps: &Deps, owner: Addr) -> StdResult<MaxRedeemResponse> {
    let owner_balance = cw20_base::contract::query_balance(*deps, owner.to_string())?.balance;
    Ok(MaxRedeemResponse {
        max_shares: owner_balance,
    })
}

pub fn preview_redeem(
    this: &Addr,
    deps: &Deps,
    shares: Uint128,
) -> StdResult<PreviewRedeemResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let assets = _convert_to_assets(total_shares, total_assets, shares, Rounding::Floor)?;
    Ok(PreviewRedeemResponse { assets })
}

pub fn ownership(storage: &dyn Storage) -> StdResult<cw_ownable::Ownership<Addr>> {
    cw_ownable::get_ownership(storage)
}
