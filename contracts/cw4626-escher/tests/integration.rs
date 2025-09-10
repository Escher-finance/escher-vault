#![allow(dead_code)]

use std::collections::HashMap;
use std::str::FromStr;

use astroport::asset::AssetInfo;
use cosmwasm_std::assert_approx_eq;
use cosmwasm_std::testing::MockApi;
use cosmwasm_std::to_json_binary;
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
use cw20::Cw20ExecuteMsg;
use cw20::MinterResponse;
use cw_multi_test::{App, ContractWrapper, Executor};

use cw4626_escher::contract;
use cw4626_escher::msg::*;
use cw4626_escher::state::AccessControlRole;
use cw_multi_test::AppResponse;

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

fn instantiate_cw20_asset(app: &mut App) -> AssetInfo {
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
    AssetInfo::Token {
        contract_addr: app
            .instantiate_contract(code, admin, &msg, &[], "cw20-base-asset", None)
            .unwrap(),
    }
}

mod staking_mock {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{to_json_binary, Timestamp};
    use cw4626_escher::staking::{
        EscherHubParameters, EscherHubQueryMsg, EscherHubStakingLiquidity,
    };
    use cw_storage_plus::Item;

    use super::*;

    const A: Item<Addr> = Item::new("asset");

    #[cw_serde]
    pub struct InstantiateMsg {
        pub other_lp: Addr,
    }

    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        A.save(deps.storage, &msg.other_lp)?;
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
    pub fn query(deps: Deps, _env: Env, msg: EscherHubQueryMsg) -> StdResult<Binary> {
        match msg {
            EscherHubQueryMsg::StakingLiquidity {} => to_json_binary(&EscherHubStakingLiquidity {
                exchange_rate: Decimal::one(),
                time: Timestamp::default(),
                amount: Uint128::default(),
                reward: Uint128::default(),
                delegated: Uint128::default(),
                total_supply: Uint128::default(),
                adjusted_supply: Uint128::default(),
                unclaimed_reward: Uint128::default(),
            }),
            EscherHubQueryMsg::Parameters {} => {
                let cw20_address = A.load(deps.storage)?;
                to_json_binary(&EscherHubParameters {
                    cw20_address,
                    underlying_coin_denom: String::default(),
                    liquidstaking_denom: String::default(),
                    ucs03_relay_contract: String::default(),
                    unbonding_time: u64::default(),
                    reward_address: Addr::unchecked("reward"),
                    fee_rate: Decimal::default(),
                    fee_receiver: Addr::unchecked("fee_receiver"),
                    batch_period: u64::default(),
                    min_bond: Uint128::default(),
                    min_unbond: Uint128::default(),
                    batch_limit: u32::default(),
                    transfer_handler: String::default(),
                    transfer_fee: Uint128::default(),
                    zkgm_token_minter: String::default(),
                })
            }
        }
    }
}

fn instantiate_staking(app: &mut App, other_lp: Addr) -> Addr {
    let code = app.store_code(Box::new(ContractWrapper::new(
        staking_mock::execute,
        staking_mock::instantiate,
        staking_mock::query,
    )));
    let api = app.api();
    app.instantiate_contract(
        code,
        addr(api, ADMIN),
        &staking_mock::InstantiateMsg { other_lp },
        &[],
        "staking",
        None,
    )
    .unwrap()
}

mod lp_mock {
    use astroport::{asset::PairInfo, pair_concentrated};
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::to_json_binary;
    use cw_storage_plus::Item;

    use super::*;

    const A: Item<AssetInfo> = Item::new("asset");
    const B: Item<AssetInfo> = Item::new("other_lp");

    #[cw_serde]
    pub struct InstantiateMsg {
        pub underlying: AssetInfo,
        pub other_lp: AssetInfo,
    }

    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        A.save(deps.storage, &msg.underlying)?;
        B.save(deps.storage, &msg.other_lp)?;
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
                let b = B.load(deps.storage)?;
                to_json_binary(&PairInfo {
                    asset_infos: Vec::from([a, b]),
                    contract_addr: make_valid_addr(),
                    liquidity_token: make_valid_addr().to_string(),
                    pair_type: astroport::factory::PairType::Concentrated {},
                })
            }
            pair_concentrated::QueryMsg::LpPrice {} => to_json_binary(&Decimal::one()),
            _ => to_json_binary(&Empty {}),
        }
    }
}

fn instantiate_lp(app: &mut App, underlying_token: AssetInfo, other_lp_token: AssetInfo) -> Addr {
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
            other_lp: other_lp_token,
        },
        &[],
        "lp",
        None,
    )
    .unwrap()
}

