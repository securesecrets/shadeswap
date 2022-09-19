// This should be callback from Snip20 Receiver
// needs to check for the amount

use std::time::{SystemTime, UNIX_EPOCH};

use cosmwasm_std::{
    to_binary, Addr, Attribute, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, Storage,
};
use cosmwasm_std::{Binary, QuerierWrapper, WasmMsg};
use shadeswap_shared::core::TokenType;
use shadeswap_shared::snip20;
use shadeswap_shared::snip20::helpers::token_info;
use shadeswap_shared::staking::QueryResponse;
use shadeswap_shared::{
    core::{ContractLink, ViewingKey},
    msg::amm_pair::InvokeMsg as AmmPairInvokeMsg,
    snip20::helpers::register_receive,
    Contract,
};

use crate::state::{
    claim_reward_info_r, claim_reward_info_w, config_r, config_w, stakers_r, stakers_vk_r,
    stakers_vk_w, stakers_w, total_staked_r, ClaimRewardsInfo, StakingInfo,
};

pub fn set_view_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    key: String,
) -> StdResult<Response> {
    let caller = info.sender.clone();
    let staker_vk = ViewingKey(key);
    stakers_vk_w(deps.storage).save(caller.as_bytes(), &staker_vk);
    Ok(Response::new().add_attributes(vec![
        Attribute::new("action", "set_view_key"),
        Attribute::new("staker", caller.to_string()),
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
    let current_timestamp = Uint128::from((env.block.time.seconds() * 1000) as u128);
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;
    let caller = from.clone();
    // check if caller exist
    let is_staker = is_address_already_staker(deps.as_ref(), caller.clone())?;
    if is_staker == true {
        let mut stake_info = stakers_r(deps.storage).load(caller.as_bytes())?;
        stake_info.amount += amount;
        stake_info.last_time_updated = current_timestamp;
        stakers_w(deps.storage).save(caller.as_bytes(), &stake_info)?;
    } else {
        stakers_w(deps.storage).save(
            caller.as_bytes(),
            &StakingInfo {
                staker: caller.clone(),
                amount: amount,
                last_time_updated: current_timestamp,
            },
        )?;
    }

    // store zero for claim rewards
    claim_reward_info_w(deps.storage).save(
        caller.as_bytes(),
        &ClaimRewardsInfo {
            amount: Uint128::from(0u128),
            last_time_claimed: current_timestamp,
        },
    )?;

    // return response
    Ok(Response::new().add_attributes(vec![
        Attribute::new("action", "stake"),
        Attribute::new("staker", caller.as_str()),
        Attribute::new("amount", amount),
    ]))
}

pub fn claim_rewards(deps: DepsMut, info: MessageInfo, env: Env) -> StdResult<Response> {
    let receiver = info.sender.clone();
    let is_user_staker = is_address_already_staker(deps.as_ref(), receiver.clone())?;
    if is_user_staker != true {
        return Err(StdError::generic_err("".to_string()));
    }
    let current_timestamp = Uint128::from((env.block.time.seconds() * 1000) as u128);
    let mut messages = Vec::new();
    // calculate for all also for user
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;
    let mut claim_info = claim_reward_info_r(deps.storage).load(&receiver.as_bytes())?;
    let claim_amount = claim_info.amount;
    claim_info.amount = Uint128::from(0u128);
    claim_info.last_time_claimed = current_timestamp;
    claim_reward_info_w(deps.storage).save(receiver.as_bytes(), &claim_info)?;
    let config = config_r(deps.storage).load()?;
    // send the message
    messages.push(config.reward_token.create_send_msg(
        env.contract.address.to_string(),
        receiver.to_string(),
        claim_amount,
    )?);

    Ok(Response::new().add_attributes(vec![
        Attribute::new("action", "claim_rewards"),
        Attribute::new("caller", receiver.as_str().clone()),
        Attribute::new("reward_amount", claim_amount),
    ]))
}

// Total Available Rewards = Daily_Rewards / 24*60*60*1000 * (current_date_time - last_calculated_date_time).miliseconds()
// User Incremental Rewards = Total Available Rewards * Staked Percentage
// User Total Rewards = User Owed Rewards + (User Incremental Rewards)
pub fn claim_rewards_for_all_stakers(storage: &dyn Storage, current_timestamp: Uint128) -> StdResult<()> {
    // TO DO FIX THIS
    /*let stakers = stakers_r(deps.storage).load()?;
    let last_timestamp = claim(deps)?;
    for staker in stakers.into_iter() {
        let mut claim_info = load_claim_reward_info(deps, staker.clone())?;
        let staking_reward = calculate_staking_reward(deps, staker.clone(), last_timestamp, current_timestamp)?;
        claim_info.amount += staking_reward;
        claim_info.last_time_claimed = current_timestamp;
        store_claim_reward_info(deps, &claim_info)?;
    }
    store_claim_reward_timestamp(deps, current_timestamp)?;*/
    Ok(())
}

pub fn set_lp_token(deps: DepsMut, env: Env, lp_token: ContractLink) -> StdResult<Response> {
    let mut config = config_r(deps.storage).load()?;

    if config.lp_token.address != Addr::unchecked("".to_string()) {
        return Err(StdError::generic_err(
            "LP Token has already been added.".to_string(),
        ));
    }
    config.lp_token = lp_token.clone();
    let mut messages = Vec::new();
    // register pair contract for LP receiver
    messages.push(register_receive(
        env.contract.code_hash.clone(),
        None,
        &Contract {
            address: lp_token.address.clone(),
            code_hash: lp_token.code_hash.clone(),
        },
    )?);

    //store lp_token
    config_w(deps.storage).save(&config)?;
    Ok(Response::new().add_attributes(vec![Attribute::new("action", "set_lp_token")]))
}

pub fn calculate_staking_reward(
    deps: Deps,
    staker: Addr,
    last_timestamp: Uint128,
    current_timestamp: Uint128,
) -> StdResult<Uint128> {
    let cons = Uint128::from(100u128);
    let percentage = get_staking_percentage(deps, staker, cons)?;
    let config = config_r(deps.storage).load()?;
    let seconds = Uint128::from(24u128 * 60u128 * 60u128 * 1000u128);
    if last_timestamp < current_timestamp {
        let time_dif = (current_timestamp - last_timestamp);
        let total_available_reward = config.daily_reward_amount.multiply_ratio(time_dif, seconds);
        let result = total_available_reward.multiply_ratio(percentage, cons);
        Ok(result)
    } else {
        Ok(Uint128::from(0u128))
    }
}

pub fn get_staking_percentage(deps: Deps, staker: Addr, cons: Uint128) -> StdResult<Uint128> {
    let total_staking = total_staked_r(deps.storage).load()?;
    let stake_info = stakers_r(deps.storage).load(&staker.as_bytes())?;
    let stake_amount = stake_info.amount;
    let percentage = stake_amount.multiply_ratio(cons, total_staking);
    Ok(percentage)
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

pub fn get_staking_reward_token_balance(
    env: Env,
    deps: Deps,
    viewing_key: String,
    address: Addr,
) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = config.reward_token.clone()
    {
        let staking_contract_address = env.contract.address;
        let reward_token_balance = config.reward_token.query_balance(
            deps,
            address.to_string(),
            viewing_key.to_string(),
        )?;
        let response_msg = QueryResponse::RewardTokenBalance {
            amount: reward_token_balance,
            reward_token: ContractLink {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
        };
        to_binary(&response_msg)
    } else {
        return Err(StdError::generic_err("Invalid reward token"));
    }
}

pub fn get_staking_stake_lp_token_info(deps: Deps, staker: Addr, key: String) -> StdResult<Binary> {
    let is_staker = is_address_already_staker(deps, staker.clone())?;
    if is_staker == false {
        return Err(StdError::generic_err("".to_string()));
    }

    let staker_info = stakers_r(deps.storage).load(&staker.as_bytes())?;
    let staker_vk = stakers_vk_r(deps.storage).load(&staker.as_bytes())?;
    let viewing_key = ViewingKey(key.clone());
    if viewing_key.check_viewing_key(&staker_vk.as_bytes()) != true {
        return Err(StdError::generic_err("".to_string()));
    }
    let response_msg = QueryResponse::StakerLpTokenInfo {
        staked_lp_token: staker_info.amount,
        total_staked_lp_token: total_staked_r(deps.storage).load()?,
    };
    to_binary(&response_msg)
}

pub fn get_staking_contract_owner(deps: Deps, env: Env) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;
    to_binary(&QueryResponse::ContractOwner {
        address: env.contract.address.to_string(),
    })
}

pub fn get_claim_reward_for_user(
    deps: Deps,
    staker: Addr,
    key: String,
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
        TokenType::NativeToken { denom } => ContractLink {
            address: Addr::unchecked("".to_string()),
            code_hash: "".to_string(),
        },
    };

    let is_staker = is_address_already_staker(deps, staker.clone())?;
    if is_staker == false {
        return Err(StdError::generic_err("".to_string()));
    }
    let staker_info = stakers_r(deps.storage).load(staker.as_bytes())?;
    let staker_vk = stakers_vk_r(deps.storage).load(staker.as_bytes())?;
    let viewing_key = ViewingKey(key.clone());
    if viewing_key.check_viewing_key(&staker_vk.to_hashed()) != true {
        return Err(StdError::generic_err("".to_string()));
    }
    let unpaid_claim = claim_reward_info_r(deps.storage).load(staker.as_bytes())?;
    let last_claim_timestamp = unpaid_claim.last_time_claimed;
    let current_timestamp = time;
    let current_claim = calculate_staking_reward(
        deps,
        staker.clone(),
        last_claim_timestamp,
        current_timestamp,
    )?;
    let total_claim = unpaid_claim.amount + current_claim;
    println!("{:?}", total_claim);
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
    let current_timestamp = Uint128::from((env.block.time.seconds() * 1000) as u128);
    let is_user_staker = is_address_already_staker(deps.as_ref(), caller.clone())?;
    let config = config_r(deps.storage).load()?;
    if is_user_staker != true {
        return Err(StdError::generic_err("".to_string()));
    }
    // claim rewards
    //claim_rewards_for_all_stakers(deps, current_timestamp)?;
    // remove staker
    let mut messages = Vec::new();
    // update stake_info
    let mut staker_info = stakers_r(deps.storage).load(caller.as_bytes())?;
    // check if the amount is higher than the current staking amount
    if amount > staker_info.amount {
        // return Err(StdError::GenericErr{ msg: "Staking Amount is higher then actual staking amount".to_string(), backtrace: None})
    }
    // if amount is the same as current staking amount remove staker from list
    let diff_amount = (staker_info.amount - amount);
    if diff_amount == Uint128::zero() {
        stakers_w(deps.storage).remove(caller.as_bytes());
    } else {
        staker_info.amount = (staker_info.amount - amount);
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
            from: Some(caller.to_string()),
        })
        .unwrap();
        let msg = to_binary(&snip20::ExecuteMsg::Send {
            recipient: config.contract_owner.to_string(),
            recipient_code_hash: None,
            amount: amount,
            msg: Some(remove_liquidity_msg.clone()),
            memo: None,
            padding: None,
        })?;
        messages.push(
            WasmMsg::Execute {
                contract_addr: config.lp_token.address.to_string(),
                code_hash: config.lp_token.code_hash.clone(),
                msg,
                funds: vec![],
            }
            .into(),
        );
    } else {
        // SEND LP Token back to Staker And User Will Manually Remove Liquidity
        let msg = to_binary(&snip20::ExecuteMsg::Transfer {
            recipient: caller.to_string(),
            amount: amount,
            memo: None,
            padding: None,
        })?;
        messages.push(
            WasmMsg::Execute {
                contract_addr: config.lp_token.address.to_string(),
                code_hash: config.lp_token.code_hash.clone(),
                msg,
                funds: vec![],
            }
            .into(),
        );
    }
    Ok(Response::new().add_messages(messages).add_attributes(vec![
        Attribute::new("action", "unstake"),
        Attribute::new("amount", amount),
        Attribute::new("staker", caller.as_str()),
    ]))
}

pub fn create_viewing_key(seed: String) -> ViewingKey {
    ViewingKey(seed.to_string())
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
        unreachable!("Reward token has no available supply.");
    }

    Ok(result.total_supply.unwrap())
}

pub fn get_current_timestamp() -> StdResult<Uint128> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    Ok(Uint128::from(since_the_epoch.as_millis()))
}

pub fn is_address_already_staker(deps: Deps, address: Addr) -> StdResult<bool> {
    let addrs = stakers_r(deps.storage).may_load(address.as_bytes())?;
    match addrs {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
