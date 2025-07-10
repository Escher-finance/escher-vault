use std::collections::HashMap;

use cosmwasm_std::assert_approx_eq;
use cosmwasm_std::testing::MockApi;
use cosmwasm_std::Addr;
use cosmwasm_std::Event;
use cosmwasm_std::StdError;
use cosmwasm_std::Uint128;
use cw4626::cw20::TokenInfoResponse;
use cw_multi_test::{App, ContractWrapper, Executor};

use cw4626::{cw20::*, *};
use cw4626_base::contract;
use cw4626_base::msg::*;
use cw4626_base::ContractError;

const USER: &str = "user";
const USER_TWO: &str = "user-two";
const ADMIN: &str = "admin";

const AMOUNT: Uint128 = Uint128::new(1000);
const HALF_FRAC: (Uint128, Uint128) = (Uint128::one(), Uint128::new(2));

fn attrs_to_map(event: &Event) -> HashMap<&str, &str> {
    event
        .attributes
        .iter()
        .map(|a| (a.key.as_str(), a.value.as_str()))
        .collect()
}

fn addr(api: &MockApi, addr: &str) -> Addr {
    api.addr_make(addr)
}

pub fn get_app() -> App {
    App::default()
}

fn instantitate_asset(app: &mut App) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    )));
    let decimals = 6;
    let amount = Uint128::from(10000 * 10_u64.pow(decimals as u32));
    let api = app.api();
    let admin = addr(api, ADMIN);
    let user = addr(api, USER);
    let msg = cw20_base::msg::InstantiateMsg {
        name: "Token".to_string(),
        symbol: "TKN".to_string(),
        // this is just to be able to airdrop assets at will in the tests
        mint: Some(MinterResponse {
            minter: admin.to_string(),
            cap: None,
        }),
        decimals,
        initial_balances: Vec::from([
            cw20::Cw20Coin {
                amount,
                address: admin.to_string(),
            },
            cw20::Cw20Coin {
                amount,
                address: user.to_string(),
            },
        ]),
        marketing: None,
    };
    app.instantiate_contract(code, admin, &msg, &[], "cw20-base-asset", None)
        .unwrap()
}

fn proper_instantiate(app: &mut App, underlying_token_address: Addr) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        contract::execute,
        contract::instantiate,
        contract::query,
    )));
    let api = app.api();
    let admin = addr(api, ADMIN);
    let msg = InstantiateMsg {
        owner: Some(admin.clone()),
        share_name: "Share Token".to_string(),
        share_symbol: "sTKN".to_string(),
        share_marketing: None,
        underlying_token_address,
    };
    app.instantiate_contract(code, admin, &msg, &[], "cw4626-base", None)
        .unwrap()
}

