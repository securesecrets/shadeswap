// This should be callback from Snip20 Receiver
// needs to check for the amount
use std::ops::Add;

const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

use cosmwasm_std::{
    to_binary, Addr, Attribute, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, Storage, Decimal, wasm_execute,
};
use cosmwasm_std::{Binary, QuerierWrapper};
use shadeswap_shared::core::TokenType;
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
    stakers_vk_w, stakers_w, total_staked_r, ClaimRewardsInfo, StakingInfo, total_staked_w, total_stakers_r, total_stakers_w, staker_index_w, staker_index_r, last_reward_time_claimed_w, Config,
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
            // store zero for claim rewards     
            claim_reward_info_w(deps.storage).save(
                caller.as_bytes(),
                &ClaimRewardsInfo {
                    amount: Uint128::zero(),
                    last_time_claimed: current_timestamp,
                },
            )?;
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
    let mut claim_info = claim_reward_info_r(deps.storage).load(&receiver.as_bytes())?;
    let current_timestamp = Uint128::new((env.block.time.seconds() * 1000) as u128);
    let mut messages = Vec::new();
    // calculate for all also for user
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;    
    let claim_amount = claim_info.amount;
    claim_info.amount = Uint128::zero();
    claim_info.last_time_claimed = current_timestamp;
    claim_reward_info_w(deps.storage).save(receiver.as_bytes(), &claim_info)?;
    let config = config_r(deps.storage).load()?;
    // send the message
    messages.push(config.reward_token.create_send_msg(
        env.contract.address.to_string(),
        receiver.to_string(),
        claim_amount,
    )?);

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        Attribute::new("action", "claim_rewards"),
        Attribute::new("caller", receiver.as_str().clone()),
        Attribute::new("reward_amount", claim_amount),
    ]))
}

// Total Available Rewards = Daily_Rewards / 24*60*60*1000 * (current_date_time - last_calculated_date_time).miliseconds()
// User Incremental Rewards = Total Available Rewards * Staked Percentage
// User Total Rewards = User Owed Rewards + (User Incremental Rewards)
// storage: &mut dyn Storage,
pub fn claim_rewards_for_all_stakers(storage: &mut dyn Storage, current_timestamp: Uint128) -> StdResult<()> {
    // TO DO FIX THISclaim_rewards
    let stakers_count = get_total_stakers_count(storage);   
    for i in 0..stakers_count.u128() {
        // load staker address        
        let staker_address: Addr = staker_index_r(storage).load(&i.to_be_bytes())?;   
        let staker_info = match stakers_r(storage).may_load(staker_address.as_bytes()).unwrap(){
            Some(staking_info) => staking_info,
            None =>  StakingInfo{ amount: Uint128::zero(), staker: staker_address.to_owned(), last_time_updated: Uint128::zero() }
        };               
        // if staker_info.amount.is_zero() {
                  
        // }    
        
        let reward = calculate_staking_reward(storage,staker_info.amount, staker_info.last_time_updated,current_timestamp)?;
        let mut claim_info = match claim_reward_info_r(storage).may_load(staker_address.as_bytes()).unwrap(){
            Some(claim_reward_info) => claim_reward_info,
            None => ClaimRewardsInfo{ amount: Uint128::zero(), last_time_claimed: Uint128::zero() }
        };

        claim_info.amount += reward;
        claim_info.last_time_claimed = current_timestamp;
        claim_reward_info_w(storage).save(staker_address.as_bytes(),&claim_info)?; 
    }
    last_reward_time_claimed_w(storage).save(&current_timestamp)?;
    Ok(())
}

// pub fn set_lp_token(deps: DepsMut, env: Env, lp_token: ContractLink) -> StdResult<Response> {
//     let mut config = config_r(deps.storage).load()?;

