use cosmwasm_std::{from_json, Addr, DepsMut, Env, Response, Uint128};
use cw4626::{
    cw20::{Cw20CoinVerified, Cw20ReceiveMsg},
    Cw4626ReceiveMsg, MaxDepositResponse, MaxMintResponse, MaxRedeemResponse, MaxWithdrawResponse,
    PreviewDepositResponse, PreviewMintResponse, PreviewRedeemResponse, PreviewWithdrawResponse,
};

use crate::{
    helpers::{_deposit, _mint, _preview_deposit, _withdraw, generate_deposit_response},
    query,
    state::UNDERLYING_ASSET,
    ContractError,
};

pub fn receive(
    deps: DepsMut,
    env: Env,
    cw20_contract: Addr,
    cw20_receive_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg = from_json::<Cw4626ReceiveMsg>(&cw20_receive_msg.msg)?;
    let sender = deps.api.addr_validate(&cw20_receive_msg.sender)?;
    let received_balance = Cw20CoinVerified {
        address: cw20_contract,
        amount: cw20_receive_msg.amount,
    };
    match msg {
        Cw4626ReceiveMsg::Deposit { receiver } => {
            receive_deposit(deps, env, sender, received_balance, receiver)
        }
    }
}

pub fn receive_deposit(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    received_balance: Cw20CoinVerified,
    receiver: Addr,
) -> Result<Response, ContractError> {
    if received_balance.address != UNDERLYING_ASSET.load(deps.storage)? {
        return Err(ContractError::UnsupportedCw20Received {
            addr: received_balance.address.clone(),
        });
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
    let PreviewDepositResponse { shares } =
        _preview_deposit(&env.contract.address, &deps.as_ref(), assets, true)?;
    _mint(deps, receiver.to_string(), shares)?;
    Ok(generate_deposit_response(
        &sender, &receiver, assets, shares,
    ))
}

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
            receiver: receiver.clone(),
            assets,
            max_assets,
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
            receiver: receiver.clone(),
            shares,
            max_shares,
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
            owner: owner.clone(),
            assets,
            max_assets,
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
            owner: owner.clone(),
            shares,
            max_shares,
        });
    }
    let PreviewRedeemResponse { assets } =
        query::preview_redeem(&env.contract.address, &deps.as_ref(), shares)?;
    _withdraw(deps, env, sender, receiver, owner, assets, shares)
}

pub fn update_ownership(
    deps: DepsMut,
    env: Env,
    new_owner: Addr,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    cw_ownable::update_ownership(deps, &env.block, &new_owner, action)?;
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
                receiver: receiver.clone(),
                max_assets,
                assets: amount
            },
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
                receiver: receiver.clone(),
                max_shares,
                shares: amount
            },
        );
    }

    #[test]
    fn receive_deposit_must_not_exceed_max_assets() {
        let mut deps = mock_dependencies();
        let deps_mut = deps.as_mut();
        let env = mock_env();
        let sender = Addr::unchecked("sender");
        let receiver = Addr::unchecked("receiver");
        let asset = Addr::unchecked("asset");
        let MaxDepositResponse { max_assets } = query::max_deposit(receiver.clone()).unwrap();
        let received_balance = Cw20CoinVerified {
            address: asset.clone(),
            amount: max_assets + Uint128::one(),
        };
        UNDERLYING_ASSET.save(deps_mut.storage, &asset).unwrap();
        assert_eq!(
            receive_deposit(
                deps_mut,
                env,
                sender,
                received_balance.clone(),
                receiver.clone()
            )
            .unwrap_err(),
            ContractError::ExceededMaxDeposit {
                receiver: receiver.clone(),
                max_assets,
                assets: received_balance.amount
            },
        );
    }

    #[test]
    fn receive_deposit_must_error_if_received_other_token() {
        let mut deps = mock_dependencies();
        let deps_mut = deps.as_mut();
        let env = mock_env();
        let sender = Addr::unchecked("sender");
        let receiver = Addr::unchecked("receiver");
        let received_balance = Cw20CoinVerified {
            address: Addr::unchecked("other-cw20"),
            amount: Uint128::one(),
        };
        UNDERLYING_ASSET
            .save(deps_mut.storage, &Addr::unchecked("asset"))
            .unwrap();
        assert_eq!(
            receive_deposit(
                deps_mut,
                env,
                sender,
                received_balance.clone(),
                receiver.clone()
            )
            .unwrap_err(),
            ContractError::UnsupportedCw20Received {
                addr: received_balance.address.clone()
            },
        );
    }
}
