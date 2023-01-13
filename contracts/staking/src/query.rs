use crate::operations::{earned, get_reward_tokens_info, get_user_claim_key, reward_per_token};
use crate::state::{
    claim_reward_info_r, config_r, reward_token_list_r, reward_token_r, stakers_r,
    total_staked_r,
};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, StdError, StdResult, Storage, Uint128};
use shadeswap_shared::core::TokenType;
use shadeswap_shared::staking::{ClaimableInfo, QueryResponse, RewardTokenInfo};
use shadeswap_shared::utils::asset::Contract;

pub fn config(deps: Deps) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = config.reward_token.clone()
    {
        let response = QueryResponse::GetConfig {
            reward_token: Contract {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
            lp_token: config.lp_token.clone(),
            daily_reward_amount: config.daily_reward_amount.clone(),
            amm_pair: config.amm_pair.to_string(),
            admin_auth: config.admin_auth,
            total_staked_lp_token: total_staked_r(deps.storage)
                .may_load()?
                .map_or_else(|| Uint128::zero(), |v| v),
        };
        return to_binary(&response);
    } else {
        return Err(StdError::generic_err("Invalid reward token"));
    }
}

pub fn claim_reward_for_user(deps: Deps, staker: Addr, time: Uint128) -> StdResult<Binary> {
    // load stakers
    let mut result_list: Vec<ClaimableInfo> = Vec::new();

    let reward_list = reward_token_list_r(deps.storage).load()?;
    let staker_info_option = stakers_r(deps.storage).may_load(staker.as_bytes())?;
    let total_staked = match total_staked_r(deps.storage).may_load()? {
        Some(s) => s,
        None => Uint128::zero(),
    };

    match staker_info_option {
        Some(staker_info) => {
            for addr in &reward_list {
                let key = get_user_claim_key(staker.to_string(), addr.to_string());
                let claim_info_option =
                    claim_reward_info_r(deps.storage).may_load(key.as_bytes())?;

                match claim_info_option {
                    Some(claim_info) => {
                        let reward_token_info: RewardTokenInfo = reward_token_r(deps.storage)
                            .load(claim_info.reward_token.unique_key().as_bytes())?;

                        result_list.push(ClaimableInfo {
                            token_address: claim_info.reward_token.unique_key(),
                            amount: earned(
                                staker_info.amount,
                                reward_per_token(time, &reward_token_info, total_staked)?,
                                claim_info.reward_token_per_token_paid,
                                claim_info.rewards,
                            )?
                        });
                    }
                    None => (),
                }
            }
        }
        None => (),
    }
    to_binary(&QueryResponse::GetClaimReward {
        claimable_rewards: result_list,
    })
}

pub fn staking_stake_lp_token_info(deps: Deps, staker: Addr) -> StdResult<Binary> {
    let staker_amount = stakers_r(deps.storage)
        .may_load(&staker.as_bytes())?
        .map_or_else(|| Uint128::zero(), |v| v.amount);

    let response_msg = QueryResponse::GetStakerLpTokenInfo {
        staked_lp_token: staker_amount,
        total_staked_lp_token: total_staked_r(deps.storage)
            .may_load()?
            .map_or_else(|| Uint128::zero(), |v| v),
    };
    to_binary(&response_msg)
}

pub fn reward_token_list(storage: &dyn Storage) -> StdResult<Binary> {
    let list: Vec<RewardTokenInfo> = get_reward_tokens_info(storage)?;
    let mut response: Vec<RewardTokenInfo> = vec![];
    for i in list.iter() {
        response.push(i.to_owned())
    }
    to_binary(&QueryResponse::GetRewardTokens { tokens: response })
}
