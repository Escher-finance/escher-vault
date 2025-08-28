use std::collections::HashMap;

use astroport::{
    asset::{AssetInfo, PairInfo},
    pair_concentrated::QueryMsg as PairConcentratedQueryMsg,
};
use cosmwasm_std::{Addr, Decimal, DepsMut, Storage};

use crate::{
    state::{TowerConfig, ORACLE_PRICES, TOWER_CONFIG},
    ContractError,
};

pub fn update_tower_config(
    deps: DepsMut,
    lp: Addr,
    slippage_tolerance: Decimal,
    incentives: Vec<AssetInfo>,
    underlying_asset: Addr,
) -> Result<TowerConfig, ContractError> {
    let invalid_tower_config_err = Err(ContractError::InvalidTowerConfig {});
    if slippage_tolerance.is_zero() {
        return invalid_tower_config_err;
    }
    let pair_info: PairInfo = deps
        .querier
        .query_wasm_smart(lp.clone(), &PairConcentratedQueryMsg::Pair {})?;
    let underlying_asset = AssetInfo::Token {
        contract_addr: underlying_asset,
    };
    if pair_info.asset_infos.len() != 2 || !pair_info.asset_infos.contains(&underlying_asset) {
        return invalid_tower_config_err;
    }
    if incentives.is_empty() || incentives.iter().any(|i| pair_info.asset_infos.contains(i)) {
        return invalid_tower_config_err;
    }
    let config = TowerConfig {
        lp: lp.clone(),
        lp_assets: [
            pair_info.asset_infos[0].clone(),
            pair_info.asset_infos[1].clone(),
        ],
        lp_token: deps.api.addr_validate(&pair_info.liquidity_token)?,
        incentives,
        slippage_tolerance,
    };
    TOWER_CONFIG.save(deps.storage, &config)?;
    Ok(config)
}

pub fn init_oracle_prices(deps: DepsMut, tower_config: &TowerConfig) -> Result<(), ContractError> {
    let mut assets = tower_config.lp_assets.to_vec();
    assets.extend(tower_config.incentives.clone());
    let initial_prices: HashMap<_, _> = assets
        .into_iter()
        .map(|info| {
            let addr = match info {
                AssetInfo::NativeToken { denom } => denom,
                AssetInfo::Token { contract_addr } => contract_addr.to_string(),
            };
            (addr, Decimal::zero())
        })
        .collect();
    ORACLE_PRICES.save(deps.storage, &initial_prices)?;
    Ok(())
}

pub fn update_oracle_prices(
    deps: DepsMut,
    prices: HashMap<String, Decimal>,
) -> Result<(), ContractError> {
    if !prices.values().all(|p| *p > Decimal::zero()) {
        return Err(ContractError::OracleZeroPrice {});
    }
    ORACLE_PRICES.update::<_, ContractError>(deps.storage, |stored_prices| {
        let mut stored_addrs = stored_prices.keys().collect::<Vec<_>>();
        stored_addrs.sort();
        let mut addrs = prices.keys().collect::<Vec<_>>();
        addrs.sort();
        if addrs != stored_addrs {
            return Err(ContractError::OracleInvalidPrices {});
        }
        Ok(prices)
    })?;
    Ok(())
}

pub fn get_and_validate_oracle_prices(
    storage: &dyn Storage,
) -> Result<HashMap<String, Decimal>, ContractError> {
    let prices = ORACLE_PRICES.load(storage)?;
    if !prices.values().all(|p| *p > Decimal::zero()) {
        return Err(ContractError::OracleZeroPrice {});
    }
    Ok(prices)
}
