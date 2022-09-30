// This should be callback from Snip20 Receiver
// needs to check for the amount

use std::thread::current;
use std::time::{SystemTime, UNIX_EPOCH};

const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

use cosmwasm_std::{
    to_binary, Addr, Attribute, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Storage, Uint128,
};
use cosmwasm_std::{Binary, QuerierWrapper, WasmMsg};
use shadeswap_shared::core::{admin_r, TokenType};
use shadeswap_shared::snip20;
use shadeswap_shared::snip20::helpers::token_info;
use shadeswap_shared::staking::{ProxyStakeMsg, QueryResponse};
use shadeswap_shared::{
    core::{ContractLink, ViewingKey},
    msg::amm_pair::InvokeMsg as AmmPairInvokeMsg,
    Contract,
};

use crate::state::{
    claim_reward_info_r, claim_reward_info_w, config_r, config_w, last_reward_time_claimed_w,
    staker_index_r, staker_index_w, stakers_r, stakers_vk_r, stakers_vk_w, stakers_w,
    total_staked_r, total_staked_w, total_stakers_r, total_stakers_w, whitelisted_proxy_stakers_r,
    whitelisted_proxy_stakers_w, ClaimRewardsInfo, Config, StakingInfo,
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

pub fn calculate_staker_shares(storage: &dyn Storage, amount: Uint128) -> StdResult<Decimal> {
    let total_staking_amount: Uint128 = match total_staked_r(storage).may_load() {
        Ok(it) => it.unwrap_or(Uint128::zero()),
        Err(err) => Uint128::zero(),
    };
    if total_staking_amount.is_zero() == true {
        return Ok(Decimal::zero());
    }

    let user_share = Decimal::from_ratio(amount, total_staking_amount);
    Ok(user_share)
}

pub fn stake(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    amount: Uint128,
    from: Addr,
) -> StdResult<Response> {
    // calculate staking for existing stakers without increasing amount
    let current_timestamp = Uint128::from((env.block.time.seconds() * 1000) as u128);
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;

    // set the new total stake amount
    let mut total_stake_amount = match total_staked_r(deps.storage).may_load() {
        Ok(it) => it.unwrap_or(Uint128::zero()),
        Err(_) => Uint128::zero(),
    };

    total_stake_amount += amount;
    total_staked_w(deps.storage).save(&total_stake_amount)?;

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
                amount,
                last_time_updated: current_timestamp,
            },
        )?;

        // new staker add it to the count
        let mut stakers_count = get_total_stakers_count(deps.storage);
        stakers_count += Uint128::from(1u128);
        total_stakers_w(deps.storage).save(&stakers_count)?;
        // store staker with index
        staker_index_w(deps.storage).save(&stakers_count.u128().to_be_bytes(), &caller.clone())?;
        // store zero for claim rewards
        println!("storing claim first time {}", current_timestamp);
        claim_reward_info_w(deps.storage).save(
            caller.as_bytes(),
            &ClaimRewardsInfo {
                amount: Uint128::from(0u128),
                last_time_claimed: current_timestamp,
            },
        )?;
    }

    // return response
    Ok(Response::new().add_attributes(vec![
        Attribute::new("action", "stake"),
        Attribute::new("staker", caller.as_str()),
        Attribute::new("amount", amount),
    ]))
}

pub fn get_total_stakers_count(storage: &dyn Storage) -> Uint128 {
    match total_stakers_r(storage).may_load() {
        Ok(it) => it.unwrap_or(Uint128::zero()),
        Err(_) => Uint128::zero(),
    }
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
        Attribute::new("caller", receiver),
        Attribute::new("reward_amount", claim_amount),
    ]))
}

