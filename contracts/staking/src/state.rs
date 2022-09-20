use cosmwasm_std::{Addr, Uint128, Storage, Decimal256};
use cosmwasm_storage::{singleton, Singleton, ReadonlySingleton, singleton_read, bucket_read, bucket, ReadonlyBucket, Bucket};
use serde::{Serialize, Deserialize};
use shadeswap_shared::core::{TokenType, ContractLink, ViewingKey};



pub static CONFIG: &[u8] = b"STAKING_CONFIG";
pub static STAKERS: &[u8] = b"LIST_STAKERS";
pub static STAKING_INFO: &[u8] = b"STAKING_INFO";
pub static CLAIM_REWARDS: &[u8] = b"CLAIM_REWARDS";
pub static LAST_REWARD_TIME_CLAIMED: &[u8] = b"LAST_REWARD_TIME_CLAIMED";
pub static PGRN_SEED: &[u8] = b"PGRN_SEED";
pub static STAKER_VK: &[u8] = b"STAKER_VK";
pub static TOTAL_STAKERS: &[u8] = b"TOTAL_STAKERS";
pub static TOTAL_STAKED: &[u8] = b"TOTAL_STAKED";
pub static STAKER_INDEX: &[u8] = b"STAKER_INDEX";

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct Config {
    pub contract_owner: Addr,
    pub daily_reward_amount: Uint128,
    pub reward_token: TokenType,
    pub lp_token: ContractLink
}

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct StakingInfo{
    pub staker: Addr,
    pub amount: Uint128,
    pub last_time_updated: Uint128,
}

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct ClaimRewardsInfo{
    pub amount: Uint128,
    pub last_time_claimed: Uint128
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
    bucket(storage, CLAIM_REWARDS)
}

pub fn stakers_vk_r(storage: &dyn Storage) -> ReadonlyBucket<ViewingKey> {
    bucket_read(storage, CLAIM_REWARDS)
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


