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
use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, Decimal, DepsMut, QuerierWrapper, Storage, Uint128, WasmMsg, StdError, StdResult,
};
use cw4626::cw20;

use crate::{
    asset_info::{get_asset_info_address, query_asset_info_balance},
    state::{PricesMap, TowerConfig, ORACLE_PRICES, TOWER_CONFIG},
    ContractError,
};

// Security constants for tower validation
const MAX_SLIPPAGE_PERCENT: u64 = 50; // 50%
const MIN_SLIPPAGE_PERCENT: u64 = 1; // 1%
const MAX_LP_INCENTIVES_SIZE: usize = 50;

/// Validates address for security
fn validate_address(addr: &Addr, field_name: &str) -> StdResult<()> {
    if addr.as_str().is_empty() {
        return Err(StdError::generic_err(format!("{} address cannot be empty", field_name)));
    }
    Ok(())
}

/// Validates amount for security
fn validate_amount(amount: Uint128, field_name: &str) -> StdResult<()> {
    if amount.is_zero() {
        return Err(StdError::generic_err(format!("{} amount cannot be zero", field_name)));
    }
    Ok(())
}

/// Validates slippage tolerance for security
fn validate_slippage_tolerance(slippage: Decimal) -> StdResult<()> {
    if slippage < Decimal::percent(MIN_SLIPPAGE_PERCENT) {
        return Err(StdError::generic_err(format!(
            "Slippage tolerance too low (min {}%)", MIN_SLIPPAGE_PERCENT
        )));
    }
    if slippage > Decimal::percent(MAX_SLIPPAGE_PERCENT) {
        return Err(StdError::generic_err(format!(
            "Slippage tolerance too high (max {}%)", MAX_SLIPPAGE_PERCENT
        )));
    }
    Ok(())
}

/// Validates asset info for security
fn validate_asset_info(asset_info: &AssetInfo, field_name: &str) -> StdResult<()> {
    match asset_info {
        AssetInfo::Token { contract_addr } => {
            if contract_addr.as_str().is_empty() {
                return Err(StdError::generic_err(format!("{} contract address cannot be empty", field_name)));
            }
        }
        AssetInfo::NativeToken { denom } => {
            if denom.is_empty() {
                return Err(StdError::generic_err(format!("{} denom cannot be empty", field_name)));
            }
        }
    }
    Ok(())
}

/// Validates LP incentives for security
fn validate_lp_incentives(lp_incentives: &[AssetInfo]) -> StdResult<()> {
    if lp_incentives.is_empty() {
        return Err(StdError::generic_err("LP incentives cannot be empty"));
    }
    if lp_incentives.len() > MAX_LP_INCENTIVES_SIZE {
        return Err(StdError::generic_err(format!(
            "LP incentives too large (max {} entries)", MAX_LP_INCENTIVES_SIZE
        )));
    }
    
    for (i, incentive) in lp_incentives.iter().enumerate() {
        validate_asset_info(incentive, &format!("lp_incentives[{}]", i))?;
    }
    
    Ok(())
}

pub fn update_tower_config(
    deps: DepsMut,
    tower_incentives: Addr,
    lp: Addr,
    slippage_tolerance: Decimal,
    lp_incentives: Vec<AssetInfo>,
    underlying_asset_info: AssetInfo,
) -> Result<TowerConfig, ContractError> {
    // CRITICAL: Validate all input parameters
    validate_address(&tower_incentives, "tower_incentives")?;
    validate_address(&lp, "lp")?;
    validate_slippage_tolerance(slippage_tolerance)?;
    validate_asset_info(&underlying_asset_info, "underlying_asset_info")?;
    validate_lp_incentives(&lp_incentives)?;
    
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
    if pair_info.asset_infos.len() != 2 {
        return invalid_tower_config_err;
    }
    let Some(underlying_asset_position) = pair_info
        .asset_infos
        .iter()
        .position(|a| *a == underlying_asset_info)
    else {
        return invalid_tower_config_err;
    };
    let Some(lp_other_asset) = pair_info
        .asset_infos
        .iter()
        .find(|info| **info != underlying_asset_info)
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
        lp_underlying_asset: underlying_asset_info,
        lp_other_asset: lp_other_asset.clone(),
        is_underlying_first_lp_asset: underlying_asset_position == 0,
        lp_token: deps.api.addr_validate(&pair_info.liquidity_token)?,
        lp_incentives,
        slippage_tolerance,
    };
    TOWER_CONFIG.save(deps.storage, &config)?;
    Ok(config)
}