// Total Available Rewards = Daily_Rewards / 24*60*60*1000 * (current_date_time - last_calculated_date_time).miliseconds()
// User Incremental Rewards = Total Available Rewards * Staked Percentage
// User Total Rewards = User Owed Rewards + (User Incremental Rewards)
pub fn claim_rewards_for_all_stakers(
    storage: &mut dyn Storage,
    current_timestamp: Uint128,
) -> StdResult<()> {
    // TO DO FIX THIS
    let stakers_count = get_total_stakers_count(storage);
    let mut index = Uint128::one();
    while index <= stakers_count {
        // load staker address
        let staker_address: Addr = staker_index_r(storage).load(&index.to_be_bytes())?;
        let mut staker_info = match stakers_r(storage).may_load(staker_address.as_bytes()) {
            Ok(it) => it.unwrap_or(StakingInfo {
                amount: Uint128::zero(),
                staker: Addr::unchecked(""),
                last_time_updated: Uint128::zero(),
            }),
            Err(_) => StakingInfo {
                amount: Uint128::zero(),
                staker: Addr::unchecked(""),
                last_time_updated: Uint128::zero(),
            },
        };

        if staker_info.amount != Uint128::zero() {
            let reward = calculate_staking_reward(
                storage,
                staker_info.amount,
                staker_info.last_time_updated,
                current_timestamp,
            )?;
            let mut claim_info =
                match claim_reward_info_r(storage).may_load(staker_address.as_bytes()) {
                    Ok(it) => it.unwrap_or(ClaimRewardsInfo {
                        amount: Uint128::zero(),
                        last_time_claimed: Uint128::zero(),
                    }),
                    Err(_) => ClaimRewardsInfo {
                        amount: Uint128::zero(),
                        last_time_claimed: Uint128::zero(),
                    },
                };

            claim_info.amount += reward;
            claim_info.last_time_claimed = current_timestamp;
            claim_reward_info_w(storage).save(staker_address.as_bytes(), &claim_info)?;
        }
        index += Uint128::one()
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
    let seconds = Uint128::from(24u128 * 60u128 * 60u128 * 1000u128);
    if last_timestamp < current_timestamp {
        let time_dif = (current_timestamp - last_timestamp);
        let total_available_reward = config.daily_reward_amount.multiply_ratio(time_dif, seconds);
        let converted_total_reward = Decimal::from_atomics(total_available_reward, 0).unwrap();
        let result = converted_total_reward.checked_mul(percentage)?;
        Ok(result.atomics().checked_div(DECIMAL_FRACTIONAL)?)
    } else {
        Ok(Uint128::from(0u128))
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
        let reward_token_balance =
            config
                .reward_token
                .query_balance(deps, staker.to_string(), viewing_key.to_string())?;
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
    let whitelisted_proxy_stakers = whitelisted_proxy_stakers_r(deps.storage).load()?;
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
            whitelisted_proxy_stakers,
        };
        return to_binary(&response);
    } else {
        return Err(StdError::generic_err("Invalid reward token"));
    }
}

pub fn get_staking_stake_lp_token_info(deps: Deps, staker: Addr) -> StdResult<Binary> {
    let is_staker = is_address_already_staker(deps, staker.clone())?;
    if is_staker == false {
        return Err(StdError::generic_err(
            "Shared address is not staker".to_string(),
        ));
    }

    let staker_info = stakers_r(deps.storage).load(&staker.as_bytes())?;
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

pub fn get_claim_reward_for_user(deps: Deps, staker: Addr, time: Uint128) -> StdResult<Binary> {
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
    claim_rewards_for_all_stakers(deps.storage, current_timestamp)?;
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
    let config = config_r(deps.storage).load()?;

    if let Some(true) = remove_liqudity {
        // SEND LP Token back to Pair Contract With Remove Liquidity
        let remove_liquidity_msg = to_binary(&AmmPairInvokeMsg::RemoveLiquidity {
            from: Some(caller.clone()),
        })
        .unwrap();
        let msg = to_binary(&snip20::ExecuteMsg::Send {
            recipient: config.contract_owner.to_string(),
            recipient_code_hash: None,
            amount,
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
            amount,
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

pub fn try_proxy_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ProxyStakeMsg,
) -> StdResult<Response> {
    let storage = deps.as_ref().storage;
    match msg {
        ProxyStakeMsg::UpdateWhitelist { add, remove } => {
            if !admin_r(storage).load()?.eq(&info.sender) {
                Err(StdError::generic_err("Unauthorized"))
            } else {
                let mut current_whitelist = whitelisted_proxy_stakers_r(storage).load()?;
                for addr in add {
                    if !current_whitelist.contains(&addr) {
                        current_whitelist.push(addr);
                    }
                }

                current_whitelist.retain(|a| !remove.contains(a));

                whitelisted_proxy_stakers_w(deps.storage).save(&current_whitelist)?;
                Ok(Response::default().add_attribute("action", "whitelist_proxy_stakers_update"))
            }
        }
        ProxyStakeMsg::Stake {
            token,
            amount,
            user,
        } => {
            require_lp_token(storage, token)?;
            stake(deps, env, info, amount, user)
        }
        ProxyStakeMsg::Unstake {
            token,
            amount,
            user,
        } => {
            require_lp_token(storage, token)?;
            // unstake uses MessageInfo to know who to stake for
            let new_info = MessageInfo {
                sender: user,
                funds: vec![],
            };
            unstake(deps, env, new_info, amount, None)
        }
    }
}

pub fn require_lp_token(storage: &dyn Storage, token: Contract) -> StdResult<()> {
    let config = config_r(storage).load()?;
    if config.lp_token.address.eq(&token.address) && config.lp_token.code_hash.eq(&token.code_hash)
    {
        Ok(())
    } else {
        Err(StdError::generic_err(format!("Token of address {} and code hash {} does not equal the LP token address {} and code hash {} registered in the staking contract.", token.address, token.code_hash, config.lp_token.address, config.lp_token.code_hash)))
    }
}
