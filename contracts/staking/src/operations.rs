// This should be callback from Snip20 Receiver
// needs to check for the amount
use std::ops::Add;

const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

use cosmwasm_std::{
    to_binary, Addr, Attribute, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, Storage, Decimal, wasm_execute, CosmosMsg, WasmMsg,
};
use cosmwasm_std::{Binary, QuerierWrapper};
use shadeswap_shared::stake_contract::ClaimableInfo;
use snafu::Backtrace;
use shadeswap_shared::core::{TokenType, apply_admin_guard};
use shadeswap_shared::snip20;
use shadeswap_shared::snip20::helpers::token_info;
use shadeswap_shared::staking::{QueryResponse, ExecuteMsg};
use shadeswap_shared::{
    core::{ContractLink, ViewingKey},
    msg::amm_pair::InvokeMsg as AmmPairInvokeMsg,
    Contract,
};

use crate::state::{
    claim_reward_info_r, claim_reward_info_w, config_r, config_w, stakers_r, stakers_vk_r,
    stakers_vk_w, stakers_w, total_staked_r, ClaimRewardsInfo, StakingInfo, total_staked_w,
    total_stakers_r, total_stakers_w, staker_index_w, staker_index_r, last_reward_time_claimed_w,
    Config, reward_token_r, reward_token_w, RewardTokenInfo, reward_token_list_r, last_reward_time_r, reward_token_list_w,
};

pub fn calculate_staker_shares(
    storage: &dyn Storage,
    amount: Uint128
) -> StdResult<Decimal>
{
    let total_staking_amount: Uint128 = match total_staked_r(storage).may_load().unwrap() {
        Some(staking_amount) => staking_amount,
        None => Uint128::zero(),
    };   
    if total_staking_amount.is_zero() {
        return Ok(Decimal::one())
    }

    let user_share = Decimal::from_ratio(amount,total_staking_amount);
    Ok(user_share)
}

pub fn store_init_reward_token_and_timestamp(
    storage: &mut dyn Storage, 
    reward_token: ContractLink, 
    emission_amount: Uint128, 
    current_timestamp: Uint128
) -> StdResult<()>
{
    // store reward token to the list
    let mut reward_token_list: Vec<Addr> = Vec::new();
    reward_token_list.push(reward_token.address.to_owned());
    reward_token_list_w(storage).save(&reward_token_list).unwrap();
    reward_token_w(storage).save(
        &reward_token.address.as_bytes(),
        &RewardTokenInfo{ 
            reward_token: reward_token.to_owned(), 
            amount: emission_amount, 
            valid_to: Uint128::new(3747905010u128)
    });       
    last_reward_time_claimed_w(storage).save(&current_timestamp).unwrap();
    Ok(())
}

pub fn set_reward_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    reward_token: ContractLink,
    amount: Uint128,
    valid_to: Uint128,
) -> StdResult<Response>{
    apply_admin_guard(&info.sender, deps.storage).unwrap();
    let reward_token_info: RewardTokenInfo = RewardTokenInfo {
        amount: amount,
        reward_token: reward_token.to_owned(),
        valid_to: valid_to
    };
    let mut reward_list_token = reward_token_list_r(deps.storage).load().unwrap();
    let result = reward_list_token.iter().find(|&x| x.to_owned() == reward_token.address.to_owned());   
    if result == None {
        reward_list_token.push(reward_token.address.to_owned());
    }
    reward_token_w(deps.storage).save(&reward_token.address.as_bytes(), &reward_token_info).unwrap();    

    Ok(Response::new().add_attributes(vec![
        Attribute::new("action", "set_reward_token"),
        Attribute::new("owner", info.sender.to_string()),
        Attribute::new("amount", amount.to_string()),
        Attribute::new("valid_to", valid_to.to_string()),
    ]))
}

