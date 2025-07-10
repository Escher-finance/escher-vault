use cosmwasm_std::{Addr, BlockInfo, DepsMut, Env, Response, Uint128};
use cw4626::{
    MaxDepositResponse, MaxMintResponse, MaxRedeemResponse, MaxWithdrawResponse,
    PreviewDepositResponse, PreviewMintResponse, PreviewRedeemResponse, PreviewWithdrawResponse,
};

use crate::{
    helpers::{_deposit, _withdraw},
    query, ContractError,
};

pub fn deposit(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    assets: Uint128,
    receiver: Addr,
) -> Result<Response, ContractError> {
    let MaxDepositResponse { max_assets } = query::max_deposit(receiver.clone())?;
    if assets > max_assets {
        return Err(ContractError::ExceededMaxDeposit {
            receiver: receiver.to_string(),
            assets: assets.u128(),
            max_assets: max_assets.u128(),
        });
    }
    let PreviewDepositResponse { shares } =
        query::preview_deposit(&env.contract.address, &deps.as_ref(), assets)?;
    _deposit(deps, env, sender, receiver, assets, shares)
}

pub fn mint(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    shares: Uint128,
    receiver: Addr,
) -> Result<Response, ContractError> {
    let deps_ref = deps.as_ref();
    let MaxMintResponse { max_shares } = query::max_mint(receiver.clone())?;
    if shares > max_shares {
        return Err(ContractError::ExceededMaxMint {
            receiver: receiver.to_string(),
            shares: shares.u128(),
            max_shares: max_shares.u128(),
        });
    }
    let PreviewMintResponse { assets } =
        query::preview_mint(&env.contract.address, &deps_ref, shares)?;
    _deposit(deps, env, sender, receiver, assets, shares)
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    assets: Uint128,
    receiver: Addr,
    owner: Addr,
) -> Result<Response, ContractError> {
    let this = env.contract.address.clone();
    let MaxWithdrawResponse { max_assets } =
        query::max_withdraw(&this, &deps.as_ref(), owner.clone())?;
    if assets > max_assets {
        return Err(ContractError::ExceededMaxWithdraw {
            owner: owner.to_string(),
            assets: assets.u128(),
            max_assets: max_assets.u128(),
        });
    }
    let PreviewWithdrawResponse { shares } =
        query::preview_withdraw(&this, &deps.as_ref(), assets)?;
    _withdraw(deps, env, sender, receiver, owner, assets, shares)
}

pub fn redeem(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    shares: Uint128,
    receiver: Addr,
    owner: Addr,
) -> Result<Response, ContractError> {
    let MaxRedeemResponse { max_shares } = query::max_redeem(&deps.as_ref(), owner.clone())?;
    if shares > max_shares {
        return Err(ContractError::ExceededMaxRedeem {
            owner: owner.to_string(),
            shares: shares.u128(),
            max_shares: max_shares.u128(),
        });
    }
    let PreviewRedeemResponse { assets } =
        query::preview_redeem(&env.contract.address, &deps.as_ref(), shares)?;
    _withdraw(deps, env, sender, receiver, owner, assets, shares)
}

pub fn update_ownership(
    deps: DepsMut,
    block: BlockInfo,
    new_owner: Addr,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    cw_ownable::update_ownership(deps, &block, &new_owner, action)?;
    Ok(Response::new())
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Addr, Uint128,
    };

    use super::*;

    #[test]
    fn deposit_must_not_exceed_max_assets() {
        let mut deps = mock_dependencies();
        let deps_mut = deps.as_mut();
        let env = mock_env();
        let sender = Addr::unchecked("sender");
        let receiver = Addr::unchecked("receiver");
        let MaxDepositResponse { max_assets } = query::max_deposit(receiver.clone()).unwrap();
        let amount = max_assets + Uint128::one();
        assert_eq!(
            deposit(deps_mut, env, sender, amount, receiver.clone()).unwrap_err(),
            ContractError::ExceededMaxDeposit {
                receiver: receiver.to_string(),
                max_assets: max_assets.u128(),
                assets: amount.u128()
            },
            "must error with exceeded max deposit"
        );
    }

    #[test]
    fn mint_must_not_exceed_max_shares() {
        let mut deps = mock_dependencies();
        let deps_mut = deps.as_mut();
        let env = mock_env();
        let sender = Addr::unchecked("sender");
        let receiver = Addr::unchecked("receiver");
        let MaxMintResponse { max_shares } = query::max_mint(receiver.clone()).unwrap();
        let amount = max_shares + Uint128::one();
        assert_eq!(
            mint(deps_mut, env, sender, amount, receiver.clone()).unwrap_err(),
            ContractError::ExceededMaxMint {
                receiver: receiver.to_string(),
                max_shares: max_shares.u128(),
                shares: amount.u128()
            },
            "must error with exceeded max mint"
        );
    }
}
