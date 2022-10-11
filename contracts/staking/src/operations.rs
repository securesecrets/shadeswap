// This should be callback from Snip20 Receiver
// needs to check for the amount
const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

use cosmwasm_std::Binary;
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Attribute, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use shadeswap_shared::core::TokenType;
use shadeswap_shared::snip20;
use shadeswap_shared::stake_contract::ClaimableInfo;
use shadeswap_shared::staking::QueryResponse;
use shadeswap_shared::{
    core::ContractLink, msg::amm_pair::InvokeMsg as AmmPairInvokeMsg, Contract,
};

use crate::state::{
    claim_reward_info_r, claim_reward_info_w, config_r, config_w, last_reward_time_claimed_w,
    last_reward_time_r, reward_token_list_r, reward_token_list_w, reward_token_r, reward_token_w,
    staker_index_r, staker_index_w, stakers_r, stakers_w, total_staked_r, total_staked_w,
    total_stakers_r, total_stakers_w, ClaimRewardsInfo, RewardTokenInfo, StakingInfo,
};

pub fn calculate_staker_shares(storage: &dyn Storage, amount: Uint128) -> StdResult<Decimal> {
    let total_staking_amount: Uint128 = match total_staked_r(storage).may_load()? {
        Some(staking_amount) => staking_amount,
        None => Uint128::zero(),
    };
    if total_staking_amount.is_zero() {
        return Ok(Decimal::one());
    }

    let user_share = Decimal::from_ratio(amount, total_staking_amount);
    Ok(user_share)
}

pub fn store_init_reward_token_and_timestamp(
    storage: &mut dyn Storage,
    reward_token: ContractLink,
    emission_amount: Uint128,
    current_timestamp: Uint128,
) -> StdResult<()> {
    // store reward token to the list
    let mut reward_token_list: Vec<Addr> = Vec::new();
    reward_token_list.push(reward_token.address.to_owned());
    reward_token_list_w(storage).save(&reward_token_list)?;
    reward_token_w(storage).save(
        &reward_token.address.as_bytes(),
        &RewardTokenInfo {
            reward_token: reward_token.to_owned(),
            daily_reward_amount: emission_amount,
            valid_to: Uint128::new(3747905010000u128),
        },
    )?;
    last_reward_time_claimed_w(storage).save(&current_timestamp)?;
    Ok(())
}

