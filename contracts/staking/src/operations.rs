const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Div;

use cosmwasm_std::{
    to_binary, Addr, Attribute, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Storage, Uint128, WasmMsg,
};
use shadeswap_shared::core::TokenType;
use shadeswap_shared::snip20;
use shadeswap_shared::staking::RewardTokenInfo;
use shadeswap_shared::utils::ExecuteCallback;
use shadeswap_shared::{msg::amm_pair::InvokeMsg as AmmPairInvokeMsg, Contract};
const SECONDS_IN_DAY: Uint128 = Uint128::new(24u128 * 60u128 * 60u128);
const MAX_DECIMALS: Uint128 = Uint128::new(1_000_000_000_000_000_000);

use crate::contract::SHADE_STAKING_KEY;
use crate::state::{
    claim_reward_info_r, claim_reward_info_w, config_r, config_w, proxy_staker_info_r,
    proxy_staker_info_w, reward_token_list_r, reward_token_list_w, reward_token_r, reward_token_w,
    stakers_r, stakers_w, total_staked_r, total_staked_w, ClaimRewardsInfo, ProxyStakingInfo,
    StakingInfo,
};

/// Store init reward token with timestamp
pub fn store_init_reward_token_and_timestamp(
    storage: &mut dyn Storage,
    reward_token: TokenType,
    daily_emission_amount: Uint128,
    current_timestamp: Uint128,
) -> StdResult<()> {
    // store reward token to the list
    let mut reward_token_list: Vec<String> = Vec::new();
    reward_token_list.push(reward_token.unique_key());
    reward_token_list_w(storage).save(&reward_token_list)?;
    reward_token_w(storage).save(
        &reward_token.unique_key().as_bytes(),
        &RewardTokenInfo {
            reward_token: reward_token.to_owned(),
            reward_rate: daily_emission_amount.checked_div(SECONDS_IN_DAY)?,
            valid_to: current_timestamp,
            reward_per_token_stored: Uint128::zero(),
            last_update_time: current_timestamp,
        },
    )?;
    Ok(())
}

/// Stake
pub fn stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    for_addr: Addr,
    from_addr: Addr,
) -> StdResult<Response> {
    // this is receiver for LP Token send to staking contract ->
    let config = config_r(deps.storage).load()?;
    if config.lp_token.address != info.sender {
        return Err(StdError::generic_err(
            "Token sent is not LP Token".to_string(),
        ));
    }

    update_reward(
        Uint128::new((env.block.time.seconds()) as u128),
        &for_addr,
        deps.storage,
    )?;

    let mut total_stake_amount = match total_staked_r(deps.storage).may_load()? {
        Some(total_amount) => total_amount,
        None => Uint128::zero(),
    };
    total_stake_amount += amount;
    total_staked_w(deps.storage).save(&total_stake_amount)?;

    let caller = from_addr.to_owned();
    // check if user has staked before
    match stakers_r(deps.storage).may_load(caller.as_bytes())? {
        Some(mut stake_info) => {
            stake_info.amount += amount;
            if for_addr != from_addr {
                stake_info.proxy_staked += amount;
            }
            stakers_w(deps.storage).save(caller.as_bytes(), &stake_info)?;
        }
        None => {
            if for_addr != from_addr {
                stakers_w(deps.storage).save(
                    caller.as_bytes(),
                    &StakingInfo {
                        amount,
                        proxy_staked: amount,
                    },
                )?;
            } else {
                stakers_w(deps.storage).save(
                    caller.as_bytes(),
                    &StakingInfo {
                        amount,
                        proxy_staked: Uint128::zero(),
                    },
                )?;
            }
        }
    }

    if from_addr != for_addr {
        let proxy_staking_key = &generate_proxy_staking_key(&from_addr, &for_addr);

        let proxy_staker_info = proxy_staker_info_r(deps.storage).may_load(proxy_staking_key)?;

        match proxy_staker_info {
            Some(mut p) => {
                p.amount += amount;
                proxy_staker_info_w(deps.storage).save(proxy_staking_key, &p)?;
            }
            None => {
                proxy_staker_info_w(deps.storage)
                    .save(proxy_staking_key, &ProxyStakingInfo { amount })?;
            }
        }
    }

    Ok(Response::new().add_attributes(vec![
        Attribute::new("action", "stake"),
        Attribute::new("staker", caller.as_str()),
        Attribute::new("amount", amount),
    ]))
}

