use astroport::{
    asset::{Asset, AssetInfo},
    querier::{query_balance, query_token_balance},
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Env, MessageInfo, QuerierWrapper, StdResult, Uint128, WasmMsg,
};
use cw4626::cw20;
use cw4626_base::helpers::validate_cw20;
use cw_utils::must_pay;

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

/// Only returns `WasmMsg` if `AssetInfo::Token`
pub fn assert_send_asset_to_contract(
    info: MessageInfo,
    env: Env,
    asset: Asset,
) -> Result<Option<WasmMsg>, ContractError> {
    let caller = info.sender.clone();
    let this = env.contract.address;
    match asset.info {
        AssetInfo::Token { contract_addr } => Ok(Some(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
                owner: caller.to_string(),
                recipient: this.to_string(),
                amount: asset.amount,
            })?,
            funds: vec![],
        })),
        AssetInfo::NativeToken { denom } => {
            if must_pay(&info, &denom)? < asset.amount {
                return Err(ContractError::InsufficientFunds {});
            }
            Ok(None)
        }
    }
}

/// If `AssetInfo::NativeToken` it attaches funds to msg
/// If `AssetInfo::Token` it uses cw20 Send
pub fn asset_cw20_send_or_attach_funds(
    asset: Asset,
    execute_contract_addr: Addr,
    msg: Binary,
) -> StdResult<WasmMsg> {
    let wasm_msg = match asset.info {
        AssetInfo::Token { contract_addr } => WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Send {
                contract: execute_contract_addr.to_string(),
                amount: asset.amount,
                msg,
            })?,
            funds: vec![],
        },
        AssetInfo::NativeToken { .. } => WasmMsg::Execute {
            contract_addr: execute_contract_addr.to_string(),
            msg,
            funds: Vec::from([asset.as_coin()?]),
        },
    };
    Ok(wasm_msg)
}
