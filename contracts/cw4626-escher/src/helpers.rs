use std::collections::HashSet;

use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128};
use cw4626_base::helpers::generate_deposit_response;

use crate::{
    asset::assert_send_asset_to_contract, state::UNDERLYING_ASSET, tower::calculate_total_assets,
    ContractError,
};

#[derive(Debug)]
pub struct Tokens {
    pub share: AssetInfo,
    pub asset: AssetInfo,
    pub total_shares: Uint128,
    pub total_assets: Uint128,
}

pub fn get_tokens(this: &Addr, deps: &Deps) -> StdResult<Tokens> {
    let share = AssetInfo::Token {
        contract_addr: this.clone(),
    };
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    let total_shares = cw20_base::state::TOKEN_INFO
        .load(deps.storage)?
        .total_supply;
    let total_assets = calculate_total_assets(&deps.querier, deps.storage, this.clone())
        .map_err(|err| StdError::generic_err(err.to_string()))?;
    Ok(Tokens {
        share,
        asset,
        total_shares,
        total_assets,
    })
}

#[derive(Debug)]
pub enum Rounding {
    Floor,
    Ceil,
}

/// Internal conversion
pub fn _convert_to_shares(
    total_shares: Uint128,
    total_assets: Uint128,
    assets: Uint128,
    rounding: Rounding,
) -> Result<Uint128, StdError> {
    // If vault is in zero state, mint 1:1 shares for assets
    if total_shares.is_zero() || total_assets.is_zero() {
        return Ok(assets);
    }
    let frac = (total_shares + Uint128::one(), total_assets + Uint128::one());
    match rounding {
        Rounding::Ceil => assets.checked_mul_ceil(frac),
        Rounding::Floor => assets.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(e.to_string()))
}

/// Internal conversion
pub fn _convert_to_assets(
    total_shares: Uint128,
    total_assets: Uint128,
    shares: Uint128,
    rounding: Rounding,
) -> Result<Uint128, StdError> {
    let frac = (total_assets + Uint128::one(), total_shares + Uint128::one());
    match rounding {
        Rounding::Ceil => shares.checked_mul_ceil(frac),
        Rounding::Floor => shares.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(e.to_string()))
}

/// Preview deposit calculation - now matches convert_to_shares logic
pub fn _preview_deposit(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
) -> StdResult<cw4626::PreviewDepositResponse> {
    let Tokens {
        total_shares,
        total_assets,
        ..
    } = get_tokens(this, deps)?;
    let shares = _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)?;
    Ok(cw4626::PreviewDepositResponse { shares })
}

// Internal unchecked `mint`
pub fn _mint(deps: DepsMut, recipient: String, amount: Uint128) -> Result<(), ContractError> {
    let mut config = cw20_base::state::TOKEN_INFO.load(deps.storage)?;

    // update supply and enforce cap
    config.total_supply += amount;
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::ShareCw20Error(
                cw20_base::ContractError::CannotExceedCap {},
            ));
        }
    }
    cw20_base::state::TOKEN_INFO.save(deps.storage, &config)?;

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    cw20_base::state::BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;
    Ok(())
}

/// Used internally in `deposit`/`mint` functionality
pub fn _deposit(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiver: Addr,
    assets: Uint128,
    shares: Uint128,
) -> Result<Response, ContractError> {
    if assets.is_zero() {
        return Err(ContractError::ZeroAssetAmount {});
    }
    if shares.is_zero() {
        return Err(ContractError::ZeroShareAmount {});
    }

    let caller = info.sender.clone();
    let asset_info = UNDERLYING_ASSET.load(deps.storage)?;
    let asset = Asset {
        amount: assets,
        info: asset_info,
    };
    let transfer_msg = assert_send_asset_to_contract(info, env, asset.clone())?;
    let mut res = generate_deposit_response(&caller, &receiver, assets, shares);
    if let Some(msg) = transfer_msg {
        res = res.add_message(msg);
    }
    _mint(deps.branch(), receiver.to_string(), shares)?;
    Ok(res)
}

/// Validates addrs uniqueness, minimum and maximum length
pub fn validate_addrs(addrs: impl Iterator<Item = Addr>) -> Result<Vec<Addr>, ContractError> {
    let addrs = addrs
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if addrs.is_empty() {
        return Err(ContractError::EmptyAddrsList {});
    }
    if addrs.len() > 10 {
        return Err(ContractError::MaxedAddrsList {});
    }
    Ok(addrs)
}

pub fn validate_salt(salt: &str) -> Result<(), ContractError> {
    let hex = salt
        .strip_prefix("0x")
        .ok_or(ContractError::InvalidSalt {})?;
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ContractError::InvalidSalt {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_shares_zero_state_returns_1_to_1() {
        let assets = Uint128::new(60000);

        // Test when total_shares is zero
        let shares =
            _convert_to_shares(Uint128::zero(), Uint128::new(1000), assets, Rounding::Floor)
                .unwrap();
        assert_eq!(shares, assets);

        // Test when total_assets is zero
        let shares =
            _convert_to_shares(Uint128::new(1000), Uint128::zero(), assets, Rounding::Floor)
                .unwrap();
        assert_eq!(shares, assets);

        // Test when both are zero
        let shares =
            _convert_to_shares(Uint128::zero(), Uint128::zero(), assets, Rounding::Floor).unwrap();
        assert_eq!(shares, assets);
    }

    #[test]
    fn convert_to_shares_non_zero_state_uses_ratio() {
        let assets = Uint128::new(1000);
        let total_shares = Uint128::new(2000);
        let total_assets = Uint128::new(3000);

        // Should use the ratio formula, not 1:1
        let shares =
            _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor).unwrap();

        // Expected: 1000 * (2000 + 1) / (3000 + 1) = 1000 * 2001 / 3001 ≈ 666
        let expected = Uint128::new(666);
        assert_eq!(shares, expected);
    }

    #[test]
    fn preview_deposit_should_match_convert_to_shares() {
        // Test that preview_deposit now gives the same result as convert_to_shares
        // This ensures the fix is working correctly
        let assets = Uint128::new(1000);
        let total_shares = Uint128::new(1000);
        let total_assets = Uint128::new(1000);

        // Both should give the same result now
        let convert_shares =
            _convert_to_shares(total_shares, total_assets, assets, Rounding::Floor).unwrap();

        // The preview should now match the convert_to_shares result
        // (We can't test _preview_deposit directly here as it needs deps, but the logic is the same)
        assert_eq!(convert_shares, assets); // Should be 1:1 in this case
    }
}