/// Generate proxy Staking Key for Proxy Staker
pub fn generate_proxy_staking_key(from: &Addr, for_addr: &Addr) -> Vec<u8> {
    [from.as_bytes(), for_addr.as_bytes()].concat()
}

/// Execute Claim Rewards for Staker
pub fn claim_rewards(deps: DepsMut, info: MessageInfo, env: Env) -> StdResult<Response> {
    let receiver = info.sender.clone();
    let current_timestamp = Uint128::new((env.block.time.seconds()) as u128);
    update_reward(current_timestamp, &receiver, deps.storage);

    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut new_hashmap = HashMap::new();

    for mut claim_info in claim_reward_info_r(deps.storage)
        .load(receiver.as_bytes())?
        .iter()
    {
        let total = claim_info.1.amount;
        if total > Uint128::zero() {
            messages.push(claim_info.1.reward_token.create_send_msg(
                env.contract.address.to_string(),
                receiver.to_string(),
                total,
            )?);
            let mut new_data = claim_info.1.clone();
            new_data.amount = Uint128::zero();
            new_hashmap.insert(claim_info.0.clone(), new_data);
        }
    }

    claim_reward_info_w(deps.storage).save(receiver.as_bytes(), &new_hashmap);

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        Attribute::new("action", "claim_rewards"),
        Attribute::new("caller", info.sender.to_string()),
    ]))
}

pub fn set_reward_token(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    daily_reward_amount: Uint128,
    reward_token: TokenType,
    valid_to: Uint128
) -> StdResult<Response> {
    let current_timestamp = Uint128::new((env.block.time.seconds()) as u128);

    let mut token_info = reward_token_w(deps.storage).load(reward_token.unique_key().as_bytes())?;

    if (current_timestamp >= token_info.valid_to) {
        token_info.reward_rate = daily_reward_amount.checked_div(SECONDS_IN_DAY)?;
    } else {
        let remaining = token_info.valid_to.checked_sub(current_timestamp)?;
        let leftover = remaining.checked_mul(token_info.reward_rate)?;
        token_info.reward_rate = daily_reward_amount.checked_add(leftover)?.checked_div(SECONDS_IN_DAY)?
    }

    // Ensure the provided reward amount is not more than the balance in the contract.
    // This keeps the reward rate in the right range, preventing overflows due to
    // very high values of rewardRate in the earned and rewardsPerToken functions;
    // Reward + leftover must be less than 2^128 / 10^18 to avoid overflow.
    let balance = reward_token.query_balance(
        deps.as_ref(),
        env.contract.address.to_string(),
        SHADE_STAKING_KEY.to_string(),
    )?;

    if token_info.reward_rate > balance {
        return Err(StdError::generic_err("Provided reward rate is too high"));
    }

    token_info.last_update_time = current_timestamp;
    token_info.valid_to = valid_to;

    reward_token_w(deps.storage).save(reward_token.unique_key().as_bytes(), &token_info);
    Ok(Response::new().add_attributes(vec![
        Attribute::new("action", "set_reward_token"),
        Attribute::new("daily_reward_amount", daily_reward_amount.to_string()),
        Attribute::new("valid_to", valid_to.to_string()),
    ]))
}

/// Return List of Reward Tokens
pub fn get_reward_tokens_info(storage: &dyn Storage) -> StdResult<Vec<RewardTokenInfo>> {
    let mut list_token: Vec<RewardTokenInfo> = Vec::new();
    let reward_list = reward_token_list_r(storage).load()?;
    for addr in &reward_list {
        // load total reward token
        let reward_token: RewardTokenInfo = reward_token_r(storage).load(addr.as_bytes())?;
        list_token.push(reward_token.to_owned())
    }
    Ok(list_token)
}

