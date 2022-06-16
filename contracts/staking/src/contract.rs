use shadeswap_shared::msg::staking::{{InitMsg, QueryMsg,QueryResponse,  HandleMsg}};
use shadeswap_shared::msg::amm_pair::HandleMsg as AmmPairHandleMsg;

use crate::state::{{Config, ClaimRewardsInfo, store_config, load_claim_reward_timestamp,  store_claim_reward_timestamp,
    get_total_staking_amount, load_stakers, load_config, is_address_already_staker, store_claim_reward_info,
    store_staker, load_staker_info, store_staker_info, remove_staker, StakingInfo, load_claim_reward_info}};   
use std::time::{SystemTime, UNIX_EPOCH};
use shadeswap_shared::admin::{{store_admin, apply_admin_guard}};
use shadeswap_shared::{ 
    fadroma::{
        scrt::{
            from_binary, log, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Decimal, Env,
            Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryRequest, QueryResult,
            StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery, 
            secret_toolkit::snip20,        
        },
        scrt_uint256::Uint256,
        scrt_callback::Callback,
        scrt_link::ContractLink,
        scrt_vk::ViewingKey,
    }
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let config = Config {
        contract_owner: env.message.sender.clone(),
        daily_reward_amount: msg.staking_amount,
        reward_token: msg.reward_token.clone()
    };
    store_config(deps, &config)?;
    store_admin(deps, &env.message.sender.clone())?;
    let mut messages = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.contract.address.clone(),
        callback_code_hash: msg.contract.code_hash.clone(),
        msg: to_binary(&AmmPairHandleMsg::SetStakingContract{ contract: ContractLink {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone()
        }})?,
        send: vec![],
    }));

    Ok(InitResponse {
        messages: messages,
        log: vec![
           log("staking_contract_addr", env.contract.address),
           log("reward_token", msg.reward_token.clone()),
           log("daily_reward_amount", msg.staking_amount),
        ],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Stake {         
            amount,
            from,
        } => {
            return stake(deps,env, amount,from)
        },
        HandleMsg::ClaimRewards { } => {
            claim_rewards(deps, env)
        }
        HandleMsg::Unstake {address} => unstake(deps,env, address),
    }    
}

// This should be callback from Snip20 Receiver
// needs to check for the amount
pub fn stake<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    from: HumanAddr
) -> StdResult<HandleResponse>{
    apply_admin_guard(env.message.sender.clone(), &deps.storage)?;
    let current_timestamp = Uint128(env.block.time * 1000 as u128);
    claim_rewards_for_all_stakers(deps, current_timestamp)?;
    let caller = from.clone();
    // check if caller exist
    let is_staker = is_address_already_staker(&deps, caller.clone())?;   
    if is_staker == true {
        let mut stake_info = load_staker_info(deps, caller.clone())?;
        stake_info.amount += amount;
        stake_info.last_time_updated = current_timestamp;        
        store_staker_info(deps, &stake_info)?;
    }
    else{
        store_staker(deps, caller.clone())?;
        store_staker_info(deps, &StakingInfo{
            staker: caller.clone(),
            amount: amount,
            last_time_updated: current_timestamp,
        })?;
    }

    // store zero for claim rewards
    store_claim_reward_info(deps, &ClaimRewardsInfo{
        staker: caller.clone(),
        amount: Uint128(0u128),
        last_time_claimed: current_timestamp,
    })?;

    // return response
    Ok(HandleResponse {
        messages: vec![],
        log: vec![
                log("action", "stake"),
                log("staker", caller.as_str()),
                log("amount", amount),
        ],
        data: None,
    })
}

pub fn claim_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env
) -> StdResult<HandleResponse>{

    let receiver = env.message.sender.clone();
    let is_user_staker = is_address_already_staker(deps, receiver.clone())?;
    if is_user_staker != true {
        return Err(StdError::unauthorized())
    }
    let current_timestamp =  Uint128((env.block.time * 1000) as u128); 
    let mut messages = Vec::new();
    // calculate for all also for user
    claim_rewards_for_all_stakers(deps, current_timestamp)?;
    let mut claim_info = load_claim_reward_info(deps, receiver.clone())?;
    let claim_amount = claim_info.amount;
    claim_info.amount = Uint128(0u128);
    claim_info.last_time_claimed =  current_timestamp;
    store_claim_reward_info(deps, &claim_info)?;   
    let config = load_config(deps)?;
    // send the message
    messages.push(config.reward_token.create_send_msg(
        env.contract.address.clone(),
        receiver.clone(),
        claim_amount,
    )?);    
   
    Ok(HandleResponse {
        messages: messages,
        log: vec![
                log("action", "claim_rewards"),
                log("caller", receiver.as_str().clone()),
                log("reward_amount",claim_amount),
        ],
        data: None,
    })
}

