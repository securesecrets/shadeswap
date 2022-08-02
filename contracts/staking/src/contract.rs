use shadeswap_shared::{msg::staking::{InitMsg, InvokeMsg ,QueryMsg,QueryResponse,  HandleMsg}};
use shadeswap_shared::msg::amm_pair::HandleMsg as AmmPairHandleMsg;
use shadeswap_shared::{msg::amm_pair::InvokeMsg as AmmPairInvokeMsg, token_type::{{TokenType}}};
use crate::state::{{Config, ClaimRewardsInfo, store_config, load_claim_reward_timestamp,  store_claim_reward_timestamp,
    get_total_staking_amount, load_stakers, load_config, is_address_already_staker, store_claim_reward_info,
    store_staker, load_staker_info, store_staker_info, store_staker_vk, load_staker_vk, remove_staker, StakingInfo, 
    load_claim_reward_info, load_prgn_seed, store_prgn_seed}};   
use std::{time::{SystemTime, UNIX_EPOCH}, env};
use shadeswap_shared::admin::{{store_admin, apply_admin_guard}};
use cosmwasm_std::{{HandleResponse, Uint128, to_binary}};
;;

use shadeswap_shared::{
    fadroma::prelude::ContractInfo
};

pub const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let config = Config {
        contract_owner: env.message.sender.clone(),
        daily_reward_amount: msg.staking_amount,
        reward_token: msg.reward_token.clone(),
        lp_token: ContractLink { 
            address: HumanAddr::default(),
            code_hash: "".to_string()
        },
        staking_contract: ContractLink { 
            address: env.contract.address.clone(), 
            code_hash: env.contract_code_hash.clone()
        }
    };
    store_config(deps, &config)?;
    store_admin(deps, &env.message.sender.clone())?;
    store_prgn_seed(deps, &msg.prng_seed.as_slice().to_vec())?;
    let mut messages = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.pair_contract.address.clone(),
        callback_code_hash: msg.pair_contract.code_hash.clone(),
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
        HandleMsg::Receive {
            from, amount, msg, ..
        } => receiver_callback(deps, env, from, amount, msg),      
        HandleMsg::ClaimRewards { } => {
            claim_rewards(deps, env)
        }
        HandleMsg::SetLPToken {lp_token} => set_lp_token(deps, env, lp_token),
        HandleMsg::Unstake {amount, remove_liqudity} => unstake(deps,env, amount, remove_liqudity),
        HandleMsg::SetVKForStaker { key} => set_view_key(deps, env, key),
    }    
}

fn receiver_callback<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    let msg = msg.ok_or_else(|| {
        StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
    })?;

    let config = load_config(deps)?;
    match from_binary(&msg)? {       
        InvokeMsg::Stake { from, amount } => {
            if config.lp_token.address != env.message.sender {
                return Err(StdError::unauthorized());
            }
            stake(deps, env, amount, from)
        }
    }
}

// This should be callback from Snip20 Receiver
// needs to check for the amount

pub fn set_view_key<S: Storage, A: Api, Q: Querier>(
  deps: &mut Extern<S, A, Q>,
  env: Env,
  key: String,
) -> StdResult<HandleResponse>{    
    let caller =  env.message.sender.clone();
    let is_staker = is_address_already_staker(&deps, caller.clone())?;  
    if is_staker == false {
        return Err(StdError::unauthorized());
    }
    let mut staker_info = load_staker_info(&deps, caller.clone())?;
    let prgn_seed = load_prgn_seed(&deps)?;
    let staker_vk = ViewingKey(key);
    store_staker_vk(deps, env.message.sender.clone() ,staker_vk)?;    
    store_staker_info(deps, &staker_info); 
    Ok(HandleResponse {
        messages: vec![],
        log: vec![
                log("action", "set_view_key"),
                log("staker", caller.to_string()),
        ],
        data: None,
    })
}

pub fn stake<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    from: HumanAddr
) -> StdResult<HandleResponse>{
    // this is receiver for LP Token send to staking contract -> 
    let config = load_config(deps)?;
    if config.lp_token.address != env.message.sender {
        return Err(StdError::unauthorized());
    }
    let current_timestamp = Uint128((env.block.time * 1000) as u128);
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
            last_time_updated: current_timestamp
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
        let staking_reward = calculate_staking_reward(deps, staker.clone(), last_timestamp, current_timestamp)?;
        claim_info.amount += staking_reward;
        claim_info.last_time_claimed = current_timestamp;
        store_claim_reward_info(deps, &claim_info)?;
    }
    store_claim_reward_timestamp(deps, current_timestamp)?;
    Ok(())
}