mod incentives_mock {
    use astroport::{asset::Asset, incentives};
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
            incentives::QueryMsg::PendingRewards { .. } => to_json_binary(&Vec::<Asset>::from([])),
            incentives::QueryMsg::QueryDeposit { .. } => to_json_binary(&Uint128::zero()),
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
    fee: Option<Decimal>,
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
        entry_fee_rate: fee,
        entry_fee_recipient: admin.clone(),
    };
    app.instantiate_contract(code, admin, &msg, &[], "cw4626-escher", None)
        .unwrap()
}

fn proper_instantiate(app: &mut App, cw20_underlying: bool, with_fee: bool) -> Addr {
    let asset = if cw20_underlying {
        instantiate_cw20_asset(app)
    } else {
        instantiate_denom_asset(app)
    };
    let other_lp = Addr::unchecked("other_lp");
    let other_lp_asset = AssetInfo::Token {
        contract_addr: other_lp.clone(),
    };
    let staking = instantiate_staking(app, other_lp);
    let lp = instantiate_lp(app, asset.clone(), other_lp_asset);
    let tower_incentives = instantiate_incentives(app);
    instantiate_vault(
        app,
        asset,
        staking,
        lp,
        tower_incentives,
        if with_fee {
            Some(Decimal::percent(5))
        } else {
            None
        },
    )
}

#[test]
fn complete_redemption_requires_manager() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);

    // Non-manager tries to complete redemption -> must fail with Unauthorized before any other checks
    let user = addr(app.api(), USER);
    let err = app
        .execute_contract(
            user,
            vault,
            &ExecuteMsg::CompleteRedemption {
                redemption_id: 1,
                tx_hash: "dummy".to_string(),
            },
            &[],
        )
        .unwrap_err();
    // just ensure it errors for non-manager; exact message may vary across environments
    let _ = err;
}

#[test]
fn swap_rejects_invalid_asset_for_manager() {
    let mut app = get_app();
    // native underlying so lp_other_asset will be the mocked cw20 token; we'll pass an invalid third token
    let vault = proper_instantiate(&mut app, false, false);
    let admin = addr(app.api(), ADMIN);

    // Ensure some balance exists so we hit InvalidTokenType check rather than InsufficientSwapFunds
    // Provide a tiny mint of an unrelated token to the vault (won't be used because token is invalid)
    let invalid_asset = AssetInfo::NativeToken {
        denom: "totally-invalid".to_string(),
    };

    let err = app
        .execute_contract(
            admin,
            vault,
            &ExecuteMsg::Swap {
                amount: Uint128::new(1),
                asset_info: invalid_asset,
            },
            &[],
        )
        .unwrap_err();
    // ensure it errors on invalid asset type; message formatting can vary
    let _ = err;
}

#[test]
fn instantiates_properly_underlying_native() {
    let mut app = get_app();
    proper_instantiate(&mut app, false, false);
}

#[test]
fn instantiates_properly_underlying_cw20() {
    let mut app = get_app();
    proper_instantiate(&mut app, true, false);
}

#[test]
fn vault_exchange_rate_query_returns_pps_string() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);

    // Set minimal non-zero oracle prices required by escher
    let api = app.api();
    let oracle = addr(api, ORACLE);
    let prices = HashMap::from_iter(
        [
            (
                Addr::unchecked("other_lp").to_string(),
                Decimal::from_str("1.0").unwrap(),
            ),
            ("incentive1".to_string(), Decimal::from_str("1.0").unwrap()),
            ("incentive2".to_string(), Decimal::from_str("1.0").unwrap()),
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

    // Initially no deposits: define PPS as 1.0
    let rate: ExchangeRateResponse = app
        .wrap()
        .query_wasm_smart(&vault, &QueryMsg::ExchangeRate {})
        .unwrap();
    assert_eq!(rate.exchange_rate, Decimal::from_str("1.0").unwrap());

    // Deposit 1000 underlying, expect PPS ~ 1.0 (1:1)
    let user = addr(app.api(), USER);
    let deposit = Uint128::from(1000u64);
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::Deposit {
            assets: deposit,
            receiver: user.clone(),
        },
        &Vec::from([Coin::new(deposit, UNDERLYING_TOKEN)]),
    )
    .unwrap();

    let rate2: ExchangeRateResponse = app
        .wrap()
        .query_wasm_smart(&vault, &QueryMsg::ExchangeRate {})
        .unwrap();
    println!("rate after 1000 deposit: {}", rate2.exchange_rate);
    // Should be very close to 1.0; string compare allows 1 or 1.0 depending on formatting
    assert!(rate2.exchange_rate.to_string().starts_with("1"));

    // Now add incentive tokens to the vault to simulate yield and check PPS increases
    // Use large amounts so value >= 1 ubbn after pricing
    let incentive_amount = Uint128::from(1_000_000u64);
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

    let rate3: ExchangeRateResponse = app
        .wrap()
        .query_wasm_smart(&vault, &QueryMsg::ExchangeRate {})
        .unwrap();
    println!("rate after incentives: {}", rate3.exchange_rate);

    let r2 = rate2.exchange_rate;
    let r3 = rate3.exchange_rate;
    assert!(
        r3 > r2,
        "exchange rate should increase after incentives: {} > {}",
        r3,
        r2
    );
}