pub fn stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    from: Addr,
) -> StdResult<Response> {
    // this is receiver for LP Token send to staking contract ->
    let config = config_r(deps.storage).load()?;
    if config.lp_token.address != info.sender {
        return Err(StdError::generic_err(
            "Token sent is not LP Token".to_string(),
        ));
    }

    // check if this is first time staking
    let mut stakers_count = get_total_stakers_count(deps.storage);
    // calculate staking for existing stakers without increasing amount    
    let current_timestamp = Uint128::new((env.block.time.seconds() * 1000) as u128);
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;

    // set the new total stake amount
    let mut total_stake_amount = match total_staked_r(deps.storage).may_load().unwrap() {
        Some(total_amount) => total_amount,
        None => Uint128::zero(),
    };

    total_stake_amount += amount;
    total_staked_w(deps.storage).save(&total_stake_amount)?;
   
    let caller = from.to_owned();
    // check if caller exist
    match stakers_r(deps.storage).may_load(caller.as_bytes()).unwrap(){
        Some(mut stake_info) => {
            stake_info.amount += amount;
            stake_info.last_time_updated = current_timestamp;
            stakers_w(deps.storage).save(caller.as_bytes(), &stake_info)?;
        },
        None => {
            stakers_w(deps.storage).save(
                caller.as_bytes(),
                &StakingInfo {
                    staker: caller.clone(),
                    amount: amount,
                    last_time_updated: current_timestamp,                 
                },
            )?;
            
            staker_index_w(deps.storage).save(&stakers_count.u128().to_be_bytes(), &caller.to_owned())?;   
            stakers_count += Uint128::one();
            total_stakers_w(deps.storage).save(&stakers_count)?; 
        }
    }
    
    // return response
    Ok(Response::new().add_attributes(vec![
        Attribute::new("action", "stake"),
        Attribute::new("staker", caller.as_str()),
        Attribute::new("amount", amount),
    ]))
}

pub fn get_total_stakers_count(
    storage: &dyn Storage
) -> Uint128 
{    
    return match total_stakers_r(storage).may_load().unwrap(){
        Some(count) => count,
        None => Uint128::zero()
    };
}

pub fn claim_rewards(deps: DepsMut, info: MessageInfo, env: Env) -> StdResult<Response> {
    let receiver = info.sender.clone();
    let mut list_claimable_rewards = claim_reward_info_r(deps.storage).load(&receiver.as_bytes())?;
    let current_timestamp = Uint128::new((env.block.time.seconds() * 1000) as u128);
    let mut messages: Vec<CosmosMsg> = Vec::new();

    // calculate for all also for user
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;
    process_all_claimable_rewards(deps.storage, receiver.to_string(), current_timestamp, &mut messages)?; 

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        Attribute::new("action", "claim_rewards"),
        Attribute::new("caller", receiver.as_str().clone()),
        // Attribute::new("reward_amount", claim_amount),
    ]))
}

fn process_all_claimable_rewards(storage: &mut dyn Storage, receiver: String, current_timestamp: Uint128, messages: &mut Vec<CosmosMsg>) -> StdResult<()> {   
    let mut claim_reward_tokens = claim_reward_info_r(storage).load(receiver.as_bytes())?;    
    for claim_reward in claim_reward_tokens.iter_mut(){
        // send all remaing reward token 
        let msg = snip20::ExecuteMsg::Send {
            recipient: receiver.to_owned(),
            recipient_code_hash: None,
            amount: claim_reward.amount,
            msg: None,
            memo: None,
            padding: None,
        };
    
        let coms_msg =  wasm_execute(
            claim_reward.reward_token_addr.to_owned(),
            claim_reward.reward_token_code_hash.to_owned(),
            &msg,
            vec![]
        ).unwrap().into();

        messages.push(coms_msg);
        claim_reward.amount = Uint128::zero();
        claim_reward.last_time_claimed = current_timestamp;
        // index += 1;
    }
    claim_reward_info_w(storage).save(receiver.as_bytes(),&claim_reward_tokens)?;
    Ok(())
}