pub fn set_lp_token<S:Storage, A:Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    lp_token: ContractLink<HumanAddr>
) -> StdResult<HandleResponse>{
    let mut config = load_config(deps)?;

    if config.lp_token.address != HumanAddr::default()
    {
        return Err(StdError::GenericErr { msg: "LP Token has already been added.".to_string(), backtrace: None });
    }
    config.lp_token = lp_token.clone();   
    let mut messages = Vec::new();
    // register pair contract for LP receiver
    messages.push(snip20::register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        BLOCK_SIZE,
        lp_token.code_hash.clone(),
        lp_token.address.clone(),
    )?);      
  
    //store lp_token
    store_config(deps, &config)?;
    Ok(HandleResponse {
        messages: messages,
        log: vec![
                log("action", "set_lp_token"),               
        ],
        data: None,
    })
}

pub fn calculate_staking_reward<S:Storage, A:Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    staker: HumanAddr,
    last_timestamp: Uint128,
    current_timestamp: Uint128
) -> StdResult<Uint128>{
    let cons = Uint128(100u128);
    let percentage = get_staking_percentage(deps,staker, cons)?;
    let percentage = Uint128(50);
    let config = load_config(deps)?;
    let seconds = Uint128(24u128 * 60u128 *60u128 *1000u128); 
    if last_timestamp < current_timestamp {
        let time_dif = (current_timestamp - last_timestamp)?;
        let total_available_reward = config.daily_reward_amount.multiply_ratio(time_dif, seconds);
        let result = total_available_reward.multiply_ratio(percentage, cons);
        Ok(result)
    }
    else{
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
        QueryMsg::GetClaimReward{ staker, time, key  } =>{get_claim_reward_for_user(deps, staker, key,time)},
        QueryMsg::GetContractOwner {} => {get_staking_contract_owner(deps)},
        QueryMsg::GetStakerLpTokenInfo { key, staker } => {get_staking_stake_lp_token_info(deps, staker, key)},
        QueryMsg::GetRewardTokenBalance {key, address} => {get_staking_reward_token_balance(deps, key, address)},
        QueryMsg::GetStakerRewardTokenBalance { key, staker } => {get_staker_reward_info(deps, key, staker)},
    }
}

fn get_staker_reward_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    viewing_key: String,
    staker: HumanAddr
) -> StdResult<Binary>{
    let config = load_config(deps)?;
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = config.reward_token.clone()
    {
        let reward_token_info = ContractLink{
            address: contract_addr.clone(),
            code_hash: token_code_hash.clone(), 
        };
        let staking_contract_address = config.staking_contract;
        let reward_token_balance =  config.reward_token.query_balance(&deps.querier,staker.clone() , viewing_key.to_string())?;
        let total_reward_token_balance = query_total_reward_liquidity(&deps.querier, &reward_token_info)?;
        let response_msg = QueryResponse::StakerRewardTokenBalance { reward_amount: reward_token_balance, total_reward_liquidity: total_reward_token_balance };
        return to_binary(&response_msg)
    }else{
        return Err(StdError::generic_err("Invalid reward token"))
    }   
}

fn get_staking_reward_token_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    viewing_key: String,
    address: HumanAddr,
) -> StdResult<Binary>{
    let config = load_config(deps)?;
    let staking_contract_address = config.staking_contract;
    let reward_token_balance = config.reward_token.query_balance(&deps.querier,  address.clone(), viewing_key.to_string())?;
    let response_msg = QueryResponse::RewardTokenBalance { amount: reward_token_balance };
    to_binary(&response_msg)
}

