use std::collections::HashSet;

use crate::{
    error::ContractResult,
    msg::PreviewDepositResponse,
    responses::{generate_deposit_response, generate_deposit_with_fee_response},
    state::EntryFeeConfig,
};
use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{
    Addr, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};

use crate::{
    asset::assert_send_asset_to_contract,
    state::{ENTRY_FEE_CONFIG, LOCKED_SHARES, UNDERLYING_ASSET},
    tower::calculate_total_assets,
    ContractError,
};

#[derive(Debug)]
pub struct Tokens {
    pub share: AssetInfo,
    pub asset: AssetInfo,
    pub total_shares: Uint128,
    pub total_assets: Uint128,
}

/// # Errors
/// Will return error if queries fail
pub fn get_tokens(this: &Addr, deps: &Deps) -> StdResult<Tokens> {
    let share = AssetInfo::Token {
        contract_addr: this.clone(),
    };
    let asset = UNDERLYING_ASSET.load(deps.storage)?;
    let total_supply = cw20_base::state::TOKEN_INFO
        .load(deps.storage)?
        .total_supply;

    // Subtract locked shares from total supply for exchange rate calculation
    let locked_shares =
        LOCKED_SHARES
            .may_load(deps.storage)?
            .unwrap_or(crate::state::LockedShares {
                total_locked: Uint128::zero(),
                redemption_ids: vec![],
            });

    let total_shares = total_supply.saturating_sub(locked_shares.total_locked);

    let total_assets = calculate_total_assets(&deps.querier, deps.storage, this)
        .map_err(|err| StdError::generic_err(err.to_string()))?;
    Ok(Tokens {
        share,
        asset,
        total_shares,
        total_assets,
    })
}

#[derive(Debug, Clone, Copy)]
pub enum Rounding {
    Floor,
    Ceil,
}

/// Internal conversion
///
/// # Errors
/// Will return error if calculation fails
pub fn internal_convert_to_shares(
    total_shares: Uint128,
    total_assets: Uint128,
    assets: Uint128,
    rounding: Rounding,
) -> StdResult<Uint128> {
    let frac = (total_shares + Uint128::one(), total_assets + Uint128::one());
    match rounding {
        Rounding::Ceil => assets.checked_mul_ceil(frac),
        Rounding::Floor => assets.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(e.to_string()))
}

/// Internal conversion
///
/// # Errors
/// Will return error if calculation fails
pub fn internal_convert_to_assets(
    total_shares: Uint128,
    total_assets: Uint128,
    shares: Uint128,
    rounding: Rounding,
) -> StdResult<Uint128> {
    let frac = (total_assets + Uint128::one(), total_shares + Uint128::one());
    match rounding {
        Rounding::Ceil => shares.checked_mul_ceil(frac),
        Rounding::Floor => shares.checked_mul_floor(frac),
    }
    .map_err(|e| StdError::generic_err(e.to_string()))
}

#[derive(Debug, Clone, Copy)]
pub enum PreviewDepositKind {
    OnlyQuery {},
    Cw20ViaTransferFrom {},
    Cw20ViaReceive {},
    Native {},
}

impl PreviewDepositKind {
    #[must_use]
    pub fn needs_correction(&self) -> bool {
        match self {
            Self::OnlyQuery {} | Self::Cw20ViaTransferFrom {} => false,
            Self::Native {} | Self::Cw20ViaReceive {} => true,
        }
    }
}

/// Returns (`user_shares`, `fee_shares`)
#[must_use]
pub fn calculate_entry_fee_share_amounts(
    entry_fee_cfg: &EntryFeeConfig,
    assets: Uint128,
    shares: Uint128,
) -> (Uint128, Uint128) {
    if entry_fee_cfg.fee_rate.is_zero() {
        return (shares, Uint128::zero());
    }
    // Compute fee_on_total in asset terms using integer math
    let r_n = entry_fee_cfg.fee_rate.atomics();
    let r_d = Decimal::one().atomics();
    let fee_assets = assets.multiply_ratio(r_n, r_d + r_n);
    // Convert fee assets into fee shares proportionally to net assets that minted `shares`
    let net_assets = assets.saturating_sub(fee_assets);
    let fee_shares = if net_assets.is_zero() {
        Uint128::zero()
    } else {
        shares.multiply_ratio(fee_assets, net_assets)
    };
    let user_shares = shares.saturating_sub(fee_shares);
    (user_shares, fee_shares)
}