// Update authenticator used to authenticate permits
pub fn update_authenticator(
    storage: &mut dyn Storage,
    authenticator: Option<Contract>,
) -> StdResult<Response> {
    let mut config = config_r(storage).load()?;
    config.authenticator = authenticator;
    config_w(storage).save(&config)?;
    Ok(Response::default())
}

/// Unstake Amount
pub fn unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    for_address: Addr,
    amount: Uint128,
    remove_liquidity: Option<bool>,
) -> StdResult<Response> {
    let caller = info.sender.clone();
    let current_timestamp = Uint128::new((env.block.time.seconds()) as u128);

    update_reward(current_timestamp, &for_address, deps.storage)?;
    let mut total_stake_amount = total_staked_w(deps.storage).load()?;
    total_stake_amount -= amount;
    total_staked_w(deps.storage).save(&total_stake_amount)?;
    let mut messages: Vec<CosmosMsg> = Vec::new();

    if info.sender == for_address {
        if let Some(mut staker_info) = stakers_r(deps.storage).may_load(caller.as_bytes())? {
            staker_info.amount = staker_info.amount - amount;
            stakers_w(deps.storage).save(caller.as_bytes(), &staker_info)?;

            // send back amount of lp token to pair contract to send pair token back with burn
            let config = config_r(deps.storage).load()?;

            if let Some(true) = remove_liquidity {
                // SEND LP Token back to Pair Contract With Remove Liquidity
                let remove_liquidity_msg = to_binary(&AmmPairInvokeMsg::RemoveLiquidity {
                    from: Some(caller.to_string()),
                    single_sided_withdraw_type: None,
                    single_sided_expected_return: None,
                })?;

                let cosmos_msg = snip20::ExecuteMsg::Send {
                    recipient: config.amm_pair.to_string(),
                    recipient_code_hash: None,
                    amount,
                    msg: Some(remove_liquidity_msg.clone()),
                    memo: None,
                    padding: None,
                }
                .to_cosmos_msg(&config.lp_token, vec![])?;

                messages.push(cosmos_msg);
            } else {
                // SEND LP Token back to Staker And User Will Manually Remove Liquidity
                let cosmos_msg = snip20::ExecuteMsg::Transfer {
                    recipient: caller.to_string(),
                    amount,
                    memo: None,
                    padding: None,
                }
                .to_cosmos_msg(&config.lp_token, vec![])?;

                messages.push(cosmos_msg);
            }
            return Ok(Response::new().add_messages(messages).add_attributes(vec![
                Attribute::new("action", "unstake"),
                Attribute::new("amount", amount),
                Attribute::new("staker", caller.as_str()),
            ]));
        } else {
            return Err(StdError::generic_err(
                "Staking information does not exist".to_string(),
            ));
        }
    } else {
        let mut staker_info = stakers_r(deps.storage).load(for_address.as_bytes())?;
        let proxy_staking_key = &generate_proxy_staking_key(&caller, &for_address);
        if let Some(mut proxy_staker_info) =
            proxy_staker_info_r(deps.storage).may_load(proxy_staking_key)?
        {
            // remove staker
            let mut messages: Vec<CosmosMsg> = Vec::new();
            // check if the amount is higher than what has been totally staked and proxy staked by this caller
            if amount > proxy_staker_info.amount || amount > staker_info.proxy_staked {
                return Err(StdError::generic_err(
                    "Staking Amount is higher then actual staking amount".to_string(),
                ));
            }

            staker_info.amount -= amount;
            staker_info.proxy_staked -= amount;
            stakers_w(deps.storage).save(for_address.as_bytes(), &staker_info)?;

            //Update the proxy stakers
            proxy_staker_info.amount -= amount;
            proxy_staker_info_w(deps.storage).save(
                &generate_proxy_staking_key(&caller, &for_address),
                &proxy_staker_info,
            )?;

            let mut total_stake_amount = total_staked_w(deps.storage).load()?;
            total_stake_amount -= amount;
            total_staked_w(deps.storage).save(&total_stake_amount)?;

            // send back amount of lp token to pair contract to send pair token back with burn
            let config = config_r(deps.storage).load()?;

            let cosmos_msg = snip20::ExecuteMsg::Transfer {
                recipient: caller.to_string(),
                amount,
                memo: None,
                padding: None,
            }
            .to_cosmos_msg(&config.lp_token, vec![])?;

            messages.push(cosmos_msg);
            return Ok(Response::new().add_messages(messages).add_attributes(vec![
                Attribute::new("action", "unstake"),
                Attribute::new("amount", amount),
                Attribute::new("staker", caller.as_str()),
            ]));
        } else {
            return Err(StdError::generic_err(
                "Staking information does not exist".to_string(),
            ));
        }
    }
}