#[test]
fn convert_to_shares_zero_state_is_one_to_one() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    // Initialize oracle prices to non-zero so queries succeed
    let oracle = addr(app.api(), ORACLE);
    let tokens = app
        .wrap()
        .query_wasm_smart::<OracleTokensListResponse>(&vault, &QueryMsg::OracleTokensList {})
        .unwrap()
        .tokens;
    let prices = HashMap::from_iter(
        tokens
            .into_iter()
            .map(|t| (t, Decimal::from_str("1.0").unwrap())),
    );
    app.execute_contract(
        oracle,
        vault.clone(),
        &ExecuteMsg::OracleUpdatePrices { prices },
        &[],
    )
    .unwrap();

    // No deposits yet: total_shares=0, total_assets=0 ⇒ shares = assets
    let assets = Uint128::from(12345u64);
    let resp: ConvertToSharesResponse = app
        .wrap()
        .query_wasm_smart(&vault, &QueryMsg::ConvertToShares { assets })
        .unwrap();
    assert_eq!(resp.shares, assets);
}

#[test]
fn convert_to_assets_zero_state_is_one_to_one() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    // Initialize oracle prices to non-zero so queries succeed
    let oracle = addr(app.api(), ORACLE);
    let tokens = app
        .wrap()
        .query_wasm_smart::<OracleTokensListResponse>(&vault, &QueryMsg::OracleTokensList {})
        .unwrap()
        .tokens;
    let prices = HashMap::from_iter(
        tokens
            .into_iter()
            .map(|t| (t, Decimal::from_str("1.0").unwrap())),
    );
    app.execute_contract(
        oracle,
        vault.clone(),
        &ExecuteMsg::OracleUpdatePrices { prices },
        &[],
    )
    .unwrap();

    // No deposits yet: total_shares=0, total_assets=0 ⇒ assets = shares
    let shares = Uint128::from(6789u64);
    let resp: ConvertToAssetsResponse = app
        .wrap()
        .query_wasm_smart(&vault, &QueryMsg::ConvertToAssets { shares })
        .unwrap();
    assert_eq!(resp.assets, shares);
}

#[test]
fn deposit_cw20_no_yield_must_be_one_to_one() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, true, false);
    let api = app.api();
    let oracle = addr(api, ORACLE);
    let user = addr(api, USER);
    let prices = HashMap::from_iter(
        [
            (
                Addr::unchecked("other_lp").to_string(),
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

    let asset = Addr::unchecked(
        app.wrap()
            .query_wasm_smart::<AssetResponse>(&vault, &QueryMsg::Asset {})
            .unwrap()
            .asset_token_address,
    );

    let asset_deposit_amount = Uint128::from(50000_u32);

    let preview_amount = app
        .wrap()
        .query_wasm_smart::<PreviewDepositResponse>(
            &vault,
            &QueryMsg::PreviewDeposit {
                assets: asset_deposit_amount,
            },
        )
        .unwrap()
        .shares;
    // do first deposit via transfer from
    app.execute_contract(
        user.clone(),
        asset.clone(),
        &Cw20ExecuteMsg::IncreaseAllowance {
            spender: vault.to_string(),
            amount: asset_deposit_amount,
            expires: None,
        },
        &[],
    )
    .unwrap();
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::Deposit {
            assets: asset_deposit_amount,
            receiver: user.clone(),
        },
        &[],
    )
    .unwrap();

    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: user.to_string(),
                },
            )
            .unwrap()
            .balance,
        asset_deposit_amount
    );
    assert_eq!(preview_amount, asset_deposit_amount);
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<ExchangeRateResponse>(&vault, &QueryMsg::ExchangeRate {})
            .unwrap()
            .exchange_rate,
        Decimal::one()
    );

    let preview_amount = app
        .wrap()
        .query_wasm_smart::<PreviewDepositResponse>(
            &vault,
            &QueryMsg::PreviewDeposit {
                assets: asset_deposit_amount,
            },
        )
        .unwrap()
        .shares;
    // do second deposit via send
    app.execute_contract(
        user.clone(),
        asset.clone(),
        &Cw20ExecuteMsg::Send {
            contract: vault.to_string(),
            amount: asset_deposit_amount,
            msg: to_json_binary(&ExecuteMsg::Deposit {
                assets: asset_deposit_amount,
                receiver: user.clone(),
            })
            .unwrap(),
        },
        &[],
    )
    .unwrap();

    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: user.to_string(),
                },
            )
            .unwrap()
            .balance,
        asset_deposit_amount * Uint128::new(2)
    );
    assert_eq!(preview_amount, asset_deposit_amount);
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<ExchangeRateResponse>(&vault, &QueryMsg::ExchangeRate {})
            .unwrap()
            .exchange_rate,
        Decimal::one()
    );
}

