use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response};

use crate::{
    ContractError,
    error::ContractResult,
    helpers::{PreviewDepositKind, internal_deposit},
    msg::{MaxDepositResponse, PreviewDepositResponse},
    query,
    state::UNDERLYING_ASSET,
};

/// # Errors
/// Will return error if internal helper fails
pub fn receive_deposit(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    sender: &Addr,
    received_balance: &cw20::Cw20CoinVerified,
    receiver: &Addr,
) -> ContractResult<Response> {
    if received_balance.address.to_string() != UNDERLYING_ASSET.load(deps.storage)?.to_string() {
        return Err(ContractError::WrongCw20Received {});
    }
    let assets = received_balance.amount;
    let MaxDepositResponse { max_assets } = query::max_deposit(receiver.clone())?;
    if assets > max_assets {
        return Err(ContractError::ExceededMaxDeposit {
            receiver: receiver.clone(),
            assets,
            max_assets,
        });
    }
    let PreviewDepositResponse { shares } = query::preview_deposit(
        &env.contract.address,
        &deps.as_ref(),
        assets,
        PreviewDepositKind::Cw20ViaReceive {},
    )?;
    internal_deposit(deps, env, info, sender, receiver, assets, shares, true)
}
