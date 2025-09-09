use std::{collections::HashMap, fmt::Display};

use astroport::asset::{Asset, AssetInfo};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Timestamp;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
#[derive(Copy)]
pub enum AccessControlRole {
    Manager {},
    Oracle {},
}

impl Display for AccessControlRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Manager {} => "manager",
                Self::Oracle {} => "oracle",
            }
        )
    }
}

impl AccessControlRole {
    #[must_use]
    pub fn key(&self) -> String {
        self.to_string()
    }
}

#[cw_serde]
pub struct TowerConfig {
    pub tower_incentives: Addr,
    pub lp: Addr,
    pub lp_underlying_asset: AssetInfo,
    pub lp_other_asset: AssetInfo,
    pub is_underlying_first_lp_asset: bool,
    pub lp_token: Addr,
    pub lp_incentives: Vec<AssetInfo>,
    pub slippage_tolerance: Decimal,
}

pub type PricesMap = HashMap<String, Decimal>;

#[cw_serde]
pub enum RedemptionStatus {
    Pending,
    Completed(Timestamp),
}

#[cw_serde]
pub struct RedemptionRequest {
    pub id: u64,
    pub owner: Addr,
    pub receiver: Addr,
    pub shares_locked: Uint128,
    pub expected_assets: Vec<Asset>,
    pub status: RedemptionStatus,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub completion_tx_hash: Option<String>,
}

#[cw_serde]
pub struct LockedShares {
    pub total_locked: Uint128,
    pub redemption_ids: Vec<u64>,
}

#[cw_serde]
pub struct PerformanceFeeConfig {
    pub fee_rate: Decimal,             // e.g., 0.1 (10%)
    pub fee_recipient: Addr,           // Manager address
    pub initial_assets: Uint128,       // Assets at vault start (baseline)
    pub last_fee_calculation: u64,     // Block height of last fee calculation
    pub fee_calculation_interval: u64, // Blocks between fee calculations (e.g., 17280 for 24h)
    pub last_assets_snapshot: Uint128, // Assets at last fee calculation
}

#[cw_serde]
pub struct EntryFeeConfig {
    pub fee_rate: Decimal,   // Entry fee rate (e.g., 0.1 for 10%)
    pub fee_recipient: Addr, // Address to receive entry fee shares
}

#[cw_serde]
pub struct FeeCalculationResult {
    pub fee_shares: Uint128,
    pub profit_per_share: Decimal,
    pub new_hwm_pps: Decimal,
    pub calculation_block: u64,
}

#[cw_serde]
pub struct FeeInfo {
    pub amount: Uint128,     // Fee amount in underlying asset terms
    pub percentage: Decimal, // Percentage of total assets
    pub calculated_at: u64,  // Block when fee was calculated
    pub distributed: bool,   // Whether fee has been distributed
}

pub const UNDERLYING_ASSET: Item<AssetInfo> = Item::new("asset");
pub const UNDERLYING_DECIMALS: Item<u8> = Item::new("asset-decimals");
pub const ACCESS_CONTROL: Map<String, Vec<Addr>> = Map::new("access-control");
pub const TOWER_CONFIG: Item<TowerConfig> = Item::new("tower-config");
/// Prices map in terms of the underlying asset
/// NOTE: It's an Item of a `HashMap` and not a Map because it needs to be read & updated completely every time
pub const ORACLE_PRICES: Item<PricesMap> = Item::new("oracle-prices");
// Staking contract configuration
pub const STAKING_CONTRACT: Item<Addr> = Item::new("staking_contract");
// Redemption system
pub const REDEMPTION_COUNTER: Item<u64> = Item::new("redemption_counter");
pub const REDEMPTION_REQUESTS: Map<u64, RedemptionRequest> = Map::new("redemption_requests");
pub const USER_REDEMPTION_IDS: Map<Addr, Vec<u64>> = Map::new("user_redemption_ids");
// Locked shares system
pub const LOCKED_SHARES: Item<LockedShares> = Item::new("locked_shares");
// Entry fee configuration for deposits/mints
pub const ENTRY_FEE_CONFIG: Item<EntryFeeConfig> = Item::new("entry_fee_config");