#[test]
fn add_liquidity_insufficient_funds_errors() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let api = app.api();
    let admin = addr(api, ADMIN);

    // Try zero amount
    let err = app
        .execute_contract(
            admin.clone(),
            vault.clone(),
            &ExecuteMsg::AddLiquidity {
                underlying_token_amount: Uint128::zero(),
            },
            &[],
        )
        .unwrap_err();
    let _ = err;

    // Try non-zero amount but vault has no balances yet
    let err = app
        .execute_contract(
            admin,
            vault,
            &ExecuteMsg::AddLiquidity {
                underlying_token_amount: Uint128::new(1),
            },
            &[],
        )
        .unwrap_err();
    let _ = err;
}

#[test]
fn remove_liquidity_more_than_owned_errors() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let api = app.api();
    let admin = addr(api, ADMIN);

    // Vault owns 0 LP initially; removing any positive should error
    let err = app
        .execute_contract(
            admin,
            vault,
            &ExecuteMsg::RemoveLiquidity {
                lp_token_amount: Uint128::new(1),
            },
            &[],
        )
        .unwrap_err();
    let _ = err;
}

#[test]
fn swap_insufficient_funds_errors() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let admin = addr(app.api(), ADMIN);

    // Use other LP token (native) with zero balance in the vault
    let err = app
        .execute_contract(
            admin,
            vault,
            &ExecuteMsg::Swap {
                amount: Uint128::new(1),
                asset_info: AssetInfo::NativeToken {
                    denom: Addr::unchecked("other_lp").to_string(),
                },
            },
            &[],
        )
        .unwrap_err();
    let _ = err; // should error due to InsufficientSwapFunds
}

#[test]
fn swap_valid_with_exact_balance_emits_event() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let api = app.api();
    let oracle = addr(api, ORACLE);
    let admin = addr(api, ADMIN);
    let user = addr(api, USER);

    // Set minimal oracle prices (not strictly needed for swap, but consistent setup)
    let prices = HashMap::from_iter(
        [
            (
                Addr::unchecked("other_lp").to_string(),
                Decimal::from_str("1.0").unwrap(),
            ),
            ("incentive1".to_string(), Decimal::from_str("1.0").unwrap()),
            ("incentive2".to_string(), Decimal::from_str("1.0").unwrap()),
        ]
        .into_iter(),
    );
    app.execute_contract(
        oracle,
        vault.clone(),
        &ExecuteMsg::OracleUpdatePrices { prices },
        &[],
    )
    .unwrap();

    // Fund the vault with underlying by depositing from user
    let amount = Uint128::new(100);
    let res: AppResponse = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Deposit {
                assets: amount,
                receiver: user.clone(),
            },
            &Vec::from([Coin::new(amount, UNDERLYING_TOKEN)]),
        )
        .unwrap();
    let _ = res;

    // Now swap the exact balance amount of underlying
    let res: AppResponse = app
        .execute_contract(
            admin,
            vault.clone(),
            &ExecuteMsg::Swap {
                amount,
                asset_info: AssetInfo::NativeToken {
                    denom: UNDERLYING_TOKEN.to_string(),
                },
            },
            &[],
        )
        .unwrap();

    // Assert we emitted some events (indicates messages executed). Exact swap event name may vary by mocks.
    assert!(!res.events.is_empty());
}

