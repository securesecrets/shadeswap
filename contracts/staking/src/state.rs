use std::any::type_name;
use cosmwasm_std::{HumanAddr, Uint128, StdResult, Extern, Storage, Api, Querier, StdError};
use serde::{Deserialize, Serialize};
use shadeswap_shared::{token_type::TokenType, core::ContractLink, scrt_storage::{ns_save, ns_load, save, load}, viewing_keys::{ViewingKey, VIEWING_KEY_SIZE}};
use serde::de::DeserializeOwned;
use shadeswap_shared::msg::amm_pair::{{ ExecuteMsg,TradeHistory}};
use std::fmt::{{Formatter, Display}};

pub static STAKING_CONFIG: &[u8] = b"STAKING_CONFIG";
pub static LIST_STAKERS: &[u8] = b"LIST_STAKERS";
pub static STAKING_INFO: &[u8] = b"STAKING_INFO";
pub static CLAIM_REWARDS: &[u8] = b"CLAIM_REWARDS";
pub static LAST_REWARD_TIME_CLAIMED: &[u8] = b"LAST_REWARD_TIME_CLAIMED";
pub static PGRN_SEED: &[u8] = b"PGRN_SEED";
pub static STAKER_VK: &[u8] = b"STAKER_VK";

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct Config {
    pub contract_owner: HumanAddr,
    pub daily_reward_amount: Uint128,
    pub reward_token: TokenType<HumanAddr>,
    pub lp_token: ContractLink<HumanAddr>,
    pub staking_contract: ContractLink<HumanAddr>,
}

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct StakingInfo{
    pub staker: HumanAddr,
    pub amount: Uint128,
    pub last_time_updated: Uint128   
}

#[derive(Serialize, Deserialize,  PartialEq, Debug)]
pub struct ClaimRewardsInfo{
    pub staker: HumanAddr,
    pub amount: Uint128,
    pub last_time_claimed: Uint128
}

pub fn store_config <S: Storage, A: Api, Q: Querier>(
    deps:   DepsMut,
    config: &Config
) -> StdResult<()> {
    save(&mut deps.storage, STAKING_CONFIG, &config)
}

pub fn load_config<S: Storage, A: Api, Q: Querier>(
    deps: &Deps<S, A, Q>
) -> StdResult<Config> {
    let result: Config = load(&deps.storage, STAKING_CONFIG)?.ok_or(
        StdError::generic_err("Config doesn't exist in storage.")
    )?;
    Ok(result)
}

pub fn load_stakers<S: Storage, A: Api, Q: Querier>(
    deps: &Deps<S, A, Q>
) -> StdResult<Vec<HumanAddr>> {
   let stakers = load(&deps.storage, LIST_STAKERS)?.unwrap_or(Vec::new());    
   Ok(stakers)
}

pub fn load_claim_reward_timestamp<S: Storage, A: Api, Q: Querier>(
    deps: &Deps<S, A, Q>
) -> StdResult<Uint128> {
   let timestamp = load(&deps.storage, LAST_REWARD_TIME_CLAIMED)?.unwrap_or(Uint128(0u128));    
   Ok(timestamp)
}

pub fn store_claim_reward_timestamp<S: Storage, A: Api, Q: Querier>(
    deps:   DepsMut,
    timestamp: Uint128
) -> StdResult<()> {
    save(&mut deps.storage, LAST_REWARD_TIME_CLAIMED, &timestamp)
}

pub fn store_staker<S: Storage, A: Api, Q: Querier>(
    deps:   DepsMut,
    staker: HumanAddr
) -> StdResult<()> {
    let mut unwrap_data = load_stakers(deps)?;
    unwrap_data.push(staker); 
    save(&mut deps.storage, LIST_STAKERS, &unwrap_data)
}

pub fn get_total_staking_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Deps<S, A, Q>
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
    deps: DepsMut, 
    staker: HumanAddr
) -> StdResult<()> {
    let mut addresses = load_stakers(deps)?;
    addresses.retain(|x| x != &staker);
    save(&mut deps.storage, LIST_STAKERS, &addresses)
}

pub fn store_prng_seed<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    pgrn_seed: &Vec<u8>
) -> StdResult<()> {
    save(&mut deps.storage, PGRN_SEED, &pgrn_seed)
}

pub fn load_prgn_seed<S: Storage, A: Api, Q: Querier>(
    deps: &Deps<S, A, Q>,
) -> StdResult<Vec<u8>> {
    let result = load(&deps.storage, PGRN_SEED)?
        .ok_or(StdError::generic_err( "No PGRN Seed has been setup"))?;    
    Ok(result)
}

pub fn is_address_already_staker<S: Storage, A: Api, Q: Querier>(
    deps: &Deps<S, A, Q>,
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
    deps: &Deps<S, A, Q>,
    staker: HumanAddr) -> StdResult<StakingInfo> {
    let staking_info: StakingInfo =
    ns_load(&deps.storage, STAKING_INFO, staker.as_str().as_bytes())?
        .ok_or_else(|| StdError::generic_err("Staking Info doesn't exist in storage for address"))?;
   Ok(staking_info)
}

pub fn store_staker_info<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut, 
    staker_info: &StakingInfo
) -> StdResult<()> {       
    ns_save(&mut deps.storage, STAKING_INFO, staker_info.staker.clone().as_str().as_bytes(), &staker_info)
}   

pub fn store_staker_vk<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut, 
    staker: HumanAddr,
    viewing_key: ViewingKey
) -> StdResult<()> {
    ns_save(&mut deps.storage, STAKER_VK, staker.clone().as_str().as_bytes(), &viewing_key.to_hashed())
}

pub fn load_staker_vk<S: Storage, A: Api, Q: Querier>(
    deps:   &Deps<S, A, Q>,
    staker: HumanAddr
) -> StdResult<[u8; VIEWING_KEY_SIZE]> {
    let staker_vk = ns_load(&deps.storage,STAKER_VK, staker.clone().as_str().as_bytes())?
        .ok_or(StdError::generic_err("Viewing key not setup for Query"))?;
    Ok(staker_vk)
}


pub fn load_claim_reward_info<S: Storage, A: Api, Q: Querier>(
    deps: &Deps<S, A, Q>,
    staker: HumanAddr
) -> StdResult<ClaimRewardsInfo> {
    let staking_info: ClaimRewardsInfo =
    ns_load(&deps.storage, CLAIM_REWARDS, staker.as_str().as_bytes())?
        .ok_or_else(|| StdError::generic_err("Claims Reward doesn't exist in storage for address"))?;
   Ok(staking_info)
}


pub fn store_claim_reward_info<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut, 
    claim_reward: &ClaimRewardsInfo
) -> StdResult<()> {       
    ns_save(&mut deps.storage, CLAIM_REWARDS, claim_reward.staker.clone().as_str().as_bytes(), 
    &claim_reward)
}  