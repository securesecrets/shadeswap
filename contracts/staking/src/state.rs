use cosmwasm_std::{Addr, Decimal256, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use serde::{Deserialize, Serialize};
use shadeswap_shared::{
    core::{ContractLink, TokenType, ViewingKey},
    Contract,
};

pub static CONFIG: &[u8] = b"CONFIG";
pub static STAKERS: &[u8] = b"LIST_STAKERS";
pub static STAKING_INFO: &[u8] = b"STAKING_INFO";
pub static CLAIM_REWARDS: &[u8] = b"CLAIM_REWARDS";
pub static LAST_REWARD_TIME_CLAIMED: &[u8] = b"LAST_REWARD_TIME_CLAIMED";
pub static PGRN_SEED: &[u8] = b"PGRN_SEED";
pub static STAKER_VK: &[u8] = b"STAKER_VK";
pub static TOTAL_STAKERS: &[u8] = b"TOTAL_STAKERS";
pub static TOTAL_STAKED: &[u8] = b"TOTAL_STAKED";
pub static STAKER_INDEX: &[u8] = b"STAKER_INDEX";
/// Whitelisted contracts that can stake and unstake on behalf of users while maintaining custody of the LP tokens
pub static WHITELISTED_PROXY_STAKERS: &[u8] = b"WHITELISTED_PROXY_STAKERS";

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    pub contract_owner: Addr,
    pub daily_reward_amount: Uint128,
    pub reward_token: TokenType,
    pub lp_token: ContractLink,
    pub authenticator: Option<Contract>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct StakingInfo {
    pub staker: Addr,
    pub amount: Uint128,
    pub last_time_updated: Uint128,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ClaimRewardsInfo {
    pub amount: Uint128,
    pub last_time_claimed: Uint128,
}

pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG)
}

pub fn stakers_w(storage: &mut dyn Storage) -> Bucket<StakingInfo> {
    bucket(storage, STAKERS)
}

pub fn stakers_r(storage: &dyn Storage) -> ReadonlyBucket<StakingInfo> {
    bucket_read(storage, STAKERS)
}

pub fn last_reward_time_claimed_w(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, LAST_REWARD_TIME_CLAIMED)
}

pub fn last_reward_time_r(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, LAST_REWARD_TIME_CLAIMED)
}

pub fn claim_reward_info_w(storage: &mut dyn Storage) -> Bucket<ClaimRewardsInfo> {
    bucket(storage, CLAIM_REWARDS)
}

pub fn claim_reward_info_r(storage: &dyn Storage) -> ReadonlyBucket<ClaimRewardsInfo> {
    bucket_read(storage, CLAIM_REWARDS)
}

pub fn stakers_vk_w(storage: &mut dyn Storage) -> Bucket<ViewingKey> {
    bucket(storage, STAKER_VK)
}

pub fn stakers_vk_r(storage: &dyn Storage) -> ReadonlyBucket<ViewingKey> {
    bucket_read(storage, STAKER_VK)
}

pub fn prng_seed_w(storage: &mut dyn Storage) -> Singleton<Vec<u8>> {
    singleton(storage, PGRN_SEED)
}

pub fn prng_seed_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<u8>> {
    singleton_read(storage, PGRN_SEED)
}

pub fn total_stakers_w(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, TOTAL_STAKERS)
}

pub fn total_stakers_r(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, TOTAL_STAKERS)
}

pub fn staker_index_w(storage: &mut dyn Storage) -> Bucket<Addr> {
    bucket(storage, STAKER_INDEX)
}

pub fn staker_index_r(storage: &dyn Storage) -> ReadonlyBucket<Addr> {
    bucket_read(storage, STAKER_INDEX)
}

pub fn total_staked_w(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, TOTAL_STAKED)
}

pub fn total_staked_r(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, TOTAL_STAKED)
}

pub fn whitelisted_proxy_stakers_w(storage: &mut dyn Storage) -> Singleton<Vec<Addr>> {
    singleton(storage, WHITELISTED_PROXY_STAKERS)
}

pub fn whitelisted_proxy_stakers_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<Addr>> {
    singleton_read(storage, WHITELISTED_PROXY_STAKERS)
}