// Total Available Rewards = Daily_Rewards / 24*60*60*1000 * (current_date_time - last_calculated_date_time).miliseconds()
// User Incremental Rewards = Total Available Rewards * Staked Percentage
// User Total Rewards = User Owed Rewards + (User Incremental Rewards)
pub fn claim_rewards_for_all_stakers<S:Storage, A:Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    current_timestamp: Uint128
) -> StdResult<()> {
    let stakers = load_stakers(deps)?;
    let last_timestamp = load_claim_reward_timestamp(deps)?;    
    for staker in stakers.into_iter() {
        let mut claim_info = load_claim_reward_info(deps, staker.clone())?;
        let staking_reward = calculate_staking_reward(deps, staker.clone(),last_timestamp, current_timestamp)?;
        claim_info.amount += staking_reward;
        claim_info.last_time_claimed = current_timestamp;
        store_claim_reward_info(deps, &claim_info)?;
    }
    store_claim_reward_timestamp(deps, current_timestamp )?;
    Ok(())
}

pub fn calculate_staking_reward<S:Storage, A:Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    staker: HumanAddr,
    last_timestamp: Uint128,
    current_timestamp: Uint128
) -> StdResult<Uint128>{
    let cons = Uint128(100u128);
    let percentage = get_staking_percentage(deps,staker, cons)?;
    let config = load_config(deps)?;
    let milisec = Uint128(24u128 * 60u128 *60u128 * 1000u128); 
    let time_dif = (current_timestamp - last_timestamp)?;           
    if time_dif != Uint128(0u128) {        
        let total_available_reward = config.daily_reward_amount.multiply_ratio(time_dif, milisec);
        let result = total_available_reward.multiply_ratio(percentage, cons);
        Ok(result)
    }else{
        Ok(Uint128(0u128))
    }
   
}

pub fn get_staking_percentage<S:Storage, A:Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    staker: HumanAddr,
    cons: Uint128
) -> StdResult<Uint128> {
    let total_staking = Uint256::from(get_total_staking_amount(deps)?);
    let stake_info = load_staker_info(&deps, staker)?;
    let stake_amount = Uint256::from(stake_info.amount);   
    let percentage =((stake_amount * Uint256::from(cons))? / total_staking)?;    
    Ok(Uint128(percentage.clamp_u128()?))
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::GetStakers{ } => {get_all_stakers(deps)},
        QueryMsg::GetClaimReward{time,staker} =>{get_claim_reward_for_user(deps, staker, time)},
        QueryMsg::GetContractOwner {} => {get_staking_contract_owner(deps)},
    }
}

pub fn get_staking_contract_owner<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
)-> StdResult<Binary> {
    let config = load_config(&deps)?;
    to_binary(&QueryResponse::ContractOwner { address: config.contract_owner})
}

pub fn get_claim_reward_for_user<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, 
    staker: HumanAddr,
    time: u128
)-> StdResult<Binary> {
    let unpaid_claim = load_claim_reward_info(deps, staker.clone())?;
    let last_claim_timestamp = load_claim_reward_timestamp(deps)?;   
    let current_timestamp = Uint128(time); 
    let current_claim = calculate_staking_reward(deps,
        staker.clone(), last_claim_timestamp, current_timestamp)?;
    let total_claim = unpaid_claim.amount + current_claim;
    to_binary(&QueryResponse::ClaimReward{amount: total_claim})
}

pub fn get_all_stakers<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary>{
    let stakers = load_stakers(deps)?;   
    println!("get_all_stakers {}",stakers.len()); 
    to_binary(&QueryResponse::Stakers{stakers: stakers}) 
}

pub fn get_current_timestamp()-> StdResult<Uint128> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    Ok(Uint128(since_the_epoch.as_millis()))
}


pub fn unstake<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: HumanAddr,
) -> StdResult<HandleResponse>{
    apply_admin_guard(env.message.sender.clone(), &deps.storage)?;
    let caller = address;
    let current_timestamp = Uint128((env.block.time * 1000) as u128);
    let is_user_staker = is_address_already_staker(deps, caller.clone())?;
    let config = load_config(deps)?;
    if is_user_staker != true {
        return Err(StdError::unauthorized())
    }
    // claim rewards
    claim_rewards_for_all_stakers(deps, current_timestamp)?;
    // remove staker
    remove_staker(deps, caller.clone())?;
    let mut messages = Vec::new();
    // update stake_info
    let mut staker_info = load_staker_info(deps, caller.clone())?;        
    staker_info.amount = Uint128(0);
    staker_info.last_time_updated = current_timestamp;
    store_staker_info(deps, &staker_info)?;

    // send reward if any and 
    let mut claim_reward = load_claim_reward_info(deps, caller.clone())?;
    // send all remaing reward token
    messages.push(config.reward_token.create_send_msg(
        env.contract.address.clone(),
        caller.clone(),
        claim_reward.amount,
    )?); 

    // update claim  reward for staker
    claim_reward.amount = Uint128(0);
    claim_reward.last_time_claimed = current_timestamp;
    store_claim_reward_info(deps, &ClaimRewardsInfo{
        staker: caller.clone(),
        amount: Uint128(0),
        last_time_claimed: current_timestamp
    })?;
  
    Ok(HandleResponse {
        messages: messages,
        log: vec![
                log("action", "unstake"),
                log("staker", caller.as_str()),
        ],
        data: None,
    })
}