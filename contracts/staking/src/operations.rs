use cosmwasm_std::{
    to_binary, Addr, Attribute, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Storage, Uint128, WasmMsg,
};
use shadeswap_shared::core::TokenType;
use shadeswap_shared::snip20;
use shadeswap_shared::snip20::helpers::set_viewing_key_msg;
use shadeswap_shared::staking::RewardTokenInfo;
use shadeswap_shared::utils::ExecuteCallback;
use shadeswap_shared::{msg::amm_pair::InvokeMsg as AmmPairInvokeMsg, Contract};
const SECONDS_IN_DAY: Uint128 = Uint128::new(24u128 * 60u128 * 60u128);
const MAX_DECIMALS: Uint128 = Uint128::new(1_000_000_000_000_000_000);

use crate::contract::{SHADE_STAKING_VIEWKEY};
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
    env: &Env,
    info: &MessageInfo,
    amount: Uint128,
    from_addr: &Addr,
    for_addr: &Addr,
) -> StdResult<Response> {
    // this is receiver for LP Token send to staking contract ->
    let config = config_r(deps.storage).load()?;
    if config.lp_token.address != info.sender {
        return Err(StdError::generic_err(
            "Token sent is not LP Token.".to_string(),
        ));
    }

    update_reward(
        Uint128::new((env.block.time.seconds()) as u128),
        &for_addr,
        deps.storage,
        &env,
    )?;

    let mut total_stake_amount = match total_staked_r(deps.storage).may_load()? {
        Some(total_amount) => total_amount,
        None => Uint128::zero(),
    };
    total_stake_amount += amount;
    total_staked_w(deps.storage).save(&total_stake_amount)?;

    // check if user has staked before
    match stakers_r(deps.storage).may_load(for_addr.as_bytes())? {
        Some(mut stake_info) => {
            stake_info.amount += amount;
            if for_addr != from_addr {
                stake_info.proxy_staked += amount;
            }
            stakers_w(deps.storage).save(for_addr.as_bytes(), &stake_info)?;
        }
        None => {
            if for_addr.to_string() != from_addr.to_string() {
                stakers_w(deps.storage).save(
                    for_addr.as_bytes(),
                    &StakingInfo {
                        amount,
                        proxy_staked: amount,
                    },
                )?;
            } else {
                stakers_w(deps.storage).save(
                    for_addr.as_bytes(),
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
        //Attribute::new("staker", caller.as_str()),
        Attribute::new("amount", amount),
    ]))
}

/// Generate proxy Staking Key for Proxy Staker
pub fn generate_proxy_staking_key(from: &Addr, for_addr: &Addr) -> Vec<u8> {
    [from.as_bytes(), for_addr.as_bytes()].concat()
}

/// Execute Claim Rewards for Staker
pub fn claim_rewards(
    deps: DepsMut,
    current_timestamp: Uint128,
    claimer: &Addr,
    env: &Env,
) -> StdResult<Response> {
    let receiver = claimer.clone();
    update_reward(current_timestamp, &receiver, deps.storage, &env)?;

    let mut messages: Vec<CosmosMsg> = Vec::new();

    let reward_list = reward_token_list_r(deps.storage).load()?;
    for addr in &reward_list {
        let key = get_user_claim_key(receiver.to_string(), addr.to_string());
        let claim_info_option = claim_reward_info_r(deps.storage).may_load(key.as_bytes())?;

        match claim_info_option {
            Some(claim_info) => {
                let total = claim_info.rewards;
                if total > Uint128::zero() {
                    messages.push(claim_info.reward_token.create_send_msg(
                        env.contract.address.to_string(),
                        receiver.to_string(),
                        total,
                    )?);
                    let mut new_data = claim_info.clone();
                    new_data.rewards = Uint128::zero();
                    claim_reward_info_w(deps.storage).save(key.as_bytes(), &new_data)?;
                }
            }
            None => (),
        }
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        Attribute::new("action", "claim_rewards"),
        Attribute::new("caller", claimer.to_string()),
    ]))
}

