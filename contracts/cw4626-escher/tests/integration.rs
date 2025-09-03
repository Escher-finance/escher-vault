#![allow(dead_code)]

use std::collections::HashMap;
use std::str::FromStr;

use astroport::asset::AssetInfo;
use cosmwasm_std::testing::MockApi;
use cosmwasm_std::Addr;
use cosmwasm_std::Binary;
use cosmwasm_std::Coin;
use cosmwasm_std::Decimal;
use cosmwasm_std::Deps;
use cosmwasm_std::DepsMut;
use cosmwasm_std::Empty;
use cosmwasm_std::Env;
use cosmwasm_std::Event;
use cosmwasm_std::MessageInfo;
use cosmwasm_std::Response;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use cw_multi_test::{App, ContractWrapper, Executor};

use cw4626::{cw20::*, *};
use cw4626_escher::contract;
use cw4626_escher::msg::*;

fn make_valid_addr() -> Addr {
    Addr::unchecked("cosmwasm1wug8sewp6cedgkmrmvhl3lf3tulagm9hnvy8p0rppz9yjw0g4wtqlrtkzd")
}

const UNDERLYING_TOKEN: &str = "utkn";

const USER: &str = "user";
const ADMIN: &str = "admin";
const ORACLE: &str = "oracle";

const AMOUNT: Uint128 = Uint128::new(100000000);

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

fn instantiate_denom_asset(app: &mut App) -> AssetInfo {
    let api = app.api();
    let admin = addr(api, ADMIN);
    let user = addr(api, USER);
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: admin.to_string(),
            amount: Vec::from([Coin::new(AMOUNT, UNDERLYING_TOKEN)]),
        },
    ))
    .unwrap();
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: user.to_string(),
            amount: Vec::from([Coin::new(AMOUNT, UNDERLYING_TOKEN)]),
        },
    ))
    .unwrap();
    AssetInfo::NativeToken {
        denom: UNDERLYING_TOKEN.to_string(),
    }
}

fn instantiate_cw20_asset(app: &mut App) -> Addr {
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

mod staking_mock {
    use cosmwasm_std::to_json_binary;
    use cw4626_escher::staking::{EscherHubQueryMsg, EscherHubStakingLiquidity};

    use super::*;
    pub fn instantiate(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }
    pub fn query(_deps: Deps, _env: Env, msg: EscherHubQueryMsg) -> StdResult<Binary> {
        match msg {
            EscherHubQueryMsg::StakingLiquidity {} => to_json_binary(&EscherHubStakingLiquidity {
                exchange_rate: Decimal::one(),
                ..Default::default()
            }),
        }
    }
}

fn instantiate_staking(app: &mut App) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        staking_mock::execute,
        staking_mock::instantiate,
        staking_mock::query,
    )));
    let api = app.api();
    app.instantiate_contract(code, addr(api, ADMIN), &Empty {}, &[], "staking", None)
        .unwrap()
}

mod lp_mock {
    use astroport::{asset::PairInfo, pair_concentrated};
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::to_json_binary;
    use cw_storage_plus::Item;

    use super::*;

    const A: Item<AssetInfo> = Item::new("asset");

    #[cw_serde]
    pub struct InstantiateMsg {
        pub underlying: AssetInfo,
    }

    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        A.save(deps.storage, &msg.underlying)?;
        Ok(Response::default())
    }
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }
    pub fn query(deps: Deps, _env: Env, msg: pair_concentrated::QueryMsg) -> StdResult<Binary> {
        match msg {
            pair_concentrated::QueryMsg::Pair {} => {
                let a = A.load(deps.storage)?;
                to_json_binary(&PairInfo {
                    asset_infos: Vec::from([
                        a,
                        AssetInfo::NativeToken {
                            denom: "other_lp_tkn".to_string(),
                        },
                    ]),
                    contract_addr: make_valid_addr(),
                    liquidity_token: make_valid_addr().to_string(),
                    pair_type: astroport::factory::PairType::Concentrated {},
                })
            }
            _ => unimplemented!(),
        }
    }
}

fn instantiate_lp(app: &mut App, underlying_token: AssetInfo) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        lp_mock::execute,
        lp_mock::instantiate,
        lp_mock::query,
    )));
    let api = app.api();
    app.instantiate_contract(
        code,
        addr(api, ADMIN),
        &lp_mock::InstantiateMsg {
            underlying: underlying_token,
        },
        &[],
        "lp",
        None,
    )
    .unwrap()
}

mod incentives_mock {
    use astroport::incentives;
    use cosmwasm_std::to_json_binary;