#[test]
fn instantiates_properly() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let querier = app.wrap();
    let user = addr(api, USER);
    let user_two = addr(api, USER_TWO);
    assert_eq!(
        querier
            .query_wasm_smart::<AssetResponse>(&vault, &QueryMsg::Asset {})
            .unwrap()
            .asset_token_address,
        asset,
        "underlying asset address must match"
    );
    let share_token_info = querier
        .query_wasm_smart::<TokenInfoResponse>(&vault, &QueryMsg::TokenInfo {})
        .unwrap();
    let asset_token_info = querier
        .query_wasm_smart::<TokenInfoResponse>(&asset, &QueryMsg::TokenInfo {})
        .unwrap();
    assert_eq!(
        share_token_info.decimals, asset_token_info.decimals,
        "asset and share must have the same decimals"
    );
    assert_eq!(
        share_token_info.total_supply,
        Uint128::zero(),
        "initial total share supply must be zero",
    );
    assert_eq!(
        querier
            .query_wasm_smart::<TotalAssetsResponse>(&vault, &QueryMsg::TotalAssets {})
            .unwrap()
            .total_managed_assets,
        Uint128::zero(),
        "initial total managed assets must be zero"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<cw_ownable::Ownership<Addr>>(&vault, &QueryMsg::Ownership {})
            .unwrap()
            .owner
            .unwrap(),
        addr(api, ADMIN),
        "admin must be set"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<ConvertToSharesResponse>(
                &vault,
                &QueryMsg::ConvertToShares { assets: AMOUNT }
            )
            .unwrap()
            .shares,
        AMOUNT,
        "initial asset to share conversion must be 1:1"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<ConvertToAssetsResponse>(
                &vault,
                &QueryMsg::ConvertToAssets { shares: AMOUNT }
            )
            .unwrap()
            .assets,
        AMOUNT,
        "initial share to asset conversion must be 1:1"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<MaxDepositResponse>(
                &vault,
                &QueryMsg::MaxDeposit {
                    receiver: user.clone()
                }
            )
            .unwrap()
            .max_assets,
        Uint128::MAX,
        "max deposit must not be limited"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<MaxMintResponse>(
                &vault,
                &QueryMsg::MaxMint {
                    receiver: user.clone()
                }
            )
            .unwrap()
            .max_shares,
        Uint128::MAX,
        "max mint must not be limited"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<MaxWithdrawResponse>(
                &vault,
                &QueryMsg::MaxWithdraw {
                    owner: user.clone()
                }
            )
            .unwrap()
            .max_assets,
        Uint128::zero(),
        "initial max withdraw must be zero"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<MaxRedeemResponse>(
                &vault,
                &QueryMsg::MaxRedeem {
                    owner: user.clone()
                }
            )
            .unwrap()
            .max_shares,
        Uint128::zero(),
        "initial max redeem must be zero"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<PreviewDepositResponse>(
                &vault,
                &QueryMsg::PreviewDeposit { assets: AMOUNT }
            )
            .unwrap()
            .shares,
        AMOUNT,
        "initial preview deposit must be 1:1"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<PreviewMintResponse>(
                &vault,
                &QueryMsg::PreviewMint { shares: AMOUNT }
            )
            .unwrap()
            .assets,
        AMOUNT,
        "initial preview mint must be 1:1"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<PreviewWithdrawResponse>(
                &vault,
                &QueryMsg::PreviewWithdraw { assets: AMOUNT }
            )
            .unwrap()
            .shares,
        AMOUNT,
        "initial preview withdraw must be 1:1"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<PreviewRedeemResponse>(
                &vault,
                &QueryMsg::PreviewRedeem { shares: AMOUNT }
            )
            .unwrap()
            .assets,
        AMOUNT,
        "initial preview redeem must be 1:1"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<AllowanceResponse>(
                &vault,
                &QueryMsg::Allowance {
                    owner: user.to_string(),
                    spender: user_two.to_string()
                }
            )
            .unwrap()
            .allowance,
        Uint128::zero(),
        "can query cw20:allowance"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<AllAllowancesResponse>(
                &vault,
                &QueryMsg::AllAllowances {
                    owner: user.to_string(),
                    start_after: None,
                    limit: None
                }
            )
            .unwrap()
            .allowances,
        vec![],
        "can query cw20:all_allowances"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<AllSpenderAllowancesResponse>(
                &vault,
                &QueryMsg::AllSpenderAllowances {
                    spender: user.to_string(),
                    start_after: None,
                    limit: None
                }
            )
            .unwrap()
            .allowances,
        vec![],
        "can query cw20:all_spender_allowances"
    );
    assert_eq!(
        querier
            .query_wasm_smart::<AllAccountsResponse>(
                &vault,
                &QueryMsg::AllAccounts {
                    start_after: None,
                    limit: None
                }
            )
            .unwrap()
            .accounts,
        Vec::<String>::new(),
        "can query cw20:all_accounts"
    );
    assert!(
        querier
            .query_wasm_smart::<MarketingInfoResponse>(&vault, &QueryMsg::MarketingInfo {})
            .is_ok(),
        "can query cw20:marketing_info"
    );
    assert!(
        matches!(
            querier
                .query_wasm_smart::<DownloadLogoResponse>(&vault, &QueryMsg::DownloadLogo {})
                .unwrap_err(),
            StdError::GenericErr { .. },
        ),
        "must not query cw20:download_logo because it wasn't set"
    );
}

