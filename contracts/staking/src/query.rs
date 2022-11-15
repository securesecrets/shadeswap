use cosmwasm_std::{Binary, StdResult, StdError, Deps, Uint128, Addr, Storage, to_binary};
use shadeswap_shared::core::TokenType;
use shadeswap_shared::staking::{QueryResponse, ClaimableInfo, RewardTokenInfo};
use shadeswap_shared::utils::asset::Contract;
use crate::operations::{find_claimable_reward_for_staker_by_reward_token, calculate_incremental_staking_reward, get_reward_tokens_info, calculate_staker_shares};
use crate::state::{stakers_r, total_staked_r, config_r};

pub fn get_config(deps: Deps) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = config.reward_token.clone()
    {
        let response = QueryResponse::Config {
            reward_token: Contract {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
            lp_token: config.lp_token.clone(),
            daily_reward_amount: config.daily_reward_amount.clone(),
            amm_pair: config.amm_pair.to_string(),
            admin_auth: config.admin_auth,
        };
        return to_binary(&response);
    } else {
        return Err(StdError::generic_err("Invalid reward token"));
    }
}

pub fn get_claim_reward_for_user(deps: Deps, staker: Addr, time: Uint128) -> StdResult<Binary> {
    // load stakers   
    let mut result_list: Vec<ClaimableInfo> = Vec::new();
    let staker_info = stakers_r(deps.storage).load(staker.as_bytes())?;
    let reward_token_list: Vec<RewardTokenInfo> = get_reward_tokens_info(deps.storage)?;
    let percentage = calculate_staker_shares(deps.storage, staker_info.amount)?;
    for reward_token in reward_token_list.iter() {
        if reward_token.valid_to < staker_info.last_time_updated {
            let reward: Uint128;
            println!("time {} - valid_to {}", time.to_string(), reward_token.valid_to.to_string());
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
        else{
            let reward = calculate_incremental_staking_reward(
                percentage,
                staker_info.last_time_updated,
                time,
                reward_token.daily_reward_amount,
            )?;
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

pub fn get_staking_stake_lp_token_info(deps: Deps, staker: Addr) -> StdResult<Binary> {
    let staker_amount = stakers_r(deps.storage).may_load(&staker.as_bytes())?.map_or_else(|| Uint128::zero(), |v| v.amount);

    let response_msg = QueryResponse::StakerLpTokenInfo {
        staked_lp_token: staker_amount,
        total_staked_lp_token: total_staked_r(deps.storage).may_load()?.map_or_else(|| Uint128::zero(), |v| v),
    };
    to_binary(&response_msg)
}

pub fn get_reward_token_to_list(storage:& dyn Storage) 
    -> StdResult<Binary> {
        let list: Vec<RewardTokenInfo> = get_reward_tokens_info(storage)?;
        let mut response: Vec<RewardTokenInfo> = vec![];
        for i in list.iter(){
            response.push(i.to_owned())
        }
        to_binary(&QueryResponse::RewardTokens{
            tokens: response
        })
}