// Total Available Rewards = Daily_Rewards / 24*60*60*1000 * (current_date_time - last_calculated_date_time).miliseconds()
// User Incremental Rewards = Total Available Rewards * Staked Percentage
// User Total Rewards = User Owed Rewards + (User Incremental Rewards)
// storage: &mut dyn Storage,
pub fn claim_rewards_for_all_stakers(storage: &mut dyn Storage, current_timestamp: Uint128) -> StdResult<()> {
    // TO DO FIX THISclaim_rewards
    let stakers_count = get_total_stakers_count(storage);   
    let last_claimed_timestamp = last_reward_time_r(storage).load().unwrap();
    for i in 0..stakers_count.u128() {
        // load staker address        
        let staker_address: Addr = staker_index_r(storage).load(&i.to_be_bytes())?;   
        let staker_info = match stakers_r(storage).may_load(staker_address.as_bytes()).unwrap(){
            Some(staking_info) => staking_info,
            None =>  StakingInfo{ amount: Uint128::zero(), staker: staker_address.to_owned(), last_time_updated: Uint128::zero() }
        };   
      
        let staker_share = calculate_staker_shares(storage, staker_info.amount)?; 
        let reward_token_list: Vec<RewardTokenInfo> = get_actual_reward_tokens(storage, current_timestamp)?;       
        for reward_token in reward_token_list.iter(){
            // calculate reward amount for each reward token
           
            let reward = calculate_staking_reward(staker_info.amount,staker_share, last_claimed_timestamp, current_timestamp, reward_token.amount).unwrap();
            // load any existing claimable reward for specfi
            save_claimable_amount_staker_by_reward_token(storage,reward,staker_address.to_owned(),reward_token.reward_token.to_owned(), current_timestamp).unwrap();    
        }
    }
    last_reward_time_claimed_w(storage).save(&current_timestamp)?;
    Ok(())
}


pub fn get_actual_reward_tokens(storage: &dyn Storage, current_timestamp: Uint128) -> StdResult<Vec<RewardTokenInfo>> {   
    let mut list_token: Vec<RewardTokenInfo> = Vec::new();
    let reward_list = reward_token_list_r(storage).load().unwrap();      
    for addr in &reward_list{
        // load total reward token
        let reward_token: RewardTokenInfo = reward_token_r(storage).load(addr.as_bytes()).unwrap();
        println!(" current timestamp {} valid to {}",current_timestamp,reward_token.valid_to );
        if current_timestamp <= reward_token.valid_to {
           list_token.push(reward_token.to_owned())
        }
    }
    Ok(list_token)
}

pub fn get_all_claimable_reward_for_staker(storage: &dyn Storage, staker_address: Addr) -> StdResult<Vec<ClaimRewardsInfo>>{
    let claim_info = match claim_reward_info_r(storage).may_load(staker_address.as_bytes()).unwrap(){
        Some(claim_reward_info) => claim_reward_info,
        None => Vec::new()
    };
    Ok(claim_info)
}

pub fn find_claimable_reward_for_staker_by_reward_token(storage: &dyn Storage, staker_address: Addr, reward_token: ContractLink) -> StdResult<ClaimRewardsInfo>{
    let all_claimable_reward = get_all_claimable_reward_for_staker(storage, staker_address).unwrap();
    let result =  match all_claimable_reward.iter().find(|&x| x.reward_token_addr == reward_token.address.to_owned()){
        Some(clm) => clm.to_owned(),
        None => ClaimRewardsInfo { 
            amount: Uint128::zero(), 
            last_time_claimed: Uint128::zero(), 
            reward_token_addr: reward_token.address.to_owned(), 
            reward_token_code_hash: reward_token.code_hash.to_owned()
        },
    };
    Ok(result)
}

pub fn find_claimable_reward_index_for_staker(storage: &dyn Storage, staker_address: Addr, reward_token: ContractLink) -> StdResult<Option<usize>>{
    let all_claimable_reward = get_all_claimable_reward_for_staker(storage, staker_address).unwrap();
    return Ok(all_claimable_reward.iter().position(|x| x.reward_token_addr == reward_token.address))
}


