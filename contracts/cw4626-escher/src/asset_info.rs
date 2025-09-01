use astroport::{
    asset::AssetInfo,
    querier::{query_balance, query_token_balance},
};
use cosmwasm_std::{Addr, QuerierWrapper, Uint128};
use cw4626::cw20;
use cw4626_base::helpers::validate_cw20;

use crate::ContractError;

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
            let cw20::TokenInfoResponse { decimals, .. } = validate_cw20(querier, &contract_addr)?;
            Ok(decimals)
        }
        AssetInfo::NativeToken { .. } => Ok(6),
    }
}