fn get_staking_stake_lp_token_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    staker: HumanAddr,
    key: String
) -> StdResult<Binary>{
    let is_staker = is_address_already_staker(&deps, staker.clone())?;  
    if is_staker == false {
        return Err(StdError::unauthorized());
    }

    let staker_info = load_staker_info(&deps, staker.clone())?;
    let staker_vk = load_staker_vk(&deps, staker.clone())?;   
    let viewing_key = ViewingKey(key.clone()); 
    if  viewing_key.check_viewing_key(&staker_vk) != true{
        return Err(StdError::unauthorized());
    } 
    let response_msg = QueryResponse::StakerLpTokenInfo { 
        staked_lp_token: staker_info.amount, 
        total_staked_lp_token: get_total_staking_amount(deps)?,
    };
    to_binary(&response_msg)
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
    key: String,
    time: Uint128
)-> StdResult<Binary> {
    // load stakers    
    let is_staker = is_address_already_staker(&deps, staker.clone())?;  
    if is_staker == false {
        return Err(StdError::unauthorized());
    }
    let staker_info = load_staker_info(&deps, staker.clone())?;
    let staker_vk = load_staker_vk(&deps, staker.clone())?;   
    let viewing_key = ViewingKey(key.clone()); 
    if  viewing_key.check_viewing_key(&staker_vk) != true{
        return Err(StdError::unauthorized());
    } 
    let unpaid_claim = load_claim_reward_info(deps, staker.clone())?;
    let last_claim_timestamp = load_claim_reward_timestamp(deps)?;     
    let current_timestamp = time; 
    let current_claim = calculate_staking_reward(deps,
         staker.clone(), last_claim_timestamp, current_timestamp)?;
    let total_claim = unpaid_claim.amount + current_claim;
    println!("{:?}", total_claim);
    to_binary(&QueryResponse::ClaimReward{amount: total_claim})
}

pub fn unstake<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,   
    amount: Uint128,
    remove_liqudity: Option<bool>
) -> StdResult<HandleResponse>{
    let caller = env.message.sender.clone();
    let current_timestamp = Uint128((env.block.time * 1000) as u128);
    let is_user_staker = is_address_already_staker(deps, caller.clone())?;
    let config = load_config(deps)?;
    if is_user_staker != true {
        return Err(StdError::unauthorized())
    }
    
    // claim rewards
    claim_rewards_for_all_stakers(deps, current_timestamp)?;
    // remove staker
   
    let mut messages = Vec::new();
    // update stake_info
    let mut staker_info = load_staker_info(deps, caller.clone())?;        
    // check if the amount is higher than the current staking amount
    if amount > staker_info.amount {
       // return Err(StdError::GenericErr{ msg: "Staking Amount is higher then actual staking amount".to_string(), backtrace: None})
    }
    // if amount is the same as current staking amount remove staker from list
    let diff_amount = (staker_info.amount - amount)?;
    if  diff_amount == Uint128(0) {
        remove_staker(deps, caller.clone())?;
    }

    staker_info.amount = (staker_info.amount - amount)?;
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

    // send back amount of lp token to pair contract to send pair token back with burn
    // TODO send LP token to user add option either to remove liqudity or just remove from staking
    let config = load_config(deps)?;

    if let Some(true) = remove_liqudity {
        // SEND LP Token back to Pair Contract With Remove Liquidity
        let remove_liquidity_msg = to_binary(&AmmPairInvokeMsg::RemoveLiquidity { 
            from: Some(caller.clone())}).unwrap();      
       
        let msg = to_binary(&snip20::HandleMsg::Send {
            recipient: config.contract_owner.clone(),
            amount: amount,
            msg: Some(remove_liquidity_msg.clone()),
            padding: None,
        })?;
    
        messages.push(
            WasmMsg::Execute {
                contract_addr:  config.lp_token.address.clone(),
                callback_code_hash: config.lp_token.code_hash.clone(),
                msg,
                send: vec![],
            }
            .into(),
        );
    }
    else{
        // SEND LP Token back to Staker And User Will Manually Remove Liquidity
        let msg = to_binary(&snip20::HandleMsg::Send {
            recipient: caller.clone(),
            amount: amount,
            msg: None,
            padding: None,
        })?;
    
        messages.push(
            WasmMsg::Execute {
                contract_addr:  config.lp_token.address.clone(),
                callback_code_hash: config.lp_token.code_hash.clone(),
                msg,
                send: vec![],
            }
            .into(),
        );
    }        
  
    Ok(HandleResponse {
        messages: messages,
        log: vec![
                log("action", "unstake"),
                log("amount", amount),
                log("staker", caller.as_str()),
        ],
        data: None,
    })
}

pub fn create_viewing_key(seed: String) -> ViewingKey {
    ViewingKey(seed.to_string())
}

fn query_total_reward_liquidity(
    querier: &impl Querier,
    reward_token_info: &ContractLink<HumanAddr>,
) -> StdResult<Uint128> {
    let result = snip20::token_info_query(
        querier,
        BLOCK_SIZE,
        reward_token_info.code_hash.clone(),
        reward_token_info.address.clone(),
    )?;

    //If this happens, the LP token has been incorrectly configured
    if result.total_supply.is_none() {
       unreachable!("Reward token has no available supply.");
    }

    Ok(result.total_supply.unwrap())
}


pub fn get_current_timestamp()-> StdResult<Uint128> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    Ok(Uint128(since_the_epoch.as_millis()))
}