pub fn set_reward_token(
    deps: DepsMut,
    env: &Env,
    daily_reward_amount: Uint128,
    reward_token: TokenType,
    valid_to: Uint128,
) -> StdResult<Response> {
    let current_timestamp = Uint128::new((env.block.time.seconds()) as u128);
    update_reward(current_timestamp, &env.contract.address, deps.storage, &env)?;

    let token_info_option =
        reward_token_w(deps.storage).may_load(reward_token.unique_key().as_bytes())?;
    let mut token_info: RewardTokenInfo;

    let mut messages = vec![];

    match token_info_option {
        None => {
            match reward_token.clone() {
                TokenType::CustomToken { contract_addr, token_code_hash } => {
                    messages.push(set_viewing_key_msg(
                        SHADE_STAKING_VIEWKEY.to_string(),
                        None,
                        &Contract {
                            address: contract_addr.clone(),
                            code_hash: token_code_hash.to_string(),
                        },
                    )?);
                },
                TokenType::NativeToken { denom:_ } => (),
            }

            let mut reward_token_list = reward_token_list_w(deps.storage).load()?;
            reward_token_list.push(reward_token.unique_key());
            reward_token_list_w(deps.storage).save(&reward_token_list)?;
            token_info = RewardTokenInfo {
                reward_token: reward_token.to_owned(),
                reward_rate: daily_reward_amount.checked_div(SECONDS_IN_DAY)?,
                valid_to: current_timestamp,
                reward_per_token_stored: Uint128::zero(),
                last_update_time: current_timestamp,
            };
            reward_token_w(deps.storage)
                .save(&reward_token.unique_key().as_bytes(), &token_info)?;
        }
        Some(ti) => {
            token_info = ti;
            if current_timestamp >= valid_to {
                token_info.reward_rate = daily_reward_amount.checked_div(SECONDS_IN_DAY)?;
            } else {
                token_info.reward_rate = daily_reward_amount
                    .checked_div(SECONDS_IN_DAY)?
            }
        }
    }

    token_info.last_update_time = current_timestamp;
    token_info.valid_to = valid_to;

    reward_token_w(deps.storage).save(reward_token.unique_key().as_bytes(), &token_info)?;
    Ok(Response::new().add_messages(messages).add_attributes(vec![
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
    env: &Env,
    from_address: &Addr,
    for_address: &Addr,
    amount: Uint128,
    remove_liquidity: Option<bool>,
) -> StdResult<Response> {
    let current_timestamp = Uint128::new((env.block.time.seconds()) as u128);

    update_reward(current_timestamp, &for_address, deps.storage, &env)?;

    let mut messages: Vec<CosmosMsg> = Vec::new();

    if from_address == for_address {
        if let Some(mut staker_info) = stakers_r(deps.storage).may_load(for_address.as_bytes())? {
            if amount > (staker_info.amount - staker_info.proxy_staked) {
                return Err(StdError::generic_err(
                    "Unstaking Amount is higher then actual staking amount".to_string(),
                ));
            }

            staker_info.amount = staker_info.amount - amount;
            stakers_w(deps.storage).save(for_address.as_bytes(), &staker_info)?;

            // send back amount of lp token to pair contract to send pair token back with burn
            let config = config_r(deps.storage).load()?;

            if let Some(true) = remove_liquidity {
                // SEND LP Token back to Pair Contract With Remove Liquidity
                let remove_liquidity_msg = to_binary(&AmmPairInvokeMsg::RemoveLiquidity {
                    from: Some(for_address.to_string()),
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
                    recipient: for_address.to_string(),
                    amount,
                    memo: None,
                    padding: None,
                }
                .to_cosmos_msg(&config.lp_token, vec![])?;

                messages.push(cosmos_msg);
            }
            let mut total_stake_amount = total_staked_w(deps.storage).load()?;
            total_stake_amount -= amount;
            total_staked_w(deps.storage).save(&total_stake_amount)?;

            return Ok(Response::new().add_messages(messages).add_attributes(vec![
                Attribute::new("action", "unstake"),
                Attribute::new("amount", amount),
                Attribute::new("staker", for_address.as_str()),
            ]));
        } else {
            return Err(StdError::generic_err(
                "Staking information does not exist".to_string(),
            ));
        }
    } else {
        let mut staker_info = stakers_r(deps.storage).load(for_address.as_bytes())?;
        let proxy_staking_key = &generate_proxy_staking_key(&from_address, &for_address);
        if let Some(mut proxy_staker_info) =
            proxy_staker_info_r(deps.storage).may_load(proxy_staking_key)?
        {
            // remove staker
            let mut messages: Vec<CosmosMsg> = Vec::new();
            // check if the amount is higher than what has been totally staked and proxy staked by this caller
            if amount > proxy_staker_info.amount || amount > staker_info.proxy_staked {
                return Err(StdError::generic_err(
                    "Unstaking Amount is higher then actual staking amount".to_string(),
                ));
            }

            staker_info.amount -= amount;
            staker_info.proxy_staked -= amount;
            stakers_w(deps.storage).save(for_address.as_bytes(), &staker_info)?;

            //Update the proxy stakers
            proxy_staker_info.amount -= amount;
            proxy_staker_info_w(deps.storage).save(
                &generate_proxy_staking_key(&from_address, &for_address),
                &proxy_staker_info,
            )?;

            // send back amount of lp token to pair contract to send pair token back with burn
            let config = config_r(deps.storage).load()?;

            let cosmos_msg = snip20::ExecuteMsg::Transfer {
                recipient: from_address.to_string(),
                amount,
                memo: None,
                padding: None,
            }
            .to_cosmos_msg(&config.lp_token, vec![])?;

            messages.push(cosmos_msg);

            let mut total_stake_amount = total_staked_w(deps.storage).load()?;
            total_stake_amount -= amount;
            total_staked_w(deps.storage).save(&total_stake_amount)?;
            return Ok(Response::new().add_messages(messages).add_attributes(vec![
                Attribute::new("action", "unstake"),
                Attribute::new("amount", amount),
                Attribute::new("staker", from_address.as_str()),
            ]));
        } else {
            return Err(StdError::generic_err(
                "Proxy stake for given proxy staker and staker does not exist.".to_string(),
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

// Takes the earliest of the two dates
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
    if total_staked.is_zero()
        || reward_token.last_update_time
            > last_time_reward_applicable(current_timestamp, reward_token.valid_to)?
    {
        return Ok(reward_token.reward_per_token_stored);
    }
    return Ok(reward_token.reward_per_token_stored.checked_add(
        (last_time_reward_applicable(current_timestamp, reward_token.valid_to)?
            .checked_sub(reward_token.last_update_time)?)
        .checked_mul(reward_token.reward_rate)?
        .checked_mul(MAX_DECIMALS)?
        .checked_div(total_staked)?,
    )?);
}

pub fn earned(
    balance: Uint128,
    reward_per_token: Uint128,
    reward_per_token_paid: Uint128,
    rewards: Uint128,
) -> StdResult<Uint128> {
    let mut sub = Uint128::zero();
    if reward_per_token > reward_per_token_paid {
        sub = reward_per_token.checked_sub(reward_per_token_paid)?;
    }

    Ok(balance
        .checked_mul(sub)?
        .checked_div(MAX_DECIMALS)?
        .checked_add(rewards)?)
}

pub fn update_reward(
    current_timestamp: Uint128,
    address: &Addr,
    storage: &mut dyn Storage,
    env: &Env,
) -> StdResult<()> {
    let reward_list = reward_token_list_r(storage).load()?;
    let total_staked = match total_staked_r(storage).may_load()? {
        Some(s) => s,
        None => Uint128::zero(),
    };
    for addr in &reward_list {
        // load total reward token
        if let Some(mut reward_token_info) = reward_token_w(storage).may_load(addr.as_bytes())? {
            let reward_per_token =
                reward_per_token(current_timestamp, &reward_token_info, total_staked)?;
            reward_token_info.reward_per_token_stored = reward_per_token;
            reward_token_info.last_update_time =
                last_time_reward_applicable(current_timestamp, reward_token_info.valid_to)?;

            if address.to_string() != env.contract.address.to_string() {
                if claim_reward_info_w(storage)
                    .may_load(get_user_claim_key(address.to_string(), addr.to_string()).as_bytes())?
                    .is_none()
                {
                    claim_reward_info_w(storage).save(
                        get_user_claim_key(address.to_string(), addr.to_string()).as_bytes(),
                        &ClaimRewardsInfo {
                            rewards: Uint128::zero(),
                            reward_token: reward_token_info.clone().reward_token,
                            reward_token_per_token_paid: reward_token_info
                                .clone()
                                .reward_per_token_stored,
                        },
                    )?;
                };
            }

            reward_token_w(storage).save(addr.as_bytes(), &reward_token_info)?;

            if let Some(staker_info) = stakers_r(storage).may_load(address.as_bytes())? {
                if address.to_string() != env.contract.address.to_string() {
                    //safe due to check above
                    let mut claim_reward_info = claim_reward_info_w(storage).load(
                        get_user_claim_key(address.to_string(), addr.to_string()).as_bytes(),
                    )?;
                    let new_amount = earned(
                        staker_info.amount,
                        reward_per_token,
                        claim_reward_info.reward_token_per_token_paid,
                        claim_reward_info.rewards,
                    )?;

                    claim_reward_info.rewards = new_amount;
                    claim_reward_info.reward_token_per_token_paid =
                        reward_token_info.clone().reward_per_token_stored;

                    claim_reward_info_w(storage).save(
                        get_user_claim_key(address.to_string(), addr.to_string()).as_bytes(),
                        &claim_reward_info,
                    )?;
                }
            }
        }
    }
    Ok(())
}

pub fn calculate_staker_shares(storage: &dyn Storage, amount: Uint128) -> StdResult<Decimal> {
    let total_staking_amount: Uint128 = match total_staked_r(storage).may_load()? {
        Some(staking_amount) => staking_amount,
        None => Uint128::zero(),
    };
    if total_staking_amount.is_zero() {
        return Ok(Decimal::zero());
    }

    let user_share = Decimal::from_ratio(amount, total_staking_amount);
    Ok(user_share)
}

/// Check if Address is already stored as Staker
pub fn is_address_already_staker(deps: Deps, address: Addr) -> StdResult<bool> {
    let addrs = stakers_r(deps.storage).may_load(address.as_bytes())?;
    match addrs {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

pub fn get_user_claim_key(address: String, reward_token_key: String) -> String {
    return address.to_owned() + &reward_token_key;
}