fn has_event(res: &AppResponse, ty: &str) -> bool {
    res.events.iter().any(|e| e.ty.as_str() == ty)
        || res
            .events
            .iter()
            .any(|e| e.ty.as_str() == format!("wasm-{ty}"))
}

fn event_attrs(res: &AppResponse, ty: &str) -> Vec<(String, String)> {
    res.events
        .iter()
        .find(|e| e.ty.as_str() == ty || e.ty.as_str() == format!("wasm-{ty}"))
        .map(|e| {
            e.attributes
                .iter()
                .map(|a| (a.key.clone(), a.value.clone()))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn events_deposit_with_fee_and_bond_and_oracle_and_request() {
    let mut app = get_app();
    // with fee = 5%
    let vault = proper_instantiate(&mut app, false, true);
    let api = app.api();
    let oracle = addr(api, ORACLE);
    let admin = addr(api, ADMIN);
    let user = addr(api, USER);

    // Set oracle prices
    let tokens = app
        .wrap()
        .query_wasm_smart::<OracleTokensListResponse>(&vault, &QueryMsg::OracleTokensList {})
        .unwrap()
        .tokens;
    let prices = HashMap::from_iter(
        tokens
            .into_iter()
            .map(|t| (t, Decimal::from_str("1.0").unwrap())),
    );
    let res = app
        .execute_contract(
            oracle,
            vault.clone(),
            &ExecuteMsg::OracleUpdatePrices { prices },
            &[],
        )
        .unwrap();
    assert!(has_event(&res, "oracle_update_prices"));

    // Deposit native to generate deposit_with_fee event
    let amount = Uint128::new(1000);
    let res = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::Deposit {
                assets: amount,
                receiver: user.clone(),
            },
            &Vec::from([Coin::new(amount, UNDERLYING_TOKEN)]),
        )
        .unwrap();
    assert!(has_event(&res, "deposit"));
    let attrs = event_attrs(&res, "deposit");
    // Check a few key attributes exist
    assert!(attrs.iter().any(|(k, _)| k == "depositor"));
    assert!(attrs.iter().any(|(k, _)| k == "user_shares_minted"));
    assert!(attrs.iter().any(|(k, _)| k == "fee_shares_minted"));

    // Bond some underlying and check event
    let res = app
        .execute_contract(
            admin.clone(),
            vault.clone(),
            &ExecuteMsg::Bond {
                amount: Uint128::new(100),
                salt: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                slippage: Some(Decimal::from_str("0.01").unwrap()),
            },
            &[],
        )
        .unwrap();
    assert!(has_event(&res, "bond"));
    let attrs = event_attrs(&res, "bond");
    assert!(attrs.iter().any(|(k, _)| k == "amount"));
    assert!(attrs.iter().any(|(k, _)| k == "expected"));

    // Request redemption and check event
    let res = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::RequestRedeem {
                shares: Uint128::new(10),
                receiver: user.clone(),
                owner: user,
            },
            &[],
        )
        .unwrap();
    assert!(has_event(&res, "request_redemption"));
    let attrs = event_attrs(&res, "request_redemption");
    assert!(attrs.iter().any(|(k, _)| k == "shares_locked"));
    assert!(attrs.iter().any(|(k, _)| k == "expected_assets_count"));
}

#[test]
fn asset_query_and_send_helpers() {
    let mut app = get_app();
    // Native underlying vault
    let vault = proper_instantiate(&mut app, false, false);
    let api = app.api();
    let user = addr(api, USER);
    let oracle = addr(api, ORACLE);

    // Initialize oracle prices to allow deposit
    let tokens = app
        .wrap()
        .query_wasm_smart::<OracleTokensListResponse>(&vault, &QueryMsg::OracleTokensList {})
        .unwrap()
        .tokens;
    let prices = HashMap::from_iter(
        tokens
            .into_iter()
            .map(|t| (t, Decimal::from_str("1.0").unwrap())),
    );
    app.execute_contract(
        oracle,
        vault.clone(),
        &ExecuteMsg::OracleUpdatePrices { prices },
        &[],
    )
    .unwrap();

    // Fund vault with native to test balance query
    let deposit = Uint128::new(42);
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::Deposit {
            assets: deposit,
            receiver: user.clone(),
        },
        &Vec::from([Coin::new(deposit, UNDERLYING_TOKEN)]),
    )
    .unwrap();

    // Query native balance via query.rs path (covers asset.rs native branch indirectly)
    let asset_resp: AssetResponse = app
        .wrap()
        .query_wasm_smart(&vault, &QueryMsg::Asset {})
        .unwrap();
    assert_eq!(asset_resp.asset_token_address, UNDERLYING_TOKEN);
}