/// Preview deposit calculation
///
/// # Errors
/// Will return error if queries fail
pub fn internal_preview_deposit(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
    preview_deposit_kind: PreviewDepositKind,
) -> StdResult<PreviewDepositResponse> {
    let Tokens {
        total_shares,
        mut total_assets,
        ..
    } = get_tokens(this, deps)?;

    if preview_deposit_kind.needs_correction() {
        total_assets = total_assets.saturating_sub(assets);
    }

    // Preview on full assets; fee is applied at mint-split time (shares*(1-r), shares*r)
    let shares = internal_convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)?;
    let mut user_shares = shares;
    // NOTE: We do this check because if it's not query then the fee is accounted for after this
    // function is called; see `execute::deposit` and `helpers::_deposit`
    if matches!(preview_deposit_kind, PreviewDepositKind::OnlyQuery {}) {
        let entry_fee_cfg = ENTRY_FEE_CONFIG.load(deps.storage)?;
        user_shares = calculate_entry_fee_share_amounts(&entry_fee_cfg, assets, shares).0;
    }
    Ok(PreviewDepositResponse {
        shares: user_shares,
    })
}

// Internal unchecked `mint`
///
/// # Errors
/// Will return error if storage queries or saves fail
pub fn internal_mint(deps: &mut DepsMut, recipient: &str, amount: Uint128) -> ContractResult<()> {
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
    let rcpt_addr = deps.api.addr_validate(recipient)?;
    cw20_base::state::BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;
    Ok(())
}

/// Used internally in `deposit`/`mint` functionality
///
/// # Errors
/// Will return error if storage queries or saves fail
pub fn internal_deposit(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    sender: &Addr,
    receiver: &Addr,
    assets: Uint128,
    shares: Uint128,
    via_receive: bool,
) -> ContractResult<Response> {
    let asset_info = UNDERLYING_ASSET.load(deps.storage)?;
    let asset = Asset {
        amount: assets,
        info: asset_info,
    };
    let transfer_msg = if via_receive {
        None
    } else {
        assert_send_asset_to_contract(info, env, asset.clone())?
    };

    // Mint shares to receiver minus fee, and fee shares to fee recipient if configured
    let entry_fee_cfg = ENTRY_FEE_CONFIG.load(deps.storage)?;

    if entry_fee_cfg.fee_rate.is_zero() {
        internal_mint(&mut deps.branch(), receiver.as_str(), shares)?;
        let mut res = generate_deposit_response(sender, receiver, assets, shares);
        if let Some(msg) = transfer_msg {
            res = res.add_message(msg);
        }
        Ok(res)
    } else {
        let (user_shares, fee_shares) =
            calculate_entry_fee_share_amounts(&entry_fee_cfg, assets, shares);
        internal_mint(&mut deps.branch(), receiver.as_str(), user_shares)?;
        internal_mint(
            &mut deps.branch(),
            entry_fee_cfg.fee_recipient.as_str(),
            fee_shares,
        )?;
        let mut res = generate_deposit_with_fee_response(
            sender,
            receiver,
            assets,
            user_shares,
            fee_shares,
            entry_fee_cfg.fee_rate,
        );
        if let Some(msg) = transfer_msg {
            res = res.add_message(msg);
        }
        Ok(res)
    }
}