#[test]
fn instantiate_must_fail_if_asset_not_cw20() {
    let mut app = get_app();
    let code = app.store_code(Box::new(ContractWrapper::new(
        contract::execute,
        contract::instantiate,
        contract::query,
    )));
    let api = app.api();
    let admin = addr(api, ADMIN);
    let msg = InstantiateMsg {
        owner: Some(admin.clone()),
        share_name: "Share Token".to_string(),
        share_symbol: "sTKN".to_string(),
        share_marketing: None,
        underlying_token_address: addr(app.api(), "not-cw20"),
    };
    assert!(
        app.instantiate_contract(code, admin, &msg, &[], "cw4626-base", None)
            .is_err(),
        "must validate that asset is cw20"
    );
}

#[test]
fn can_transfer_ownership() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let user = addr(api, USER);
    let admin = addr(api, ADMIN);
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw_ownable::Ownership<Addr>>(&vault, &QueryMsg::Ownership {})
            .unwrap()
            .owner
            .unwrap(),
        admin,
        "initial owner must be admin"
    );
    app.execute_contract(
        admin.clone(),
        vault.clone(),
        &ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
            new_owner: user.to_string(),
            expiry: None,
        }),
        &[],
    )
    .unwrap();
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw_ownable::Ownership<Addr>>(&vault, &QueryMsg::Ownership {})
            .unwrap()
            .owner
            .unwrap(),
        admin,
        "owner must still be admin after transfer request"
    );
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership {}),
        &[],
    )
    .unwrap();
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw_ownable::Ownership<Addr>>(&vault, &QueryMsg::Ownership {})
            .unwrap()
            .owner
            .unwrap(),
        user,
        "owner must be user after accepting the transfer request"
    );
}

#[test]
fn only_owner_can_transfer_ownership() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let user = addr(api, USER);
    let admin = addr(api, ADMIN);
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw_ownable::Ownership<Addr>>(&vault, &QueryMsg::Ownership {})
            .unwrap()
            .owner
            .unwrap(),
        admin,
        "initial owner must be admin"
    );
    let err = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: user.to_string(),
                expiry: None,
            }),
            &[],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::Ownership(cw_ownable::OwnershipError::NotOwner {}),
        err.downcast().unwrap(),
        "non owner transferring ownership must fail"
    );
}

#[test]
fn deposit_no_yield_must_be_one_to_one() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let user = addr(api, USER);
    let user_two = addr(api, USER_TWO);
    let user_assets_balance = app
        .wrap()
        .query_wasm_smart::<BalanceResponse>(
            &asset,
            &QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
        .balance;
    app.execute_contract(
        user.clone(),
        asset.clone(),
        &ExecuteMsg::IncreaseAllowance {
            spender: vault.to_string(),
            amount: AMOUNT,
            expires: None,
        },
        &[],
    )
    .unwrap();
    let wasm_event = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Deposit {
                assets: AMOUNT,
                receiver: user_two.clone(),
            },
            &[],
        )
        .unwrap()
        .events
        .iter()
        .find(|e| e.ty == "wasm")
        .unwrap()
        .clone();
    let attrs = attrs_to_map(&wasm_event);
    assert_eq!(
        attrs["action"], "deposit",
        "must emit the right action attribute"
    );
    assert_eq!(
        attrs["depositor"],
        user.as_str(),
        "must emit the right depositor attribute"
    );
    assert_eq!(
        attrs["receiver"],
        user_two.as_str(),
        "must emit the right receiver attribute"
    );
    assert_eq!(
        attrs["assets_transferred"],
        AMOUNT.to_string().as_str(),
        "must emit the right assets_transferred attribute"
    );
    assert_eq!(
        attrs["shares_minted"],
        AMOUNT.to_string().as_str(),
        "must emit the right shares_minted attribute"
    );
    assert_eq!(
        user_assets_balance - AMOUNT,
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &asset,
                &QueryMsg::Balance {
                    address: user.to_string(),
                },
            )
            .unwrap()
            .balance,
        "must transfer the right amount of assets from the depositor"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<TotalAssetsResponse>(&vault, &QueryMsg::TotalAssets {})
            .unwrap()
            .total_managed_assets,
        AMOUNT,
        "vault total assets must match the initial deposit amount"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: user_two.to_string()
                }
            )
            .unwrap()
            .balance,
        AMOUNT,
        "must mint the same amount of shares to the specified receiver"
    );
}

