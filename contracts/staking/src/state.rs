use shadeswap_shared::{
    fadroma::{
        scrt_link::{ContractLink},    
        scrt_addr::{Humanize, Canonize},
        scrt::{
            Api, CanonicalAddr, Extern, HumanAddr, Uint128,
            Querier, StdResult, Storage, StdError
        },
        scrt_storage::{load, save, ns_save, ns_load},
        scrt_vk::ViewingKey,
    },
    token_pair::TokenPair
};
use std::any::type_name;
use serde::{Deserialize, Serialize};
use shadeswap_shared::token_type::TokenType;
use serde::de::DeserializeOwned;
use shadeswap_shared::msg::amm_pair::{{ HandleMsg,TradeHistory}};
use std::fmt::{{Formatter, Display}};

pub static STAKING_CONFIG: &[u8] = b"STAKING_CONFIG";
pub static LIST_STAKERS: &[u8] = b"LIST_STAKERS";
pub static STAKING_INFO: &[u8] = b"STAKING_INFO";
pub static CLAIM_REWARDS: &[u8] = b"CLAIM_REWARDS";
pub static LAST_REWARD_TIME_CLAIMED: &[u8] = b"LAST_REWARD_TIME_CLAIMED";

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct Config {
    pub contract_owner: HumanAddr,
    pub daily_reward_amount: Uint128,
    pub reward_token: TokenType<HumanAddr>,
    pub lp_token: ContractLink<HumanAddr>,
}

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct StakingInfo{
    pub staker: HumanAddr,
    pub amount: Uint128,
    pub last_time_updated: Uint128,
    pub viewing_key: ViewingKey
}

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct ClaimRewardsInfo{
    pub staker: HumanAddr,
    pub amount: Uint128,
    pub last_time_claimed: Uint128
}

pub fn store_config <S: Storage, A: Api, Q: Querier>(
    deps:   &mut Extern<S, A, Q>,
    config: &Config
) -> StdResult<()> {
    save(&mut deps.storage, STAKING_CONFIG, &config)
}

pub fn load_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Config> {
    let result: Config = load(&deps.storage, STAKING_CONFIG)?.ok_or(
        StdError::generic_err("Config doesn't exist in storage.")
    )?;
    Ok(result)
}

pub fn load_stakers<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Vec<HumanAddr>> {
   let stakers = load(&deps.storage, LIST_STAKERS)?.unwrap_or(Vec::new());    
   Ok(stakers)
}

pub fn load_claim_reward_timestamp<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Uint128> {
   let timestamp = load(&deps.storage, LAST_REWARD_TIME_CLAIMED)?.unwrap_or(Uint128(0u128));    
   Ok(timestamp)
}

pub fn store_claim_reward_timestamp<S: Storage, A: Api, Q: Querier>(
    deps:   &mut Extern<S, A, Q>,
    timestamp: Uint128
) -> StdResult<()> {
    save(&mut deps.storage, LAST_REWARD_TIME_CLAIMED, &timestamp)
}

pub fn store_staker<S: Storage, A: Api, Q: Querier>(
    deps:   &mut Extern<S, A, Q>,
    staker: HumanAddr
) -> StdResult<()> {
    let mut unwrap_data = load_stakers(deps)?;
    unwrap_data.push(staker); 
    save(&mut deps.storage, LIST_STAKERS, &unwrap_data)
}

pub fn get_total_staking_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Uint128> {
    let stakers = load_stakers(&deps)?;
    let mut amount = Uint128(0u128);
    for staker in stakers.into_iter(){
        let stake_info = load_staker_info(deps, staker)?;
        amount += stake_info.amount;
    }
    Ok(amount)
}

    
pub fn remove_staker<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>, 
    staker: HumanAddr
) -> StdResult<()> {
    let mut addresses = load_stakers(deps)?;
    addresses.retain(|x| x != &staker);
    save(&mut deps.storage, LIST_STAKERS, &addresses)
}


pub fn is_address_already_staker<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr
) -> StdResult<bool>{
    let addrs = load_stakers(&deps)?;
    if addrs.contains(&address) {
       return Ok(true)
    } else {
       return Ok(false)
    }      
}

pub fn load_staker_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    staker: HumanAddr) -> StdResult<StakingInfo> {
    let staking_info: StakingInfo =
    ns_load(&deps.storage, STAKING_INFO, staker.as_str().as_bytes())?
        .ok_or_else(|| StdError::generic_err("Staking Info doesn't exist in storage for address"))?;
   Ok(staking_info)
}

pub fn store_staker_info<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>, 
    staker_info: &StakingInfo
) -> StdResult<()> {       
    ns_save(&mut deps.storage, STAKING_INFO, staker_info.staker.clone().as_str().as_bytes(), &staker_info)
}   

pub fn load_claim_reward_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    staker: HumanAddr
) -> StdResult<ClaimRewardsInfo> {
    let staking_info: ClaimRewardsInfo =
    ns_load(&deps.storage, CLAIM_REWARDS, staker.as_str().as_bytes())?
        .ok_or_else(|| StdError::generic_err("Claims Reward doesn't exist in storage for address"))?;
   Ok(staking_info)
}


pub fn store_claim_reward_info<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>, 
    claim_reward: &ClaimRewardsInfo
) -> StdResult<()> {       
    ns_save(&mut deps.storage, CLAIM_REWARDS, claim_reward.staker.clone().as_str().as_bytes(), 
    &claim_reward)
}  