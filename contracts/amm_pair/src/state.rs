use cosmwasm_std::{Addr, Binary, Storage};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use serde::{Deserialize, Serialize};
use shadeswap_shared::{
    core::{CustomFee, TokenPair, ViewingKey},
    msg::amm_pair::TradeHistory,
    staking::StakingContractInit,
    Contract,
};

pub const PAGINATION_LIMIT: u8 = 30;
pub static CONFIG: &[u8] = b"config";
pub static TRADE_COUNT: &[u8] = b"tradecount";
pub static TRADE_HISTORY: &[u8] = b"trade_history";
pub static WHITELIST: &[u8] = b"whitelist";
pub const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub factory_contract: Option<Contract>,
    pub lp_token: Contract,
    pub staking_contract: Option<Contract>,
    pub arbitrage_contract: Option<Contract>,
    pub pair: TokenPair,
    pub viewing_key: ViewingKey,
    pub custom_fee: Option<CustomFee>,
    pub staking_contract_init: Option<StakingContractInit>,
    pub prng_seed: Binary,
    pub admin_auth: Contract,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum DirectionType {
    Buy,
    Sell,
    Unknown,
}

pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG)
}

pub fn trade_count_w(storage: &mut dyn Storage) -> Singleton<u64> {
    singleton(storage, TRADE_COUNT)
}

pub fn trade_count_r(storage: &dyn Storage) -> ReadonlySingleton<u64> {
    singleton_read(storage, TRADE_COUNT)
}

pub fn whitelist_w(storage: &mut dyn Storage) -> Singleton<Vec<Addr>> {
    singleton(storage, WHITELIST)
}

pub fn whitelist_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<Addr>> {
    singleton_read(storage, WHITELIST)
}

pub fn trade_history_w(storage: &mut dyn Storage) -> Bucket<TradeHistory> {
    bucket(storage, TRADE_HISTORY)
}

pub fn trade_history_r(storage: &dyn Storage) -> ReadonlyBucket<TradeHistory> {
    bucket_read(storage, TRADE_HISTORY)
}