#[test]
fn mint_no_yield_must_be_one_to_one() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let user = addr(api, USER);
    let user_two = addr(api, USER_TWO);
    let user_assets_balance = app
        .wrap()
        .query_wasm_smart::<BalanceResponse>(
            &asset,
            &QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
        .balance;
    app.execute_contract(
        user.clone(),
        asset.clone(),
        &ExecuteMsg::IncreaseAllowance {
            spender: vault.to_string(),
            amount: AMOUNT,
            expires: None,
        },
        &[],
    )
    .unwrap();
    let wasm_event = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Mint {
                shares: AMOUNT,
                receiver: user_two.clone(),
            },
            &[],
        )
        .unwrap()
        .events
        .iter()
        .find(|e| e.ty == "wasm")
        .unwrap()
        .clone();
    let attrs = attrs_to_map(&wasm_event);
    assert_eq!(
        attrs["action"], "deposit",
        "must emit the right action attribute"
    );
    assert_eq!(
        attrs["depositor"],
        user.as_str(),
        "must emit the right depositor attribute"
    );
    assert_eq!(
        attrs["receiver"],
        user_two.as_str(),
        "must emit the right receiver attribute"
    );
    assert_eq!(
        attrs["assets_transferred"],
        AMOUNT.to_string().as_str(),
        "must emit the right assets_transferred attribute"
    );
    assert_eq!(
        attrs["shares_minted"],
        AMOUNT.to_string().as_str(),
        "must emit the right shares_minted attribute"
    );
    assert_eq!(
        user_assets_balance - AMOUNT,
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &asset,
                &QueryMsg::Balance {
                    address: user.to_string(),
                },
            )
            .unwrap()
            .balance,
        "must transfer the right amount of assets from the depositor"
    );
    assert_eq!(
        AMOUNT,
        app.wrap()
            .query_wasm_smart::<TotalAssetsResponse>(&vault, &QueryMsg::TotalAssets {})
            .unwrap()
            .total_managed_assets,
        "vault total assets must match the initial deposit amount"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: user_two.to_string()
                }
            )
            .unwrap()
            .balance,
        AMOUNT,
        "must mint the same amount of shares to the specified receiver"
    );
}

