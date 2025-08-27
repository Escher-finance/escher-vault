use astroport::{
    asset::{AssetInfo, PairInfo},
    pair_concentrated::QueryMsg as PairConcentratedQueryMsg,
};
use cosmwasm_std::{Addr, Decimal, DepsMut};

use crate::{state::{TowerConfig, TOWER_CONFIG}, ContractError};

pub fn update_tower_config(
    deps: DepsMut,
    lp: Addr,
    slippage_tolerance: Decimal,
    incentives: Vec<AssetInfo>,
    underlying_asset: Addr,
) -> Result<(), ContractError> {
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
    if pair_info.asset_infos.len() != 2
        || pair_info
            .asset_infos
            .iter()
            .find(|a| **a == underlying_asset)
            .is_none()
    {
        return invalid_tower_config_err;
    }
    if incentives.is_empty() {
        return invalid_tower_config_err;
    }
    TOWER_CONFIG.save(
        deps.storage,
        &TowerConfig {
            lp: lp.clone(),
            lp_assets: [
                pair_info.asset_infos[0].clone(),
                pair_info.asset_infos[1].clone(),
            ],
            lp_token: deps.api.addr_validate(&pair_info.liquidity_token)?,
            incentives,
            slippage_tolerance: slippage_tolerance,
        },
    )?;
    Ok(())
}
