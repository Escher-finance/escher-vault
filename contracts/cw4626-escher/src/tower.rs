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
    to_json_binary, Addr, CosmosMsg, Decimal, DepsMut, QuerierWrapper, StdResult, Storage, Uint128,
    WasmMsg,
};
use cw4626::cw20;

use crate::{
    asset::{
        asset_cw20_send_or_attach_funds, asset_generate_increase_allowance_or_funds,
        get_asset_info_address, query_asset_info_balance,
    },
    state::{PricesMap, TowerConfig, ORACLE_PRICES, TOWER_CONFIG},
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
) -> Result<Vec<CosmosMsg>, ContractError> {
    let underlying_asset = Asset {
        info: tower_config.lp_underlying_asset.clone(),
        amount: underlying_asset_amount,
    };
    let other_lp_asset = Asset {
        info: tower_config.lp_other_asset.clone(),
        amount: other_lp_asset_amount,
    };

    let mut msgs = Vec::new();
    let mut funds = Vec::new();

    let (underlying_increase_allowance_msg, underlying_coin) =
        asset_generate_increase_allowance_or_funds(
            underlying_asset.clone(),
            tower_config.lp.clone(),
        )?;
    let (other_lp_increase_allowance_msg, other_lp_coin) =
        asset_generate_increase_allowance_or_funds(
            other_lp_asset.clone(),
            tower_config.lp.clone(),
        )?;

    if let Some(m) = underlying_increase_allowance_msg {
        msgs.push(m);
    } else if let Some(c) = underlying_coin {
        funds.push(c);
    }
    if let Some(m) = other_lp_increase_allowance_msg {
        msgs.push(m);
    } else if let Some(c) = other_lp_coin {
        funds.push(c);
    }

    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: tower_config.lp.to_string(),
        msg: to_json_binary(&PairExecuteMsg::ProvideLiquidity {
            assets: Vec::from([underlying_asset, other_lp_asset]),
            auto_stake: Some(true),
            slippage_tolerance: Some(tower_config.slippage_tolerance),
            receiver: None,
            min_lp_to_receive: None,
        })?,
        funds,
    }));

    Ok(msgs)
}

pub fn remove_tower_liquidity(
    tower_config: &TowerConfig,
    lp_token_amount: Uint128,
) -> Result<Vec<CosmosMsg>, ContractError> {
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

pub fn claim_tower_incentives(tower_config: &TowerConfig) -> Result<CosmosMsg, ContractError> {
    let incentives_execute_msg = IncentivesExecuteMsg::ClaimRewards {
        lp_tokens: Vec::from([tower_config.lp_token.to_string()]),
    };
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: tower_config.tower_incentives.to_string(),
        msg: to_json_binary(&incentives_execute_msg)?,
        funds: vec![],
    }))
}

pub fn calculate_total_assets(
    querier: &QuerierWrapper,
    storage: &dyn Storage,
    this: Addr,
) -> Result<Uint128, ContractError> {
    let prices = get_and_validate_oracle_prices(storage)?;
    let tower_config = TOWER_CONFIG.load(storage)?;
    let mut total_balance = query_asset_info_balance(
        querier,
        tower_config.lp_underlying_asset.clone(),
        this.clone(),
    )?;
    let mut asset_infos = tower_config.lp_incentives.clone();
    asset_infos.push(tower_config.lp_other_asset.clone());
    for asset_info in asset_infos {
        let asset_balance = query_asset_info_balance(querier, asset_info.clone(), this.clone())?;
        let Some(asset_price) = prices.get(&get_asset_info_address(&asset_info)) else {
            return Err(ContractError::OracleInvalidPrices {});
        };
        total_balance += asset_balance.mul_floor(*asset_price);
    }
    let mut assets: Vec<Asset> = get_tower_pending_rewards(querier, &tower_config, &this)?
        .into_iter()
        .filter(|a| tower_config.lp_incentives.contains(&a.info))
        .collect();
    let lp_amount = get_tower_lp_token_deposit(querier, &tower_config, &this)?;
    if !lp_amount.is_zero() {
        assets.extend(querier.query_wasm_smart::<Vec<Asset>>(
            tower_config.lp.clone(),
            &PairConcentratedQueryMsg::SimulateWithdraw { lp_amount },
        )?);
    }
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
    Ok(total_balance)
}

// build the swap cosmos messages to the lp contract
pub fn tower_swap(
    tower_config: TowerConfig,
    amount: Uint128,
    asset_info: &AssetInfo,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let offer_asset = Asset {
        info: asset_info.clone(),
        amount,
    };
    let (allowance_msg, fund) =
        asset_generate_increase_allowance_or_funds(offer_asset.clone(), tower_config.lp.clone())?;

    let mut msgs = Vec::new();
    let mut funds = Vec::new();

    if let Some(msg) = allowance_msg {
        msgs.push(msg);
    } else if let Some(coin) = fund {
        funds.push(coin);
    }

    let ask_asset_info = if asset_info == &tower_config.lp_underlying_asset {
        tower_config.lp_other_asset.clone()
    } else {
        tower_config.lp_underlying_asset.clone()
    };

    let msg = {
        let ask_asset_info = Some(ask_asset_info);
        let belief_price = None;
        let max_spread = Some(tower_config.slippage_tolerance);
        let to = None;
        match asset_info {
            AssetInfo::NativeToken { .. } => to_json_binary(&PairExecuteMsg::Swap {
                offer_asset: offer_asset.clone(),
                ask_asset_info,
                belief_price,
                max_spread,
                to,
            }),
            AssetInfo::Token { .. } => to_json_binary(&Cw20HookMsg::Swap {
                ask_asset_info,
                belief_price,
                max_spread,
                to,
            }),
        }?
    };
    msgs.push(CosmosMsg::Wasm(asset_cw20_send_or_attach_funds(
        Asset {
            info: asset_info.clone(),
            amount,
        },
        tower_config.lp.clone(),
        msg,
    )?));
    Ok(msgs)
}

pub fn get_tower_lp_token_deposit(
    querier: &QuerierWrapper,
    tower_config: &TowerConfig,
    addr: &Addr,
) -> StdResult<Uint128> {
    querier.query_wasm_smart::<Uint128>(
        tower_config.tower_incentives.clone(),
        &IncentivesQueryMsg::QueryDeposit {
            lp_token: tower_config.lp_token.to_string(),
            user: addr.to_string(),
        },
    )
}

pub fn get_tower_pending_rewards(
    querier: &QuerierWrapper,
    tower_config: &TowerConfig,
    addr: &Addr,
) -> StdResult<Vec<Asset>> {
    querier.query_wasm_smart::<Vec<Asset>>(
        tower_config.tower_incentives.clone(),
        &IncentivesQueryMsg::PendingRewards {
            lp_token: tower_config.lp_token.to_string(),
            user: addr.to_string(),
        },
    )
}