#[test]
fn deposit_with_yield_must_mint_less_shares() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let admin = addr(api, ADMIN);
    let user = addr(api, USER);
    // initial deposit - 1:1
    {
        app.execute_contract(
            user.clone(),
            asset.clone(),
            &ExecuteMsg::IncreaseAllowance {
                spender: vault.to_string(),
                amount: AMOUNT,
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Deposit {
                assets: AMOUNT,
                receiver: user.clone(),
            },
            &[],
        )
        .unwrap();
        assert_eq!(
            app.wrap()
                .query_wasm_smart::<BalanceResponse>(
                    &vault,
                    &QueryMsg::Balance {
                        address: user.to_string()
                    }
                )
                .unwrap()
                .balance,
            AMOUNT,
            "must mint the same amount of shares to the user after initial deposit"
        );
    }
    // simulate yield by airdropping some assets to the vault
    app.execute_contract(
        admin.clone(),
        asset.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: vault.to_string(),
            amount: AMOUNT,
        },
        &[],
    )
    .unwrap();
    // second deposit - 2:1
    {
        app.execute_contract(
            user.clone(),
            asset.clone(),
            &ExecuteMsg::IncreaseAllowance {
                spender: vault.to_string(),
                amount: AMOUNT,
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Deposit {
                assets: AMOUNT,
                receiver: user.clone(),
            },
            &[],
        )
        .unwrap();
        assert_eq!(
            app.wrap()
                .query_wasm_smart::<BalanceResponse>(
                    &vault,
                    &QueryMsg::Balance {
                        address: user.to_string()
                    }
                )
                .unwrap()
                .balance,
            AMOUNT + AMOUNT.mul_floor(HALF_FRAC),
            "must mint shares to the user accordingly"
        );
    }
}

#[test]
fn mint_with_yield_must_mint_less_shares() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let admin = addr(api, ADMIN);
    let user = addr(api, USER);
    // initial mint - 1:1
    {
        app.execute_contract(
            user.clone(),
            asset.clone(),
            &ExecuteMsg::IncreaseAllowance {
                spender: vault.to_string(),
                amount: AMOUNT,
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Mint {
                shares: AMOUNT,
                receiver: user.clone(),
            },
            &[],
        )
        .unwrap();
        assert_eq!(
            app.wrap()
                .query_wasm_smart::<BalanceResponse>(
                    &vault,
                    &QueryMsg::Balance {
                        address: user.to_string()
                    }
                )
                .unwrap()
                .balance,
            AMOUNT,
            "must mint the same amount of shares to the user after initial deposit"
        );
    }
    // simulate yield by airdropping some assets to the vault
    app.execute_contract(
        admin.clone(),
        asset.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: vault.to_string(),
            amount: AMOUNT,
        },
        &[],
    )
    .unwrap();
    // second mint - 2:1
    {
        app.execute_contract(
            user.clone(),
            asset.clone(),
            &ExecuteMsg::IncreaseAllowance {
                spender: vault.to_string(),
                amount: AMOUNT,
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Mint {
                shares: AMOUNT.mul_floor(HALF_FRAC),
                receiver: user.clone(),
            },
            &[],
        )
        .unwrap();
        assert_eq!(
            app.wrap()
                .query_wasm_smart::<BalanceResponse>(
                    &vault,
                    &QueryMsg::Balance {
                        address: user.to_string()
                    }
                )
                .unwrap()
                .balance,
            AMOUNT + AMOUNT.mul_floor(HALF_FRAC),
            "must mint shares to the user accordingly"
        );
    }
}