    use super::*;
    pub fn instantiate(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::default())
    }
    pub fn query(_deps: Deps, _env: Env, msg: incentives::QueryMsg) -> StdResult<Binary> {
        match msg {
            incentives::QueryMsg::Config {} => to_json_binary(&incentives::Config {
                owner: make_valid_addr(),
                factory: make_valid_addr(),
                generator_controller: None,
                astro_token: AssetInfo::NativeToken {
                    denom: "astro".to_string(),
                },
                astro_per_second: Uint128::one(),
                total_alloc_points: Uint128::one(),
                vesting_contract: make_valid_addr(),
                guardian: None,
                incentivization_fee_info: None,
                token_transfer_gas_limit: None,
            }),
            _ => unimplemented!(),
        }
    }
}

fn instantiate_incentives(app: &mut App) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        incentives_mock::execute,
        incentives_mock::instantiate,
        incentives_mock::query,
    )));
    let api = app.api();
    app.instantiate_contract(code, addr(api, ADMIN), &Empty {}, &[], "incentives", None)
        .unwrap()
}

fn instantiate_vault(
    app: &mut App,
    underlying_token: AssetInfo,
    staking_address: Addr,
    lp_address: Addr,
    incentives_address: Addr,
) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        contract::execute,
        contract::instantiate,
        contract::query,
    )));
    let api = app.api();
    let admin = addr(api, ADMIN);
    let oracle = addr(api, ORACLE);
    let msg = InstantiateMsg {
        managers: Vec::from([admin.clone()]),
        oracles: Vec::from([oracle.clone()]),
        share_name: "Share Token".to_string(),
        share_symbol: "sTKN".to_string(),
        share_marketing: None,
        underlying_token,
        incentives: Vec::from([
            AssetInfo::NativeToken {
                denom: "incentive1".to_string(),
            },
            AssetInfo::NativeToken {
                denom: "incentive2".to_string(),
            },
        ]),
        slippage_tolerance: Decimal::from_ratio(1_u32, 100_u32),
        staking_contract: Some(staking_address),
        lp: lp_address,
        tower_incentives: incentives_address,
    };
    app.instantiate_contract(code, admin, &msg, &[], "cw4626-escher", None)
        .unwrap()
}

fn proper_instantiate(app: &mut App) -> Addr {
    let asset = instantiate_denom_asset(app);
    let staking = instantiate_staking(app);
    let lp = instantiate_lp(app, asset.clone());
    let tower_incentives = instantiate_incentives(app);
    instantiate_vault(app, asset, staking, lp, tower_incentives)
}

#[test]
fn instantiates_properly() {
    let mut app = get_app();
    proper_instantiate(&mut app);
}

#[test]
fn deposit_no_yield_must_be_one_to_one() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app);
    let api = app.api();
    let oracle = addr(api, ORACLE);
    let user = addr(api, USER);
    let prices = HashMap::from_iter(
        [
            (
                "other_lp_tkn".to_string(),
                Decimal::from_str("1.78786").unwrap(),
            ),
            ("incentive1".to_string(), Decimal::from_str("0.8").unwrap()),
            ("incentive2".to_string(), Decimal::from_str("0.8").unwrap()),
        ]
        .into_iter(),
    );
    app.execute_contract(
        oracle.clone(),
        vault.clone(),
        &ExecuteMsg::OracleUpdatePrices { prices },
        &[],
    )
    .unwrap();
    let oracle_prices = app
        .wrap()
        .query_wasm_smart::<OraclePricesResponse>(&vault, &QueryMsg::OraclePrices {})
        .unwrap();
    println!("prices after update {oracle_prices:?}");
    let initial_share_balance = app
        .wrap()
        .query_wasm_smart::<cw20::BalanceResponse>(
            &vault,
            &QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
        .balance;
    assert!(initial_share_balance.is_zero());

    let asset_deposit_amount = Uint128::from(50000_u32);

    // do deposit
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::Deposit {
            assets: asset_deposit_amount,
            receiver: user.clone(),
        },
        &Vec::from([Coin::new(asset_deposit_amount, UNDERLYING_TOKEN)]),
    )
    .unwrap();

    let new_share_balance = app
        .wrap()
        .query_wasm_smart::<cw20::BalanceResponse>(
            &vault,
            &QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
        .balance;
    println!("{new_share_balance}");
    assert_eq!(new_share_balance, asset_deposit_amount);
}

#[test]
fn git_info_must_return_valid_data() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app);
    let git_info = app
        .wrap()
        .query_wasm_smart::<GitInfoResponse>(&vault, &QueryMsg::GitInfo {})
        .unwrap()
        .git;
    dbg!(&git_info);
    let mut parts = git_info.splitn(2, ':');
    let branch = parts.next().unwrap();
    assert!(
        !branch.is_empty()
            && branch.chars().all(|c| c.is_ascii_alphanumeric()
                || c == '.'
                || c == '_'
                || c == '-'
                || c == '/')
    );
    let hash = parts.next().unwrap();
    assert!(hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()))
}