pub fn save_claimable_amount_staker_by_reward_token(
    storage: &mut dyn Storage, 
    amount: Uint128, 
    staker_address: Addr, 
    reward_token: ContractLink,
    timestamp: Uint128
) -> StdResult<()>{   
    let mut list_claimable_reward = get_all_claimable_reward_for_staker(storage, staker_address.to_owned()).unwrap();
    let claimable_reward_index = find_claimable_reward_index_for_staker(storage, staker_address.to_owned(), reward_token.to_owned()).unwrap();
    let mut claimable_reward = find_claimable_reward_for_staker_by_reward_token(storage, staker_address.to_owned(), reward_token.to_owned()).unwrap();    
    match claimable_reward_index{
        Some(index) => {
            list_claimable_reward[index].amount += amount;
            list_claimable_reward[index].last_time_claimed = timestamp;
        },
        None => {
            claimable_reward.amount += amount;
            claimable_reward.last_time_claimed = timestamp;  
            list_claimable_reward.push(claimable_reward.to_owned());
        },
    }       
    claim_reward_info_w(storage).save(staker_address.as_bytes(),&list_claimable_reward)?; 
    Ok(())
}

pub fn calculate_staking_reward(   
    amount: Uint128,
    percentage: Decimal,
    last_timestamp: Uint128,
    current_timestamp: Uint128,
    emmision_rate: Uint128
) -> StdResult<Uint128> {      
    let seconds = Uint128::new(24u128 * 60u128 * 60u128 * 1000u128);   
    if last_timestamp < current_timestamp || amount > Uint128::zero() {
        let time_dif = current_timestamp - last_timestamp;
        let total_available_reward = emmision_rate.multiply_ratio(time_dif, seconds);
        let converted_total_reward = Decimal::from_atomics(total_available_reward, 0).unwrap();  
        let result = converted_total_reward.checked_mul(percentage)?;
        Ok(result.atomics().checked_div(DECIMAL_FRACTIONAL)?)
    } else {
        Ok(Uint128::zero())
    }
}

pub fn get_config(deps: Deps) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = config.reward_token.clone()
    {
        let response = QueryResponse::Config {
            reward_token: ContractLink {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
            lp_token: config.lp_token.clone(),
            daily_reward_amount: config.daily_reward_amount.clone(),
            contract_owner: config.contract_owner.clone(),
        };
        return to_binary(&response);
    } else {
        return Err(StdError::generic_err("Invalid reward token"));
    }
}

pub fn get_staking_stake_lp_token_info(deps: Deps, staker: Addr) -> StdResult<Binary> {
    let staker_info = stakers_r(deps.storage).load(&staker.as_bytes())?;
    let response_msg = QueryResponse::StakerLpTokenInfo {
        staked_lp_token: staker_info.amount,
        total_staked_lp_token: total_staked_r(deps.storage).load()?,
    };
    to_binary(&response_msg)
}

pub fn get_claim_reward_for_user(
    deps: Deps,
    staker: Addr,
    time: Uint128,
) -> StdResult<Binary> {
    // load stakers
    let config = config_r(deps.storage).load()?;
    let reward_token_info = match config.reward_token.clone() {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => ContractLink {
            address: contract_addr.clone(),
            code_hash: token_code_hash,
        },
        TokenType::NativeToken { denom:_ } => ContractLink {
            address: Addr::unchecked("".to_string()),
            code_hash: "".to_string(),
        },
    };
   
    let mut result_list: Vec<ClaimableInfo> = Vec::new();
    let staker_info = stakers_r(deps.storage).load(staker.as_bytes())?;
    let unpaid_claim_list = claim_reward_info_r(deps.storage).load(staker.as_bytes())?;
    let reward_token_list: Vec<RewardTokenInfo> = get_actual_reward_tokens(deps.storage, time)?;
    let percentage = calculate_staker_shares(deps.storage, staker_info.amount)?; 
    for reward_token in reward_token_list.iter(){
        // calculate reward amount for each reward token
        let reward = calculate_staking_reward(staker_info.amount, percentage,staker_info.last_time_updated,time, reward_token.amount).unwrap();
        // load any existing claimable reward for specif user
        let claimable_reward = find_claimable_reward_for_staker_by_reward_token(deps.storage, staker.to_owned(), reward_token.reward_token.to_owned()).unwrap();
        result_list.push(ClaimableInfo{
            token_address: reward_token.reward_token.address.to_owned(),
            amount: claimable_reward.amount + reward,
        });
    }
    to_binary(&QueryResponse::ClaimRewards { 
        claimable_rewards:result_list})

}