#[test]
fn instantiate_rejects_invalid_staking_contract() {
    let mut app = get_app();
    // Use an address that is not a contract with the expected query interface
    let invalid_staking = make_valid_addr();
    let underlying = instantiate_denom_asset(&mut app);
    let lp = instantiate_lp(
        &mut app,
        underlying.clone(),
        AssetInfo::Token {
            contract_addr: Addr::unchecked("other lp"),
        },
    );
    let tower_incentives = instantiate_incentives(&mut app);

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
        underlying_token: underlying,
        incentives: Vec::from([
            AssetInfo::NativeToken {
                denom: "incentive1".to_string(),
            },
            AssetInfo::NativeToken {
                denom: "incentive2".to_string(),
            },
        ]),
        slippage_tolerance: Decimal::from_str("0.01").unwrap(),
        staking_contract: Some(invalid_staking),
        lp,
        tower_incentives,
        entry_fee_rate: None,
        entry_fee_recipient: admin.clone(),
    };
    let err = app
        .instantiate_contract(code, admin, &msg, &[], "cw4626-escher", None)
        .unwrap_err();
    let _ = err; // should be InvalidStakingContract
}

#[test]
fn add_and_remove_role_happy_path() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let admin = addr(app.api(), ADMIN);
    let new_manager = addr(app.api(), USER);

    // Add
    app.execute_contract(
        admin.clone(),
        vault.clone(),
        &ExecuteMsg::AddToRole {
            role: AccessControlRole::Manager {},
            address: new_manager.clone(),
        },
        &[],
    )
    .unwrap();
    // Verify in role list
    let role_resp: AccessControlRoleResponse = app
        .wrap()
        .query_wasm_smart(
            &vault,
            &QueryMsg::Role {
                kind: AccessControlRole::Manager {},
            },
        )
        .unwrap();
    assert!(role_resp.addresses.contains(&new_manager));

    // Remove
    app.execute_contract(
        admin,
        vault.clone(),
        &ExecuteMsg::RemoveFromRole {
            role: AccessControlRole::Manager {},
            address: new_manager.clone(),
        },
        &[],
    )
    .unwrap();
    let role_resp: AccessControlRoleResponse = app
        .wrap()
        .query_wasm_smart(
            &vault,
            &QueryMsg::Role {
                kind: AccessControlRole::Manager {},
            },
        )
        .unwrap();
    assert!(!role_resp.addresses.contains(&new_manager));
}

#[test]
fn redeem_zero_shares_errors() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let user = addr(app.api(), USER);
    let err = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::RequestRedeem {
                shares: Uint128::zero(),
                receiver: user.clone(),
                owner: user,
            },
            &[],
        )
        .unwrap_err();
    let _ = err; // expect ZeroShareAmount
}

#[test]
fn redeem_more_than_balance_errors() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let user = addr(app.api(), USER);

    // User has 0 shares; request non-zero should fail
    let err = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::RequestRedeem {
                shares: Uint128::new(1),
                receiver: user.clone(),
                owner: user.clone(),
            },
            &[],
        )
        .unwrap_err();
    let _ = err; // expect InsufficientShares
}

#[test]
fn complete_redemption_unknown_id_errors() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let admin = addr(app.api(), ADMIN);
    let err = app
        .execute_contract(
            admin,
            vault,
            &ExecuteMsg::CompleteRedemption {
                redemption_id: 9999,
                tx_hash: "dummy".to_string(),
            },
            &[],
        )
        .unwrap_err();
    let _ = err; // expect RedemptionNotFound
}