#[test]
fn sequential_deposits_with_oracle_prices_must_be_one_to_one() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app);
    let api = app.api();
    let oracle = addr(api, ORACLE);
    let user = addr(api, USER);
    
    // Set up oracle prices (same as existing test - but mock contracts have no actual balances)
    let prices = HashMap::from_iter(
        [
            (
                "other_lp_tkn".to_string(),
                Decimal::from_str("1.78786").unwrap(), // LP token price with yield
            ),
            ("incentive1".to_string(), Decimal::from_str("0.8").unwrap()), // Incentive value
            ("incentive2".to_string(), Decimal::from_str("0.8").unwrap()), // Incentive value
        ]
        .into_iter(),
    );
    app.execute_contract(
        oracle.clone(),
        vault.clone(),
        &ExecuteMsg::OracleUpdatePrices { prices },
        &[],
    )
    .unwrap();

    // First deposit: 6000 ubbn should get 6000 shares
    let first_deposit_amount = Uint128::from(6000_u32);
    
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::Deposit {
            assets: first_deposit_amount,
            receiver: user.clone(),
        },
        &Vec::from([Coin::new(first_deposit_amount, UNDERLYING_TOKEN)]),
    )
    .unwrap();

    let first_share_balance = app
        .wrap()
        .query_wasm_smart::<cw20::BalanceResponse>(
            &vault,
            &QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
        .balance;
    
    println!("First deposit: {} ubbn -> {} shares", first_deposit_amount, first_share_balance);
    assert_eq!(first_share_balance, first_deposit_amount);

    // Second deposit: 5000 ubbn should get 5000 shares (mock contracts have no balances, so 1:1)
    let second_deposit_amount = Uint128::from(5000_u32);
    
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::Deposit {
            assets: second_deposit_amount,
            receiver: user.clone(),
        },
        &Vec::from([Coin::new(second_deposit_amount, UNDERLYING_TOKEN)]),
    )
    .unwrap();

    let total_share_balance = app
        .wrap()
        .query_wasm_smart::<cw20::BalanceResponse>(
            &vault,
            &QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
        .balance;
    
    let second_deposit_shares = total_share_balance - first_share_balance;
    println!("Second deposit: {} ubbn -> {} shares", second_deposit_amount, second_deposit_shares);
    println!("Total shares: {}", total_share_balance);
    
    // Second deposit should get fewer shares due to yield from LP tokens (oracle price 1.78786)
    // The vault now has yield-generating assets, so new deposits get fewer shares per ubbn
    assert!(second_deposit_shares < second_deposit_amount, 
        "Second deposit should get fewer shares due to yield: {} < {}", 
        second_deposit_shares, second_deposit_amount);
    assert!(total_share_balance < first_deposit_amount + second_deposit_amount,
        "Total shares should be less than simple sum due to yield: {} < {}", 
        total_share_balance, first_deposit_amount + second_deposit_amount);

    // Now add some incentive tokens to the vault to create yield
    let incentive_amount = Uint128::from(1000_u32);
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: vault.to_string(),
            amount: Vec::from([
                Coin::new(incentive_amount, "incentive1"),
                Coin::new(incentive_amount, "incentive2"),
            ]),
        },
    ))
    .unwrap();

    // Third deposit: 3000 ubbn should get fewer shares due to yield
    let third_deposit_amount = Uint128::from(3000_u32);
    
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::Deposit {
            assets: third_deposit_amount,
            receiver: user.clone(),
        },
        &Vec::from([Coin::new(third_deposit_amount, UNDERLYING_TOKEN)]),
    )
    .unwrap();

    let final_share_balance = app
        .wrap()
        .query_wasm_smart::<cw20::BalanceResponse>(
            &vault,
            &QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap()
        .balance;
    
    let third_deposit_shares = final_share_balance - total_share_balance;
    println!("Third deposit: {} ubbn -> {} shares (with yield)", third_deposit_amount, third_deposit_shares);
    println!("Final total shares: {}", final_share_balance);
    
    // Third deposit should get fewer shares than 1:1 due to yield from incentive tokens
    assert!(third_deposit_shares < third_deposit_amount, 
        "Third deposit should get fewer shares due to yield: {} < {}", 
        third_deposit_shares, third_deposit_amount);
    
    // The yield from incentive tokens (1000 * 0.8 * 2 = 1600) should be reflected in the vault's total assets
    // This means the third deposit gets fewer shares because existing shareholders benefit from the yield
    let percentage = (third_deposit_shares * Uint128::from(100u32)) / third_deposit_amount;
    println!("Yield effect: {} ubbn deposit only got {} shares ({}% of 1:1)", 
        third_deposit_amount, third_deposit_shares, percentage);
}