pub fn create_send_msg(
    recipient: String,
    amount: Uint128,
    token_link: Contract,
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

pub fn last_time_reward_applicable(
    current_timestamp: Uint128,
    valid_to: Uint128,
) -> StdResult<Uint128> {
    if current_timestamp < valid_to {
        return Ok(current_timestamp);
    }
    Ok(valid_to)
}

pub fn reward_per_token(
    current_timestamp: Uint128,
    reward_token: &RewardTokenInfo,
    total_staked: Uint128,
) -> StdResult<Uint128> {
    if (total_staked.is_zero()) {
        return Ok(reward_token.reward_per_token_stored);
    }
    return Ok(reward_token.reward_per_token_stored.checked_add(
        last_time_reward_applicable(current_timestamp, reward_token.valid_to)?
            .checked_sub(reward_token.last_update_time)?
            .checked_mul(MAX_DECIMALS)?
            .checked_div(total_staked)?,
    )?);
}

pub fn earned(
    address: &Addr,
    accumulated_reward: Uint128,
    reward_per_token: Uint128,
    reward_per_token_paid: Uint128,
    storage: &dyn Storage,
) -> StdResult<Uint128> {
    let staker_info = stakers_r(storage).load(address.as_bytes())?;
    Ok(staker_info
        .amount
        .checked_mul(reward_per_token.checked_sub(reward_per_token_paid)?)?
        .checked_div(MAX_DECIMALS)?
        .checked_add(accumulated_reward)?)
}

pub fn update_reward(
    current_timestamp: Uint128,
    address: &Addr,
    storage: &mut dyn Storage,
) -> StdResult<()> {
    let reward_list = reward_token_list_r(storage).load()?;
    let total_staked = match total_staked_r(storage).may_load()? {
        Some(s) => s,
        None => Uint128::zero(),
    };
    for addr in &reward_list {
        // load total reward token
        let mut reward_token_info: RewardTokenInfo = reward_token_w(storage).load(addr.as_bytes())?;
        reward_token_info.reward_per_token_stored =
            reward_per_token(current_timestamp, &reward_token_info, total_staked)?;
        reward_token_info.last_update_time =
            last_time_reward_applicable(current_timestamp, reward_token_info.valid_to)?;
        let mut claim_reward_info =
            match claim_reward_info_w(storage).may_load(address.as_bytes())? {
                Some(hm) => hm,
                None => HashMap::new(),
            };

        if !claim_reward_info.contains_key(addr) {
            claim_reward_info.insert(
                addr.clone(),
                ClaimRewardsInfo {
                    amount: Uint128::zero(),
                    reward_token: reward_token_info.clone().reward_token,
                    reward_token_per_token_paid: Uint128::zero(),
                },
            );
        }

        let new_amount = earned(
            &address,
            claim_reward_info[addr].amount,
            reward_token_info.reward_per_token_stored.clone(),
            claim_reward_info[addr].reward_token_per_token_paid,
            storage,
        )?;

        claim_reward_info.entry(addr.to_string()).and_modify(|d| { d.amount = new_amount; d.reward_token_per_token_paid = reward_token_info.clone().reward_per_token_stored});

        claim_reward_info_w(storage).save(address.as_bytes(), &claim_reward_info)?;
    }
    Ok(())
}

/// Check if Address is already stored as Staker
pub fn is_address_already_staker(deps: Deps, address: Addr) -> StdResult<bool> {
    let addrs = stakers_r(deps.storage).may_load(address.as_bytes())?;
    match addrs {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