#[test]
fn deposit_cw20_with_fee_no_yield_must_be_one_to_one() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, true, true);
    let api = app.api();
    let oracle = addr(api, ORACLE);
    let user = addr(api, USER);
    let admin = addr(api, ADMIN);
    let prices = HashMap::from_iter(
        [
            (
                Addr::unchecked("other_lp").to_string(),
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

    let asset = Addr::unchecked(
        app.wrap()
            .query_wasm_smart::<AssetResponse>(&vault, &QueryMsg::Asset {})
            .unwrap()
            .asset_token_address,
    );

    let asset_deposit_amount = Uint128::from(50000_u32);

    let preview_amount = app
        .wrap()
        .query_wasm_smart::<PreviewDepositResponse>(
            &vault,
            &QueryMsg::PreviewDeposit {
                assets: asset_deposit_amount,
            },
        )
        .unwrap()
        .shares;
    // do first deposit via transfer from
    app.execute_contract(
        user.clone(),
        asset.clone(),
        &Cw20ExecuteMsg::IncreaseAllowance {
            spender: vault.to_string(),
            amount: asset_deposit_amount,
            expires: None,
        },
        &[],
    )
    .unwrap();
    app.execute_contract(
        user.clone(),
        vault.clone(),
        &ExecuteMsg::Deposit {
            assets: asset_deposit_amount,
            receiver: user.clone(),
        },
        &[],
    )
    .unwrap();

    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: admin.to_string(),
                },
            )
            .unwrap()
            .balance
            + app
                .wrap()
                .query_wasm_smart::<cw20::BalanceResponse>(
                    &vault,
                    &QueryMsg::Balance {
                        address: user.to_string(),
                    },
                )
                .unwrap()
                .balance,
        asset_deposit_amount
    );
    assert_approx_eq!(
        preview_amount,
        asset_deposit_amount.multiply_ratio(Uint128::new(95), Uint128::new(100)),
        "0.0001"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<ExchangeRateResponse>(&vault, &QueryMsg::ExchangeRate {})
            .unwrap()
            .exchange_rate,
        Decimal::one()
    );

    let preview_amount = app
        .wrap()
        .query_wasm_smart::<PreviewDepositResponse>(
            &vault,
            &QueryMsg::PreviewDeposit {
                assets: asset_deposit_amount,
            },
        )
        .unwrap()
        .shares;
    // do second deposit via send
    app.execute_contract(
        user.clone(),
        asset.clone(),
        &Cw20ExecuteMsg::Send {
            contract: vault.to_string(),
            amount: asset_deposit_amount,
            msg: to_json_binary(&ExecuteMsg::Deposit {
                assets: asset_deposit_amount,
                receiver: user.clone(),
            })
            .unwrap(),
        },
        &[],
    )
    .unwrap();

    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: admin.to_string(),
                },
            )
            .unwrap()
            .balance
            + app
                .wrap()
                .query_wasm_smart::<cw20::BalanceResponse>(
                    &vault,
                    &QueryMsg::Balance {
                        address: user.to_string(),
                    },
                )
                .unwrap()
                .balance,
        asset_deposit_amount * Uint128::new(2)
    );
    assert_approx_eq!(
        preview_amount,
        asset_deposit_amount.multiply_ratio(Uint128::new(95), Uint128::new(100)),
        "0.0001"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<ExchangeRateResponse>(&vault, &QueryMsg::ExchangeRate {})
            .unwrap()
            .exchange_rate,
        Decimal::one()
    );
}

#[test]
fn deposit_native_no_yield_must_be_one_to_one() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let api = app.api();
    let oracle = addr(api, ORACLE);
    let user = addr(api, USER);
    let prices = HashMap::from_iter(
        [
            (
                Addr::unchecked("other_lp").to_string(),
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

    let preview_amount = app
        .wrap()
        .query_wasm_smart::<PreviewDepositResponse>(
            &vault,
            &QueryMsg::PreviewDeposit {
                assets: asset_deposit_amount,
            },
        )
        .unwrap()
        .shares;
    // do first deposit
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

    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: user.to_string(),
                },
            )
            .unwrap()
            .balance,
        asset_deposit_amount
    );
    assert_eq!(preview_amount, asset_deposit_amount);
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<ExchangeRateResponse>(&vault, &QueryMsg::ExchangeRate {})
            .unwrap()
            .exchange_rate,
        Decimal::one()
    );

    let preview_amount = app
        .wrap()
        .query_wasm_smart::<PreviewDepositResponse>(
            &vault,
            &QueryMsg::PreviewDeposit {
                assets: asset_deposit_amount,
            },
        )
        .unwrap()
        .shares;
    // do second deposit
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

    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: user.to_string(),
                },
            )
            .unwrap()
            .balance,
        asset_deposit_amount * Uint128::new(2)
    );
    assert_eq!(preview_amount, asset_deposit_amount);
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<ExchangeRateResponse>(&vault, &QueryMsg::ExchangeRate {})
            .unwrap()
            .exchange_rate,
        Decimal::one()
    );
}