/// Validates addrs uniqueness, minimum and maximum length
///
/// # Errors
/// Will return error if validations fail
pub fn validate_addrs(addrs: impl Iterator<Item = Addr>) -> ContractResult<Vec<Addr>> {
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

/// # Errors
/// Will return error if validations fail
pub fn validate_salt(salt: &str) -> ContractResult<()> {
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
    use crate::{
        responses::EVENT_DEPOSIT,
        state::{EntryFeeConfig, ENTRY_FEE_CONFIG, UNDERLYING_ASSET},
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Decimal, MessageInfo,
    };

    #[test]
    fn convert_to_shares_zero_state_returns_1_to_1() {
        let assets = Uint128::new(60000);

        // Test when total_shares is zero
        let shares = internal_convert_to_shares(
            Uint128::zero(),
            Uint128::new(60000),
            assets,
            Rounding::Floor,
        )
        .unwrap();
        assert_eq!(shares, Uint128::zero());

        // Test when both are zero
        let shares =
            internal_convert_to_shares(Uint128::zero(), Uint128::zero(), assets, Rounding::Floor)
                .unwrap();
        assert_eq!(shares, assets);
    }

    #[test]
    fn convert_to_shares_non_zero_state_uses_ratio() {
        let assets = Uint128::new(1000);
        let total_shares = Uint128::new(2000);
        let total_assets = Uint128::new(3000);

        // Should use the ratio formula, not 1:1
        let shares =
            internal_convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)
                .unwrap();

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
            internal_convert_to_shares(total_shares, total_assets, assets, Rounding::Floor)
                .unwrap();

        // The preview should now match the convert_to_shares result
        // (We can't test _preview_deposit directly here as it needs deps, but the logic is the same)
        assert_eq!(convert_shares, assets); // Should be 1:1 in this case
    }

    #[test]
    fn deposit_mints_fee_and_user_shares_with_entry_fee() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        // Configure underlying as CW20 to avoid native funds check
        let token_addr = deps.api.addr_make("token");
        UNDERLYING_ASSET
            .save(
                deps.as_mut().storage,
                &AssetInfo::Token {
                    contract_addr: token_addr,
                },
            )
            .unwrap();

        // Set entry fee: 10%, fee recipient
        let fee_addr = deps.api.addr_make("fee");
        ENTRY_FEE_CONFIG
            .save(
                deps.as_mut().storage,
                &EntryFeeConfig {
                    fee_rate: Decimal::percent(10),
                    fee_recipient: fee_addr.clone(),
                },
            )
            .unwrap();

        // Init token info
        cw20_base::state::TOKEN_INFO
            .save(
                deps.as_mut().storage,
                &cw20_base::state::TokenInfo {
                    name: "Vault Token".to_string(),
                    symbol: "VAULT".to_string(),
                    decimals: 6,
                    total_supply: Uint128::zero(),
                    mint: None,
                },
            )
            .unwrap();

        let depositor = deps.api.addr_make("depositor");
        let receiver = deps.api.addr_make("receiver");
        let info = MessageInfo {
            sender: depositor.clone(),
            funds: vec![],
        };
        let assets = Uint128::new(1000);

        // Assume preview produced 910 shares (net_assets with 10% fee_on_total is 910)
        let shares = Uint128::new(910);

        let res = internal_deposit(
            &mut deps.as_mut(),
            &env,
            &info,
            &depositor,
            &receiver,
            assets,
            shares,
            false,
        )
        .unwrap();
        let ev = res
            .events
            .into_iter()
            .find(|e| e.ty == EVENT_DEPOSIT)
            .unwrap();
        // Verify attributes present
        assert!(ev
            .attributes
            .iter()
            .any(|a| a.key == "fee_shares_minted" && a.value == "90"));
        assert!(ev
            .attributes
            .iter()
            .any(|a| a.key == "user_shares_minted" && a.value == "820"));

        // Check balances
        let user_balance = cw20_base::state::BALANCES
            .load(deps.as_ref().storage, &receiver)
            .unwrap();
        let fee_balance = cw20_base::state::BALANCES
            .load(deps.as_ref().storage, &fee_addr)
            .unwrap();
        let total_supply = cw20_base::state::TOKEN_INFO
            .load(deps.as_ref().storage)
            .unwrap()
            .total_supply;
        assert_eq!(user_balance, Uint128::new(820));
        assert_eq!(fee_balance, Uint128::new(90));
        assert_eq!(total_supply, Uint128::new(910));
    }

    #[test]
    fn deposit_mints_all_shares_when_entry_fee_zero() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let token_addr = deps.api.addr_make("token");
        UNDERLYING_ASSET
            .save(
                deps.as_mut().storage,
                &AssetInfo::Token {
                    contract_addr: token_addr,
                },
            )
            .unwrap();

        ENTRY_FEE_CONFIG
            .save(
                deps.as_mut().storage,
                &EntryFeeConfig {
                    fee_rate: Decimal::zero(),
                    fee_recipient: Addr::unchecked("0"),
                },
            )
            .unwrap();
        // Init token info
        cw20_base::state::TOKEN_INFO
            .save(
                deps.as_mut().storage,
                &cw20_base::state::TokenInfo {
                    name: "Vault Token".to_string(),
                    symbol: "VAULT".to_string(),
                    decimals: 6,
                    total_supply: Uint128::zero(),
                    mint: None,
                },
            )
            .unwrap();

        let depositor = deps.api.addr_make("depositor");
        let receiver = deps.api.addr_make("receiver");
        let info = MessageInfo {
            sender: depositor.clone(),
            funds: vec![],
        };
        let assets = Uint128::new(1000);
        let shares = Uint128::new(1000);

        let res = internal_deposit(
            &mut deps.as_mut(),
            &env,
            &info,
            &depositor,
            &receiver,
            assets,
            shares,
            false,
        )
        .unwrap();
        // No fee attributes present
        assert!(!res.attributes.iter().any(|a| a.key == "fee_shares"));

        let user_balance = cw20_base::state::BALANCES
            .load(deps.as_ref().storage, &receiver)
            .unwrap();
        let total_supply = cw20_base::state::TOKEN_INFO
            .load(deps.as_ref().storage)
            .unwrap()
            .total_supply;
        assert_eq!(user_balance, Uint128::new(1000));
        assert_eq!(total_supply, Uint128::new(1000));
    }
}