pub fn init_oracle_prices(deps: DepsMut, tower_config: &TowerConfig) -> Result<(), ContractError> {
    let mut assets = Vec::from([tower_config.lp_other_asset.clone()]);
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

pub fn update_and_validate_prices(deps: DepsMut, prices: PricesMap) -> Result<(), ContractError> {
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

pub fn get_and_validate_oracle_prices(storage: &dyn Storage) -> Result<PricesMap, ContractError> {
    let prices = ORACLE_PRICES.load(storage)?;
    if !prices.values().all(|p| *p > Decimal::zero()) {
        return Err(ContractError::OracleZeroPrice {});
    }
    Ok(prices)
}

pub fn add_tower_liquidity(
    tower_config: &TowerConfig,
    underlying_asset_amount: Uint128,
    other_lp_asset_amount: Uint128,
) -> Result<CosmosMsg, ContractError> {
    // CRITICAL: Validate input parameters
    validate_amount(underlying_asset_amount, "underlying_asset_amount")?;
    validate_amount(other_lp_asset_amount, "other_lp_asset_amount")?;
    let assets = Vec::from([
        Asset {
            info: tower_config.lp_underlying_asset.clone(),
            amount: underlying_asset_amount,
        },
        Asset {
            info: tower_config.lp_other_asset.clone(),
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
    // CRITICAL: Validate input parameters
    validate_amount(lp_token_amount, "lp_token_amount")?;
    let tower_config = TOWER_CONFIG.load(storage)?;
    let incentives_execute_msg = IncentivesExecuteMsg::Withdraw {
        lp_token: tower_config.lp_token.to_string(),
        amount: lp_token_amount,
    };
    let lp_token_execute_msg = cw20::Cw20ExecuteMsg::Send {
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

pub fn calculate_total_assets(
    querier: &QuerierWrapper,
    storage: &dyn Storage,
    addr: Addr,
) -> Result<Uint128, ContractError> {
    // CRITICAL: Validate input parameters
    validate_address(&addr, "addr")?;
    let prices = get_and_validate_oracle_prices(storage)?;
    let tower_config = TOWER_CONFIG.load(storage)?;
    let mut total_balance = query_asset_info_balance(
        querier,
        tower_config.lp_underlying_asset.clone(),
        addr.clone(),
    )?;
    let mut asset_infos = tower_config.lp_incentives.clone();
    asset_infos.push(tower_config.lp_other_asset);
    for asset_info in asset_infos {
        let asset_balance = query_asset_info_balance(querier, asset_info.clone(), addr.clone())?;
        let Some(asset_price) = prices.get(&get_asset_info_address(&asset_info)) else {
            return Err(ContractError::OracleInvalidPrices {});
        };
        total_balance += asset_balance.mul_floor(*asset_price);
    }
    let lp_token_balance = query_asset_info_balance(
        querier,
        AssetInfo::Token {
            contract_addr: tower_config.lp_token.clone(),
        },
        addr.clone(),
    )?;
    if !lp_token_balance.is_zero() {
        let mut assets: Vec<Asset> = querier.query_wasm_smart(
            tower_config.lp,
            &PairConcentratedQueryMsg::SimulateWithdraw {
                lp_amount: lp_token_balance,
            },
        )?;
        assets.extend(
            querier
                .query_wasm_smart::<Vec<Asset>>(
                    tower_config.tower_incentives,
                    &IncentivesQueryMsg::PendingRewards {
                        lp_token: tower_config.lp_token.to_string(),
                        user: addr.to_string(),
                    },
                )?
                .into_iter()
                .filter(|a| tower_config.lp_incentives.contains(&a.info)),
        );
        for asset in assets {
            if asset.info == tower_config.lp_underlying_asset {
                total_balance += asset.amount;
                continue;
            }
            let Some(asset_price) = prices.get(&get_asset_info_address(&asset.info)) else {
                return Err(ContractError::OracleInvalidPrices {});
            };
            total_balance += asset.amount.mul_floor(*asset_price);
        }
    }
    Ok(total_balance)
}