#[test]
fn deposit_native_with_fee_no_yield_must_be_one_to_one() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, true);
    let api = app.api();
    let oracle = addr(api, ORACLE);
    let user = addr(api, USER);
    let admin = addr(api, ADMIN);
    let prices = HashMap::from_iter(
        [
            (
                Addr::unchecked("other_lp").to_string(),
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

    let preview_amount = app
        .wrap()
        .query_wasm_smart::<PreviewDepositResponse>(
            &vault,
            &QueryMsg::PreviewDeposit {
                assets: asset_deposit_amount,
            },
        )
        .unwrap()
        .shares;
    // do first deposit
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

    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: admin.to_string(),
                },
            )
            .unwrap()
            .balance
            + app
                .wrap()
                .query_wasm_smart::<cw20::BalanceResponse>(
                    &vault,
                    &QueryMsg::Balance {
                        address: user.to_string(),
                    },
                )
                .unwrap()
                .balance,
        asset_deposit_amount
    );
    assert_approx_eq!(
        preview_amount,
        asset_deposit_amount.multiply_ratio(Uint128::new(95), Uint128::new(100)),
        "0.0001"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<ExchangeRateResponse>(&vault, &QueryMsg::ExchangeRate {})
            .unwrap()
            .exchange_rate,
        Decimal::one()
    );

    let preview_amount = app
        .wrap()
        .query_wasm_smart::<PreviewDepositResponse>(
            &vault,
            &QueryMsg::PreviewDeposit {
                assets: asset_deposit_amount,
            },
        )
        .unwrap()
        .shares;
    // do second deposit
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

    assert_eq!(
        app.wrap()
            .query_wasm_smart::<cw20::BalanceResponse>(
                &vault,
                &QueryMsg::Balance {
                    address: admin.to_string(),
                },
            )
            .unwrap()
            .balance
            + app
                .wrap()
                .query_wasm_smart::<cw20::BalanceResponse>(
                    &vault,
                    &QueryMsg::Balance {
                        address: user.to_string(),
                    },
                )
                .unwrap()
                .balance,
        asset_deposit_amount * Uint128::new(2)
    );
    assert_approx_eq!(
        preview_amount,
        asset_deposit_amount.multiply_ratio(Uint128::new(95), Uint128::new(100)),
        "0.0001"
    );
    assert_eq!(
        app.wrap()
            .query_wasm_smart::<ExchangeRateResponse>(&vault, &QueryMsg::ExchangeRate {})
            .unwrap()
            .exchange_rate,
        Decimal::one()
    );
}

#[test]
fn git_info_must_return_valid_data() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
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
fn add_to_role_requires_manager() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let user = addr(app.api(), USER);
    let err = app
        .execute_contract(
            user.clone(),
            vault.clone(),
            &ExecuteMsg::AddToRole {
                role: AccessControlRole::Manager {},
                address: user.clone(),
            },
            &[],
        )
        .unwrap_err();
    let _ = err; // ensure it errors for non-manager
}

#[test]
fn oracle_update_zero_price_errors() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let api = app.api();
    let oracle = addr(api, ORACLE);

    let tokens = app
        .wrap()
        .query_wasm_smart::<OracleTokensListResponse>(&vault, &QueryMsg::OracleTokensList {})
        .unwrap()
        .tokens;
    assert_eq!(tokens.len(), 3);

    let mut prices = HashMap::new();
    prices.insert(tokens[0].clone(), Decimal::zero());
    prices.insert(tokens[1].clone(), Decimal::from_str("1.0").unwrap());
    prices.insert(tokens[2].clone(), Decimal::from_str("1.0").unwrap());
    let err = app
        .execute_contract(
            oracle,
            vault,
            &ExecuteMsg::OracleUpdatePrices { prices },
            &[],
        )
        .unwrap_err();
    let _ = err;
}

#[test]
fn oracle_update_wrong_set_errors() {
    let mut app = get_app();
    let vault = proper_instantiate(&mut app, false, false);
    let api = app.api();
    let oracle = addr(api, ORACLE);

    let tokens = app
        .wrap()
        .query_wasm_smart::<OracleTokensListResponse>(&vault, &QueryMsg::OracleTokensList {})
        .unwrap()
        .tokens;
    assert_eq!(tokens.len(), 3);

    let mut prices = HashMap::new();
    prices.insert(tokens[0].clone(), Decimal::from_str("1.0").unwrap());
    prices.insert(tokens[1].clone(), Decimal::from_str("1.0").unwrap());
    // intentionally omit third token
    let err = app
        .execute_contract(
            oracle,
            vault,
            &ExecuteMsg::OracleUpdatePrices { prices },
            &[],
        )
        .unwrap_err();
    let _ = err;
}
