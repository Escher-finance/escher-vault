use std::collections::HashMap;

use astroport::{
    asset::{Asset, AssetInfo, PairInfo},
    incentives::{
        Config as IncentivesConfig, ExecuteMsg as IncentivesExecuteMsg,
        QueryMsg as IncentivesQueryMsg,
    },
    pair::{Cw20HookMsg, ExecuteMsg as PairExecuteMsg},
    pair_concentrated::QueryMsg as PairConcentratedQueryMsg,
};
use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, Decimal, DepsMut, Storage, Uint128, WasmMsg};
use cw4626::cw20::Cw20ExecuteMsg;

use crate::{
    state::{TowerConfig, ORACLE_PRICES, TOWER_CONFIG},
    ContractError,
};

pub fn update_tower_config(
    deps: DepsMut,
    tower_incentives: Addr,
    lp: Addr,
    slippage_tolerance: Decimal,
    lp_incentives: Vec<AssetInfo>,
    underlying_asset: Addr,
) -> Result<TowerConfig, ContractError> {
    let invalid_tower_config_err = Err(ContractError::InvalidTowerConfig {});
    if deps
        .querier
        .query_wasm_smart::<IncentivesConfig>(&tower_incentives, &IncentivesQueryMsg::Config {})
        .is_err()
    {
        return invalid_tower_config_err;
    }
    if slippage_tolerance.is_zero() {
        return invalid_tower_config_err;
    }
    let pair_info: PairInfo = deps
        .querier
        .query_wasm_smart(lp.clone(), &PairConcentratedQueryMsg::Pair {})?;
    let lp_underlying_asset = AssetInfo::Token {
        contract_addr: underlying_asset,
    };
    if pair_info.asset_infos.len() != 2 || !pair_info.asset_infos.contains(&lp_underlying_asset) {
        return invalid_tower_config_err;
    }
    let Some(lp_other_asset) = pair_info
        .asset_infos
        .iter()
        .find(|info| **info != lp_underlying_asset)
    else {
        return invalid_tower_config_err;
    };
    if lp_incentives.is_empty()
        || lp_incentives
            .iter()
            .any(|i| pair_info.asset_infos.contains(i))
    {
        return invalid_tower_config_err;
    }
    let config = TowerConfig {
        tower_incentives,
        lp: lp.clone(),
        lp_underlying_asset,
        lp_other_asset: lp_other_asset.clone(),
        lp_token: deps.api.addr_validate(&pair_info.liquidity_token)?,
        lp_incentives,
        slippage_tolerance,
    };
    TOWER_CONFIG.save(deps.storage, &config)?;
    Ok(config)
}

pub fn init_oracle_prices(deps: DepsMut, tower_config: &TowerConfig) -> Result<(), ContractError> {
    let mut assets = Vec::from([
        tower_config.lp_underlying_asset.clone(),
        tower_config.lp_other_asset.clone(),
    ]);
    assets.extend(tower_config.lp_incentives.clone());
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

pub fn update_and_validate_prices(
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

pub fn add_liquidity(
    storage: &dyn Storage,
    underlying_asset_amount: Uint128,
    other_lp_asset_amount: Uint128,
) -> Result<CosmosMsg, ContractError> {
    let tower_config = TOWER_CONFIG.load(storage)?;
    let assets = Vec::from([
        Asset {
            info: tower_config.lp_underlying_asset,
            amount: underlying_asset_amount,
        },
        Asset {
            info: tower_config.lp_other_asset,
            amount: other_lp_asset_amount,
        },
    ]);
    let execute_msg = PairExecuteMsg::ProvideLiquidity {
        assets,
        auto_stake: Some(true),
        slippage_tolerance: Some(tower_config.slippage_tolerance),
        receiver: None,
        min_lp_to_receive: None,
    };
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: tower_config.lp.to_string(),
        msg: to_json_binary(&execute_msg)?,
        funds: vec![],
    }))
}

pub fn withdraw_liquidity(
    storage: &dyn Storage,
    lp_token_amount: Uint128,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let tower_config = TOWER_CONFIG.load(storage)?;
    let incentives_execute_msg = IncentivesExecuteMsg::Withdraw {
        lp_token: tower_config.lp_token.to_string(),
        amount: lp_token_amount,
    };
    let lp_token_execute_msg = Cw20ExecuteMsg::Send {
        contract: tower_config.lp.to_string(),
        amount: lp_token_amount,
        msg: to_json_binary(&Cw20HookMsg::WithdrawLiquidity {
            min_assets_to_receive: None,
        })?,
    };
    Ok(Vec::from([
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: tower_config.tower_incentives.to_string(),
            msg: to_json_binary(&incentives_execute_msg)?,
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: tower_config.lp_token.to_string(),
            msg: to_json_binary(&lp_token_execute_msg)?,
            funds: vec![],
        }),
    ]))
}