#[test]
fn withdraw_to_self_no_yield_must_be_one_to_one() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let user = addr(api, USER);
    // deposit
    {
        app.execute_contract(
            user.clone(),
            asset.clone(),
            &ExecuteMsg::IncreaseAllowance {
                spender: vault.to_string(),
                amount: AMOUNT,
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Deposit {
                assets: AMOUNT,
                receiver: user.clone(),
            },
            &[],
        )
        .unwrap();
    }
    let user_assets_balance = app
        .wrap()
        .query_wasm_smart::<BalanceResponse>(
            &asset,
            &QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
        .balance;
    // withdraw more must fail
    assert_eq!(
        ContractError::ExceededMaxWithdraw {
            owner: user.to_string(),
            assets: (AMOUNT + Uint128::one()).u128(),
            max_assets: AMOUNT.u128()
        },
        app.execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Withdraw {
                assets: AMOUNT + Uint128::one(),
                receiver: user.clone(),
                owner: user.clone(),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap(),
        "must error with exceeded max withdraw"
    );
    // withdraw all to self
    let wasm_event = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Withdraw {
                assets: AMOUNT,
                receiver: user.clone(),
                owner: user.clone(),
            },
            &[],
        )
        .unwrap()
        .events
        .iter()
        .find(|e| e.ty == "wasm")
        .unwrap()
        .clone();
    let attrs = attrs_to_map(&wasm_event);
    assert_eq!(
        attrs["action"], "withdraw",
        "must emit the right action attribute"
    );
    assert_eq!(
        attrs["withdrawer"],
        user.as_str(),
        "must emit the right withdrawer attribute"
    );
    assert_eq!(
        attrs["receiver"],
        user.as_str(),
        "must emit the right receiver attribute"
    );
    assert_eq!(
        attrs["assets_received"],
        AMOUNT.to_string().as_str(),
        "must emit the right assets_received attribute"
    );
    assert_eq!(
        attrs["shares_burned"],
        AMOUNT.to_string().as_str(),
        "must emit the right shares_burned attribute"
    );
    assert_eq!(
        user_assets_balance + AMOUNT,
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &asset,
                &QueryMsg::Balance {
                    address: user.to_string(),
                },
            )
            .unwrap()
            .balance,
        "user must receive the right amount of assets"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<TotalAssetsResponse>(&vault, &QueryMsg::TotalAssets {})
            .unwrap()
            .total_managed_assets,
        Uint128::zero(),
        "vault total assets must adapt after withdraw"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: user.to_string()
                }
            )
            .unwrap()
            .balance,
        Uint128::zero(),
        "must burn the same amount of shares from the user"
    );
}

#[test]
fn redeem_to_self_no_yield_must_be_one_to_one() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let user = addr(api, USER);
    // deposit
    {
        app.execute_contract(
            user.clone(),
            asset.clone(),
            &ExecuteMsg::IncreaseAllowance {
                spender: vault.to_string(),
                amount: AMOUNT,
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Deposit {
                assets: AMOUNT,
                receiver: user.clone(),
            },
            &[],
        )
        .unwrap();
    }
    let user_assets_balance = app
        .wrap()
        .query_wasm_smart::<BalanceResponse>(
            &asset,
            &QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
        .balance;
    // redeem more must fail
    assert_eq!(
        ContractError::ExceededMaxRedeem {
            owner: user.to_string(),
            shares: (AMOUNT + Uint128::one()).u128(),
            max_shares: AMOUNT.u128()
        },
        app.execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Redeem {
                shares: AMOUNT + Uint128::one(),
                receiver: user.clone(),
                owner: user.clone(),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap(),
        "must error with exceeded max redeem"
    );
    // redeem all to self
    let wasm_event = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Redeem {
                shares: AMOUNT,
                receiver: user.clone(),
                owner: user.clone(),
            },
            &[],
        )
        .unwrap()
        .events
        .iter()
        .find(|e| e.ty == "wasm")
        .unwrap()
        .clone();
    let attrs = attrs_to_map(&wasm_event);
    assert_eq!(
        attrs["action"], "withdraw",
        "must emit the right action attribute"
    );
    assert_eq!(
        attrs["withdrawer"],
        user.as_str(),
        "must emit the right withdrawer attribute"
    );
    assert_eq!(
        attrs["receiver"],
        user.as_str(),
        "must emit the right receiver attribute"
    );
    assert_eq!(
        attrs["assets_received"],
        AMOUNT.to_string().as_str(),
        "must emit the right assets_received attribute"
    );
    assert_eq!(
        attrs["shares_burned"],
        AMOUNT.to_string().as_str(),
        "must emit the right shares_burned attribute"
    );
    assert_eq!(
        user_assets_balance + AMOUNT,
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &asset,
                &QueryMsg::Balance {
                    address: user.to_string(),
                },
            )
            .unwrap()
            .balance,
        "user must receive the right amount of assets"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<TotalAssetsResponse>(&vault, &QueryMsg::TotalAssets {})
            .unwrap()
            .total_managed_assets,
        Uint128::zero(),
        "vault total assets must adapt after withdraw"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: user.to_string()
                }
            )
            .unwrap()
            .balance,
        Uint128::zero(),
        "must burn the same amount of shares from the user"
    );
}

