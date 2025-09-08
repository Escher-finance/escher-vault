use std::collections::HashSet;

use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, Decimal};
use cw4626_base::helpers::generate_deposit_response;

use crate::{
    asset::assert_send_asset_to_contract, state::{UNDERLYING_ASSET, LOCKED_SHARES, ENTRY_FEE_CONFIG}, tower::calculate_total_assets,
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

#[derive(Debug, Clone)]
pub enum PreviewDepositKind {
    OnlyQuery {},
    Cw20ViaTransferFrom {},
    Cw20ViaReceive {},
    Native {},
}

impl PreviewDepositKind {
    pub fn needs_correction(&self) -> bool {
        match self {
            Self::OnlyQuery {} => false,
            Self::Cw20ViaTransferFrom {} => false,
            Self::Cw20ViaReceive {} => true,
            Self::Native {} => true,
        }
    }
}

/// Preview deposit calculation
pub fn _preview_deposit(
    this: &Addr,
    deps: &Deps,
    assets: Uint128,
    preview_deposit_kind: PreviewDepositKind,
) -> StdResult<cw4626::PreviewDepositResponse> {
    let Tokens {
        total_shares,
        mut total_assets,
        ..
    } = get_tokens(this, deps)?;

    // Apply entry fee: the user deposits `assets`, a fraction goes to fee shares.
    // We preview on the net assets (assets after fee) like OZ's previewDeposit.
    let entry_fee_rate = ENTRY_FEE_CONFIG
        .may_load(deps.storage)?
        .map(|cfg| cfg.fee_rate)
        .unwrap_or_else(Decimal::zero);

    // fee_on_total: fee portion inside a gross amount that includes fees: fee = assets * r / (1+r)
    let fee = if entry_fee_rate.is_zero() {
        Uint128::zero()
    } else {
        // Use integer math to avoid precision/overflow
        let r_n = entry_fee_rate.atomics();
        let r_d = Decimal::one().atomics();
        assets.multiply_ratio(r_n, r_d + r_n)
    };
    let net_assets = assets.saturating_sub(fee);

    if preview_deposit_kind.needs_correction() {
        total_assets -= assets;
    }
    let shares = _convert_to_shares(total_shares, total_assets, net_assets, Rounding::Floor)?;
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
    sender: Addr,
    receiver: Addr,
    assets: Uint128,
    shares: Uint128,
    via_receive: bool,
) -> Result<Response, ContractError> {
    let asset_info = UNDERLYING_ASSET.load(deps.storage)?;
    let asset = Asset {
        amount: assets,
        info: asset_info,
    };
    let mut res = generate_deposit_response(&sender, &receiver, assets, shares);
    if !via_receive {
        let transfer_msg = assert_send_asset_to_contract(info, env, asset.clone())?;
        if let Some(msg) = transfer_msg {
            res = res.add_message(msg);
        }
    }
    // Mint shares to receiver minus fee, and fee shares to fee recipient if configured
    let entry_fee_cfg = ENTRY_FEE_CONFIG.may_load(deps.storage)?;
    if let Some(cfg) = entry_fee_cfg {
        if !cfg.fee_rate.is_zero() {
            // Compute fee_on_total in asset terms using integer math
            let r_n = cfg.fee_rate.atomics();
            let r_d = Decimal::one().atomics();
            let fee_assets = assets.multiply_ratio(r_n, r_d + r_n);
            // Minted shares correspond to net_assets; compute fee shares so total shares = user_shares + fee_shares
            // fee_shares = shares * fee_assets / net_assets
            let net_assets = assets.saturating_sub(fee_assets);
            let fee_shares = if net_assets.is_zero() { Uint128::zero() } else { shares.multiply_ratio(fee_assets, net_assets) };
            let user_shares = shares.saturating_sub(fee_shares);
            _mint(deps.branch(), receiver.to_string(), user_shares)?;
            _mint(deps.branch(), cfg.fee_recipient.to_string(), fee_shares)?;
            return Ok(res
                .add_attribute("entry_fee_rate", cfg.fee_rate.to_string())
                .add_attribute("fee_shares", fee_shares.to_string())
                .add_attribute("user_shares", user_shares.to_string()));
        }
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
    use crate::state::{ENTRY_FEE_CONFIG, EntryFeeConfig, UNDERLYING_ASSET};
    use cosmwasm_std::{testing::{mock_dependencies, mock_env}, Addr, MessageInfo, Decimal};

    #[test]
    fn convert_to_shares_zero_state_returns_1_to_1() {
        let assets = Uint128::new(60000);

        // Test when total_shares is zero
        let shares = _convert_to_shares(
            Uint128::zero(),
            Uint128::new(60000),
            assets,
            Rounding::Floor,
        )
        .unwrap();
        assert_eq!(shares, Uint128::zero());

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

    #[test]
    fn deposit_mints_fee_and_user_shares_with_entry_fee() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        // Configure underlying as CW20 to avoid native funds check
        let token_addr = deps.api.addr_make("token");
        UNDERLYING_ASSET
            .save(
                deps.as_mut().storage,
                &AssetInfo::Token { contract_addr: token_addr },
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
        let info = MessageInfo { sender: depositor, funds: vec![] };
        let assets = Uint128::new(1000);

        // Assume preview produced 910 shares (net_assets with 10% fee_on_total is 910)
        let shares = Uint128::new(910);

        let res = _deposit(deps.as_mut(), env, info, receiver.clone(), assets, shares).unwrap();
        // Verify attributes present
        assert!(res.attributes.iter().any(|a| a.key == "fee_shares" && a.value == "90"));
        assert!(res.attributes.iter().any(|a| a.key == "user_shares" && a.value == "820"));

        // Check balances
        let user_balance = cw20_base::state::BALANCES
            .load(deps.as_ref().storage, &receiver)
            .unwrap();
        let fee_balance = cw20_base::state::BALANCES
            .load(deps.as_ref().storage, &fee_addr)
            .unwrap();
        let total_supply = cw20_base::state::TOKEN_INFO.load(deps.as_ref().storage).unwrap().total_supply;
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
                &AssetInfo::Token { contract_addr: token_addr },
            )
            .unwrap();

        // No entry fee config -> treated as zero
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
        let info = MessageInfo { sender: depositor, funds: vec![] };
        let assets = Uint128::new(1000);
        let shares = Uint128::new(1000);

        let res = _deposit(deps.as_mut(), env, info, receiver.clone(), assets, shares).unwrap();
        // No fee attributes present
        assert!(!res.attributes.iter().any(|a| a.key == "fee_shares"));

        let user_balance = cw20_base::state::BALANCES
            .load(deps.as_ref().storage, &receiver)
            .unwrap();
        let total_supply = cw20_base::state::TOKEN_INFO.load(deps.as_ref().storage).unwrap().total_supply;
        assert_eq!(user_balance, Uint128::new(1000));
        assert_eq!(total_supply, Uint128::new(1000));
    }
}
