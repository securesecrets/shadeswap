use cosmwasm_std::{Addr, Uint128, Storage};
use cosmwasm_storage::{singleton, Singleton, ReadonlySingleton, singleton_read, bucket_read, bucket, ReadonlyBucket, Bucket};
use serde::{Serialize, Deserialize};
use shadeswap_shared::{core::{TokenType, ContractLink, ViewingKey}, Contract};


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
pub static REWARD_TOKEN_INFO: &[u8] = b"REWARD_TOKEN_INFO";
pub static REWARD_TOKEN_LIST: &[u8] = b"REWARD_TOKEN_LIST";
pub static PROXY_STAKE: &[u8] = b"PROXY_STAKE";

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct Config {
    pub amm_pair: Addr,
    pub daily_reward_amount: Uint128,
    pub reward_token: TokenType,
    pub lp_token: ContractLink,
    pub authenticator: Option<Contract>
}

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct StakingInfo{
    pub amount: Uint128,
    pub proxy_staked: Uint128,
    pub last_time_updated: Uint128,
}

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct ProxyStakingInfo{
    pub amount: Uint128
}

#[derive(Serialize, Deserialize, Clone,  PartialEq, Debug)]
pub struct RewardTokenInfo{
    pub reward_token: ContractLink,
    pub daily_reward_amount: Uint128,
    pub valid_to: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RewardTokenInfoList{
    pub list_tokens: Vec<Addr>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ClaimRewardsInfo{
    pub amount: Uint128,
    pub reward_token_addr: Addr,
    pub reward_token_code_hash: String
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

pub fn proxy_staker_info_w(storage: &mut dyn Storage) -> Bucket<ProxyStakingInfo> {
    bucket(storage, PROXY_STAKE)
}

pub fn proxy_staker_info_r(storage: &dyn Storage) -> ReadonlyBucket<ProxyStakingInfo> {
    bucket_read(storage, PROXY_STAKE)
}


pub fn claim_reward_info_w(storage: &mut dyn Storage) -> Bucket<Vec<ClaimRewardsInfo>> {
    bucket(storage, CLAIM_REWARDS)
}

pub fn claim_reward_info_r(storage: &dyn Storage) -> ReadonlyBucket<Vec<ClaimRewardsInfo>> {
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

pub fn reward_token_w(storage: &mut dyn Storage) -> Bucket<RewardTokenInfo> {
    bucket(storage, REWARD_TOKEN_INFO)
}

pub fn reward_token_r(storage: &dyn Storage) -> ReadonlyBucket<RewardTokenInfo> {
    bucket_read(storage, REWARD_TOKEN_INFO)
}

pub fn reward_token_list_w(storage: &mut dyn Storage) -> Singleton<Vec<Addr>> {
    singleton(storage, REWARD_TOKEN_LIST)
}

pub fn reward_token_list_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<Addr>> {
    singleton_read(storage, REWARD_TOKEN_LIST)
}