//     if config.lp_token.address != Addr::unchecked("".to_string()) {
//         return Err(StdError::generic_err(
//             "LP Token has already been added.".to_string(),
//         ));
//     }
//     config.lp_token = lp_token.clone();
//     let mut messages = Vec::new();
//     // register pair contract for LP receiver
//     messages.push(register_receive(
//         env.contract.code_hash.clone(),
//         None,
//         &Contract {
//             address: lp_token.address.clone(),
//             code_hash: lp_token.code_hash.clone(),
//         },
//     )?);

//     //store lp_token
//     config_w(deps.storage).save(&config)?;
//     Ok(Response::new().add_attributes(vec![Attribute::new("action", "set_lp_token")]))
// }

pub fn calculate_staking_reward(
    storage: &dyn Storage,
    amount: Uint128,
    last_timestamp: Uint128,
    current_timestamp: Uint128,
) -> StdResult<Uint128> {
    let percentage = calculate_staker_shares(storage, amount)?;
    let config: Config = config_r(storage).load()?;
    let seconds = Uint128::new(24u128 * 60u128 * 60u128 * 1000u128);   
    if last_timestamp < current_timestamp || amount > Uint128::zero() {
        let time_dif = current_timestamp - last_timestamp;
        let total_available_reward = config.daily_reward_amount.multiply_ratio(time_dif, seconds);
        let converted_total_reward = Decimal::from_atomics(total_available_reward, 0).unwrap();  
        let result = converted_total_reward.checked_mul(percentage)?;
        Ok(result.atomics().checked_div(DECIMAL_FRACTIONAL)?)
    } else {
        Ok(Uint128::zero())
    }
}


pub fn get_staker_reward_info(deps: Deps, viewing_key: String, staker: Addr) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = config.reward_token.clone()
    {
        let reward_token_info = ContractLink {
            address: contract_addr.clone(),
            code_hash: token_code_hash.clone(),
        };
        let reward_token_balance = config.reward_token.query_balance(
            deps,
            staker.to_string(),
            viewing_key.to_string(),
        )?;
        let total_reward_token_balance =
            query_total_reward_liquidity(&deps.querier, &reward_token_info)?;
        let response_msg = QueryResponse::StakerRewardTokenBalance {
            reward_amount: reward_token_balance,
            total_reward_liquidity: total_reward_token_balance,
            reward_token: ContractLink {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
        };
        return to_binary(&response_msg);
    } else {
        return Err(StdError::generic_err("Invalid reward token"));
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

    let staker_info = stakers_r(deps.storage).load(staker.as_bytes())?;
    let unpaid_claim = claim_reward_info_r(deps.storage).load(staker.as_bytes())?;
    let last_claim_timestamp = unpaid_claim.last_time_claimed;
    let current_timestamp = time;
    let current_claim = calculate_staking_reward(
        deps.storage,
        staker_info.amount,
        last_claim_timestamp,
        current_timestamp,
    )?;
    let total_claim = unpaid_claim.amount + current_claim;   
    to_binary(&QueryResponse::ClaimReward {
        amount: total_claim,
        reward_token: reward_token_info,
    })
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
    let mut messages = Vec::new();
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

    // send reward if any and
    let mut claim_reward = claim_reward_info_r(deps.storage).load(caller.as_bytes())?;
    // send all remaing reward token
    messages.push(config.reward_token.create_send_msg(
        env.contract.address.to_string(),
        caller.to_string(),
        claim_reward.amount,
    )?);

    // update claim  reward for staker
    claim_reward.amount = Uint128::zero();
    claim_reward.last_time_claimed = current_timestamp;
    claim_reward_info_w(deps.storage).save(
        caller.as_bytes(),
        &ClaimRewardsInfo {
            amount: Uint128::zero(),
            last_time_claimed: current_timestamp,
        },
    )?;

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


pub fn is_address_already_staker(deps: Deps, address: Addr) -> StdResult<bool> {
    let addrs = stakers_r(deps.storage).may_load(address.as_bytes()).unwrap();
    match addrs {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