pub fn set_reward_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    reward_token: ContractLink,
    daily_reward_amount: Uint128,
    valid_to: Uint128,
) -> StdResult<Response> {
    let current_timestamp = Uint128::new((env.block.time.seconds()) as u128);
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;
    let reward_token_info: RewardTokenInfo = RewardTokenInfo {
        daily_reward_amount,
        reward_token: reward_token.to_owned(),
        valid_to,
    };
    let mut reward_list_token = reward_token_list_r(deps.storage).load()?;
    let result = reward_list_token
        .iter()
        .find(|&x| x.to_owned() == reward_token.address.to_owned());
    if result == None {
        reward_list_token.push(reward_token.address.to_owned());
    }
    reward_token_w(deps.storage).save(&reward_token.address.as_bytes(), &reward_token_info)?;

    Ok(Response::new().add_attributes(vec![
        Attribute::new("action", "set_reward_token"),
        Attribute::new("owner", info.sender.to_string()),
        Attribute::new("daily_reward_amount", daily_reward_amount.to_string()),
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

    let mut stakers_count = get_total_stakers_count(deps.storage)?;
    let current_timestamp = Uint128::new((env.block.time.seconds()) as u128);
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;

    // set the new total stake amount
    let mut total_stake_amount = match total_staked_r(deps.storage).may_load()? {
        Some(total_amount) => total_amount,
        None => Uint128::zero(),
    };

    total_stake_amount += amount;
    total_staked_w(deps.storage).save(&total_stake_amount)?;

    let caller = from.to_owned();
    // check if user has staked before
    match stakers_r(deps.storage).may_load(caller.as_bytes())? {
        Some(mut stake_info) => {
            stake_info.amount += amount;
            stake_info.last_time_updated = current_timestamp;
            stakers_w(deps.storage).save(caller.as_bytes(), &stake_info)?;
        }
        None => {
            stakers_w(deps.storage).save(
                caller.as_bytes(),
                &StakingInfo {
                    staker: caller.clone(),
                    amount: amount,
                    last_time_updated: current_timestamp,
                },
            )?;

            staker_index_w(deps.storage)
                .save(&stakers_count.u128().to_be_bytes(), &caller.to_owned())?;
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

pub fn get_total_stakers_count(storage: &dyn Storage) -> StdResult<Uint128> {
    return match total_stakers_r(storage).may_load()? {
        Some(count) => Ok(count),
        None => Ok(Uint128::zero()),
    };
}

pub fn claim_rewards(deps: DepsMut, info: MessageInfo, env: Env) -> StdResult<Response> {
    let receiver = info.sender.clone();
    let current_timestamp = Uint128::new((env.block.time.seconds()) as u128);
    let mut messages: Vec<CosmosMsg> = Vec::new();

    // calculate for all also for user
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;
    process_all_claimable_rewards(
        deps.storage,
        receiver.to_string(),
        current_timestamp,
        &mut messages,
    )?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        Attribute::new("action", "claim_rewards"),
        Attribute::new("caller", receiver.as_str().clone()),
        // Attribute::new("reward_amount", claim_amount),
    ]))
}

fn process_all_claimable_rewards(
    storage: &mut dyn Storage,
    receiver: String,
    timestamp: Uint128,
    messages: &mut Vec<CosmosMsg>,
) -> StdResult<()> {
    let mut claim_reward_tokens = claim_reward_info_r(storage).load(receiver.as_bytes())?;
    for claim_reward in claim_reward_tokens.iter_mut() {
        // send all remaing reward token
        let msg = snip20::ExecuteMsg::Send {
            recipient: receiver.to_owned(),
            recipient_code_hash: None,
            amount: claim_reward.amount,
            msg: None,
            memo: None,
            padding: None,
        };

        let cosmos_msg = wasm_execute(
            claim_reward.reward_token_addr.to_owned(),
            claim_reward.reward_token_code_hash.to_owned(),
            &msg,
            vec![],
        )?
        .into();

        messages.push(cosmos_msg);
        claim_reward.amount = Uint128::zero();
    }
    let mut staker_info = stakers_r(storage).load(receiver.as_bytes())?;
    staker_info.last_time_updated = timestamp;
    claim_reward_info_w(storage).save(receiver.as_bytes(), &claim_reward_tokens)?;
    Ok(())
}

// Total Available Rewards = Daily_Rewards / 24*60*60*1000 * (current_date_time - last_calculated_date_time).miliseconds()
// User Incremental Rewards = Total Available Rewards * Staked Percentage
// User Total Rewards = User Owed Rewards + (User Incremental Rewards)
// storage: &mut dyn Storage,
pub fn claim_rewards_for_all_stakers(
    storage: &mut dyn Storage,
    current_timestamp: Uint128,
) -> StdResult<()> {
    let stakers_count = get_total_stakers_count(storage)?;
    let last_claimed_timestamp = last_reward_time_r(storage).load()?;
    for i in 0..stakers_count.u128() {
        // load staker address
        let staker_address: Addr = staker_index_r(storage).load(&i.to_be_bytes())?;
        let mut staker_info = stakers_r(storage).load(staker_address.as_bytes())?;

        let staker_share = calculate_staker_shares(storage, staker_info.amount)?;
        let reward_token_list: Vec<RewardTokenInfo> =
            get_actual_reward_tokens(storage, current_timestamp)?;
        for reward_token in reward_token_list.iter() {
            // calculate reward amount for each reward token
            let mut reward = find_claimable_reward_for_staker_by_reward_token(
                storage,
                &staker_address,
                &reward_token.reward_token,
            )?
            .amount;

            if last_claimed_timestamp < reward_token.valid_to {
                if current_timestamp < reward_token.valid_to {
                    reward += calculate_incremental_staking_reward(
                        staker_share,
                        last_claimed_timestamp,
                        current_timestamp,
                        reward_token.daily_reward_amount,
                    )?;
                } else {
                    reward += calculate_incremental_staking_reward(
                        staker_share,
                        last_claimed_timestamp,
                        reward_token.valid_to,
                        reward_token.daily_reward_amount,
                    )?;
                }
            }

            save_claimable_amount_staker_by_reward_token(
                storage,
                // Add previous claimable for the staker
                reward,
                &staker_address,
                &reward_token.reward_token
            )?;
        }
        staker_info.last_time_updated = current_timestamp;
        // Update the stakers information
        stakers_w(storage).save(staker_address.as_bytes(), &staker_info)?;
    }
    last_reward_time_claimed_w(storage).save(&current_timestamp)?;
    Ok(())
}

pub fn get_actual_reward_tokens(
    storage: &dyn Storage,
    current_timestamp: Uint128,
) -> StdResult<Vec<RewardTokenInfo>> {
    let mut list_token: Vec<RewardTokenInfo> = Vec::new();
    let reward_list = reward_token_list_r(storage).load()?;
    for addr in &reward_list {
        // load total reward token
        let reward_token: RewardTokenInfo = reward_token_r(storage).load(addr.as_bytes())?;
        list_token.push(reward_token.to_owned())
    }
    Ok(list_token)
}

pub fn get_all_claimable_reward_for_staker(
    storage: &dyn Storage,
    staker_address: &Addr,
) -> StdResult<Vec<ClaimRewardsInfo>> {
    let claim_info = match claim_reward_info_r(storage).may_load(staker_address.as_bytes())? {
        Some(claim_reward_info) => claim_reward_info,
        None => Vec::new(),
    };
    Ok(claim_info)
}

pub fn find_claimable_reward_for_staker_by_reward_token(
    storage: &dyn Storage,
    staker_address: &Addr,
    reward_token: &ContractLink,
) -> StdResult<ClaimRewardsInfo> {
    let all_claimable_reward = get_all_claimable_reward_for_staker(storage, staker_address)?;
    let result = match all_claimable_reward
        .iter()
        .find(|&x| x.reward_token_addr == reward_token.address.to_owned())
    {
        Some(clm) => clm.to_owned(),
        None => ClaimRewardsInfo {
            amount: Uint128::zero(),
            reward_token_addr: reward_token.address.to_owned(),
            reward_token_code_hash: reward_token.code_hash.to_owned(),
        },
    };
    Ok(result)
}

pub fn find_claimable_reward_index_for_staker(
    storage: &dyn Storage,
    staker_address: &Addr,
    reward_token: &ContractLink,
) -> StdResult<Option<usize>> {
    let all_claimable_reward = get_all_claimable_reward_for_staker(storage, staker_address)?;
    return Ok(all_claimable_reward
        .iter()
        .position(|x| x.reward_token_addr == reward_token.address));
}

pub fn save_claimable_amount_staker_by_reward_token(
    storage: &mut dyn Storage,
    amount: Uint128,
    staker_address: &Addr,
    reward_token: &ContractLink,
) -> StdResult<()> {
    let mut list_claimable_reward =
        get_all_claimable_reward_for_staker(storage, &staker_address)?;
    let claimable_reward_index = find_claimable_reward_index_for_staker(
        storage,
        staker_address,
        reward_token
    )?;
    let mut claimable_reward = find_claimable_reward_for_staker_by_reward_token(
        storage,
        &staker_address,
        &reward_token,
    )?;
    match claimable_reward_index {
        Some(index) => {
            list_claimable_reward[index].amount = amount;
        }
        None => {
            claimable_reward.amount = amount;
            list_claimable_reward.push(claimable_reward.to_owned());
        }
    }
    claim_reward_info_w(storage).save(staker_address.as_bytes(), &list_claimable_reward)?;
    Ok(())
}

/**
 *
 */
pub fn calculate_incremental_staking_reward(
    percentage: Decimal,
    last_timestamp: Uint128,
    to_timestamp: Uint128,
    emmision_rate: Uint128,
) -> StdResult<Uint128> {
    let seconds_in_day = Uint128::new(24u128 * 60u128 * 60u128);
    if last_timestamp < to_timestamp {
        let time_dif = to_timestamp - last_timestamp;
        let total_available_reward = emmision_rate.multiply_ratio(time_dif, seconds_in_day);
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
            amm_pair: config.amm_pair.clone(),
        };
        return to_binary(&response);
    } else {
        return Err(StdError::generic_err("Invalid reward token"));
    }
}

pub fn update_authenticator(
    storage: &mut dyn Storage,
    authenticator: Option<Contract>,
) -> StdResult<Response> {
    let mut config = config_r(storage).load()?;
    config.authenticator = authenticator;
    config_w(storage).save(&config)?;
    Ok(Response::default())
}

pub fn get_staking_stake_lp_token_info(deps: Deps, staker: Addr) -> StdResult<Binary> {
    let staker_info = stakers_r(deps.storage).load(&staker.as_bytes())?;
    let response_msg = QueryResponse::StakerLpTokenInfo {
        staked_lp_token: staker_info.amount,
        total_staked_lp_token: total_staked_r(deps.storage).load()?,
    };
    to_binary(&response_msg)
}

pub fn get_claim_reward_for_user(deps: Deps, staker: Addr, time: Uint128) -> StdResult<Binary> {
    // load stakers
    let mut result_list: Vec<ClaimableInfo> = Vec::new();
    let staker_info = stakers_r(deps.storage).load(staker.as_bytes())?;
    let reward_token_list: Vec<RewardTokenInfo> = get_actual_reward_tokens(deps.storage, time)?;
    let percentage = calculate_staker_shares(deps.storage, staker_info.amount)?;
    for reward_token in reward_token_list.iter() {
        if reward_token.valid_to < staker_info.last_time_updated {
            let reward: Uint128;
            if time > reward_token.valid_to {
                // calculate reward amount for each reward token
                reward = calculate_incremental_staking_reward(
                    percentage,
                    staker_info.last_time_updated,
                    time,
                    reward_token.daily_reward_amount,
                )?;
            } else {
                reward = calculate_incremental_staking_reward(
                    percentage,
                    staker_info.last_time_updated,
                    reward_token.valid_to,
                    reward_token.daily_reward_amount,
                )?;
            }
            // load any existing claimable reward for specif user
            let claimable_reward = find_claimable_reward_for_staker_by_reward_token(
                deps.storage,
                &staker,
                &reward_token.reward_token,
            )?;
            result_list.push(ClaimableInfo {
                token_address: reward_token.reward_token.address.to_owned(),
                amount: claimable_reward.amount + reward,
            });
        }
    }
    to_binary(&QueryResponse::ClaimRewards {
        claimable_rewards: result_list,
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
    let current_timestamp = Uint128::new((env.block.time.seconds()) as u128);
    let mut staker_info = stakers_r(deps.storage).load(caller.as_bytes())?;
    // claim rewards
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;
    // remove staker
    let mut messages: Vec<CosmosMsg> = Vec::new();
    // check if the amount is higher than the current staking amount
    if amount > staker_info.amount {
        return Err(StdError::generic_err(
            "Staking Amount is higher then actual staking amount".to_string(),
        ));
    }

    staker_info.amount = staker_info.amount - amount;
    staker_info.last_time_updated = current_timestamp;
    stakers_w(deps.storage).save(caller.as_bytes(), &staker_info)?;

    process_all_claimable_rewards(
        deps.storage,
        caller.to_string(),
        current_timestamp,
        &mut messages,
    )?;
    // send back amount of lp token to pair contract to send pair token back with burn
    let config = config_r(deps.storage).load()?;

    if let Some(true) = remove_liqudity {
        // SEND LP Token back to Pair Contract With Remove Liquidity
        let remove_liquidity_msg = to_binary(&AmmPairInvokeMsg::RemoveLiquidity {
            from: Some(caller.clone()),
        })?;
        let msg = snip20::ExecuteMsg::Send {
            recipient: config.amm_pair.to_string(),
            recipient_code_hash: None,
            amount: amount,
            msg: Some(remove_liquidity_msg.clone()),
            memo: None,
            padding: None,
        };

        let coms_msg = wasm_execute(
            config.lp_token.address.to_string(),
            config.lp_token.code_hash.clone(),
            &msg,
            vec![],
        )?
        .into();

        messages.push(coms_msg);
    } else {
        // SEND LP Token back to Staker And User Will Manually Remove Liquidity
        let msg = snip20::ExecuteMsg::Transfer {
            recipient: caller.to_string(),
            amount: amount,
            memo: None,
            padding: None,
        };

        let coms_msg = wasm_execute(
            config.lp_token.address.to_string(),
            config.lp_token.code_hash.clone(),
            &msg,
            vec![],
        )?
        .into();

        messages.push(coms_msg);
    }
    Ok(Response::new().add_messages(messages).add_attributes(vec![
        Attribute::new("action", "unstake"),
        Attribute::new("amount", amount),
        Attribute::new("staker", caller.as_str()),
    ]))
}

pub fn create_send_msg(
    recipient: String,
    amount: Uint128,
    token_link: ContractLink,
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
    let addrs = stakers_r(deps.storage).may_load(address.as_bytes())?;
    match addrs {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
