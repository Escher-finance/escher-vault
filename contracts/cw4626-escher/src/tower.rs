use std::collections::HashMap;

use astroport::{
    asset::{Asset, AssetInfo, PairInfo},
    incentives::{
        Config as IncentivesConfig, ExecuteMsg as IncentivesExecuteMsg,
        QueryMsg as IncentivesQueryMsg,
    },
    pair::{Cw20HookMsg, ExecuteMsg as PairExecuteMsg},
    pair_concentrated::QueryMsg as PairConcentratedQueryMsg,
    querier::{query_balance, query_token_balance},
};
use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, Decimal, DepsMut, QuerierWrapper, Storage, Uint128, WasmMsg,
};
use cw4626::cw20::{self, Cw20ExecuteMsg};
use cw4626_base::helpers::validate_cw20;

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
    underlying_asset_info: AssetInfo,
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
    if pair_info.asset_infos.len() != 2 || !pair_info.asset_infos.contains(&underlying_asset_info) {
        return invalid_tower_config_err;
    }
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

pub fn get_asset_info_address(asset_info: &AssetInfo) -> String {
    match asset_info {
        AssetInfo::NativeToken { denom } => denom.clone(),
        AssetInfo::Token { contract_addr } => contract_addr.to_string(),
    }
}

pub fn query_asset_info_balance(
    querier: &QuerierWrapper,
    asset_info: AssetInfo,
    addr: Addr,
) -> Result<Uint128, cosmwasm_std::StdError> {
    match asset_info {
        AssetInfo::Token { contract_addr, .. } => query_token_balance(querier, contract_addr, addr),
        AssetInfo::NativeToken { denom } => query_balance(querier, addr, denom),
    }
}

pub fn query_asset_info_decimals(
    querier: &QuerierWrapper,
    asset_info: AssetInfo,
) -> Result<u8, ContractError> {
    match asset_info {
        AssetInfo::Token { contract_addr, .. } => {
            let cw20::TokenInfoResponse { decimals, .. } = validate_cw20(&querier, &contract_addr)?;
            Ok(decimals)
        }
        AssetInfo::NativeToken { .. } => Ok(6),
    }
}

pub fn calculate_total_assets(
    querier: &QuerierWrapper,
    storage: &dyn Storage,
    addr: Addr,
) -> Result<Uint128, ContractError> {
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