#[test]
fn withdraw_from_must_deduct_allowance() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let user = addr(api, USER);
    let user_two = addr(api, USER_TWO);
    // deposit
    {
        app.execute_contract(
            user.clone(),
            asset.clone(),
            &ExecuteMsg::IncreaseAllowance {
                spender: vault.to_string(),
                amount: AMOUNT,
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Deposit {
                assets: AMOUNT,
                receiver: user.clone(),
            },
            &[],
        )
        .unwrap();
    }
    // allow other to withdraw half
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::IncreaseAllowance {
            spender: user_two.to_string(),
            amount: AMOUNT.mul_floor(HALF_FRAC),
            expires: None,
        },
        &[],
    )
    .unwrap();
    // withdraw half from user
    app.execute_contract(
        user_two.clone(),
        vault.clone(),
        &ExecuteMsg::Withdraw {
            assets: AMOUNT.mul_floor(HALF_FRAC),
            receiver: user_two.clone(),
            owner: user.clone(),
        },
        &[],
    )
    .unwrap();
    let err = app
        .execute_contract(
            user_two.clone(),
            vault.clone(),
            &ExecuteMsg::Withdraw {
                assets: AMOUNT.mul_floor(HALF_FRAC),
                receiver: user_two.clone(),
                owner: user.clone(),
            },
            &[],
        )
        .unwrap_err();
    assert!(
        matches!(
            err.downcast().unwrap(),
            ContractError::ShareCw20Error(cw20_base::ContractError::Std(StdError::Overflow { .. }))
        ),
        "attempt to withdraw more than allowance must fail with overflow"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<AllowanceResponse>(
                &asset,
                &QueryMsg::Allowance {
                    owner: user.to_string(),
                    spender: user_two.to_string()
                },
            )
            .unwrap()
            .allowance,
        Uint128::zero(),
        "new allowance must be zero"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &asset,
                &QueryMsg::Balance {
                    address: user_two.to_string(),
                },
            )
            .unwrap()
            .balance,
        AMOUNT.mul_floor(HALF_FRAC),
        "other must receive the right amount of assets"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: user.to_string()
                }
            )
            .unwrap()
            .balance,
        AMOUNT.mul_floor(HALF_FRAC),
        "must burn the same amount of shares from the user"
    );
}

#[test]
fn withdraw_with_yield_must_mint_less_shares() {
    let mut app = get_app();
    let asset = instantitate_asset(&mut app);
    let vault = proper_instantiate(&mut app, asset.clone());
    let api = app.api();
    let admin = addr(api, ADMIN);
    let user = addr(api, USER);
    // deposit - 1:1
    {
        app.execute_contract(
            user.clone(),
            asset.clone(),
            &ExecuteMsg::IncreaseAllowance {
                spender: vault.to_string(),
                amount: AMOUNT,
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Deposit {
                assets: AMOUNT,
                receiver: user.clone(),
            },
            &[],
        )
        .unwrap();
    }
    // simulate yield by airdropping some assets to the vault
    app.execute_contract(
        admin.clone(),
        asset.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: vault.to_string(),
            amount: AMOUNT,
        },
        &[],
    )
    .unwrap();
    // withdraw initial deposit to self - 2:1
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::Withdraw {
            assets: AMOUNT,
            receiver: user.clone(),
            owner: user.clone(),
        },
        &[],
    )
    .unwrap();
    assert_approx_eq!(
        app.wrap()
            .query_wasm_smart::<BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: user.to_string()
                }
            )
            .unwrap()
            .balance,
        AMOUNT.mul_floor(HALF_FRAC),
        "0.01",
        "must still have half the shares"
    );
}