pub fn unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    remove_liqudity: Option<bool>,
) -> StdResult<Response> {
    let caller = info.sender.clone();
    let current_timestamp = Uint128::new((env.block.time.seconds() * 1000) as u128);
    let mut staker_info = stakers_r(deps.storage).load(caller.as_bytes())?;   
    let config = config_r(deps.storage).load()?;   
    // claim rewards
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;
    // remove staker
    let mut messages:Vec<CosmosMsg> = Vec::new();
    // check if the amount is higher than the current staking amount
    if amount > staker_info.amount {
        return Err(StdError::generic_err("Staking Amount is higher then actual staking amount".to_string()));
    }
    // if amount is the same as current staking amount remove staker from list
    let diff_amount = staker_info.amount - amount;
    if diff_amount.is_zero() {
        stakers_w(deps.storage).remove(caller.as_bytes());
    } else {
        staker_info.amount = staker_info.amount - amount;
        staker_info.last_time_updated = current_timestamp;
        stakers_w(deps.storage).save(caller.as_bytes(), &staker_info)?;
    }

    process_all_claimable_rewards(deps.storage,caller.to_string(), current_timestamp, &mut messages)?; 
    // send back amount of lp token to pair contract to send pair token back with burn
    // TODO send LP token to user add option either to remove liqudity or just remove from staking
    let config = config_r(deps.storage).load()?;

    if let Some(true) = remove_liqudity {
        // SEND LP Token back to Pair Contract With Remove Liquidity
        let remove_liquidity_msg = to_binary(&AmmPairInvokeMsg::RemoveLiquidity {
            from: Some(caller.clone()),
        })
        .unwrap();
        let msg = snip20::ExecuteMsg::Send {
            recipient: config.contract_owner.to_string(),
            recipient_code_hash: None,
            amount: amount,
            msg: Some(remove_liquidity_msg.clone()),
            memo: None,
            padding: None,
        };
        
        let coms_msg =  wasm_execute(
            config.lp_token.address.to_string(),
            config.lp_token.code_hash.clone(),
            &msg,
            vec![]
        ).unwrap().into();

        messages.push(coms_msg);
    } else {
        // SEND LP Token back to Staker And User Will Manually Remove Liquidity
        let msg = snip20::ExecuteMsg::Transfer {
            recipient: caller.to_string(),
            amount: amount,
            memo: None,
            padding: None,
        };

        let coms_msg =  wasm_execute(
            config.lp_token.address.to_string(),
            config.lp_token.code_hash.clone(),
            &msg,
            vec![]
        ).unwrap().into();

        messages.push(coms_msg);
    }
    Ok(Response::new().add_messages(messages).add_attributes(vec![
        Attribute::new("action", "unstake"),
        Attribute::new("amount", amount),
        Attribute::new("staker", caller.as_str()),
    ]))
}

fn query_total_reward_liquidity(
    querier: &QuerierWrapper,
    reward_token_info: &ContractLink,
) -> StdResult<Uint128> {
    let result = token_info(
        querier,
        &Contract {
            address: reward_token_info.address.clone(),
            code_hash: reward_token_info.code_hash.clone(),
        },
    )?;

    //If this happens, the LP token has been incorrectly configured
    if result.total_supply.is_none() {
        StdError::GenericErr{msg: "Reward token has no available supply.".to_string() };
    }

    Ok(result.total_supply.unwrap())
}

pub fn create_send_msg(
    recipient: String,
    amount: Uint128,
    token_link: ContractLink
) -> StdResult<CosmosMsg> {
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_link.address.to_string(),
        code_hash: token_link.code_hash.to_owned(),
        msg: to_binary(&snip20::ExecuteMsg::Send {
            recipient,
            amount,
            padding: None,
            msg: None,
            recipient_code_hash: None,
            memo: None,
        })?,
        funds: vec![],
    });
    
    Ok(msg)
}


pub fn is_address_already_staker(deps: Deps, address: Addr) -> StdResult<bool> {
    let addrs = stakers_r(deps.storage).may_load(address.as_bytes()).unwrap();
    match addrs {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
