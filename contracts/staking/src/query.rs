use cosmwasm_std::{Binary, StdResult, StdError, Deps, Uint128, Addr, Storage, to_binary};
use shadeswap_shared::core::TokenType;
use shadeswap_shared::staking::{QueryResponse, ClaimableInfo, RewardTokenInfo};
use shadeswap_shared::utils::asset::Contract;
use crate::operations::{get_reward_tokens_info, earned};
use crate::state::{stakers_r, total_staked_r, config_r, claim_reward_info_r, reward_token_w, reward_token_r};

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
            total_staked_lp_token: total_staked_r(deps.storage).may_load()?.map_or_else(|| Uint128::zero(), |v| v)
        };
        return to_binary(&response);
    } else {
        return Err(StdError::generic_err("Invalid reward token"));
    }
}

pub fn claim_reward_for_user(deps: Deps, staker: Addr, time: Uint128) -> StdResult<Binary> {
    // load stakers   
    let mut result_list: Vec<ClaimableInfo> = Vec::new();
    for claim_info in claim_reward_info_r(deps.storage)
        .load(staker.clone().as_bytes())?
        .iter()
    {
        let reward_token_info: RewardTokenInfo = reward_token_r(deps.storage).load(claim_info.1.reward_token.unique_key().as_bytes())?;
        result_list.push(ClaimableInfo {
            token_address: claim_info.1.reward_token.unique_key(),
            amount: earned(&staker, claim_info.1.amount, reward_token_info.reward_per_token_stored, claim_info.1.reward_token_per_token_paid, deps.storage)?
        });
    }
    to_binary(&QueryResponse::GetClaimReward {
        claimable_rewards: result_list,
    })
}

pub fn staking_stake_lp_token_info(deps: Deps, staker: Addr) -> StdResult<Binary> {
    let staker_amount = stakers_r(deps.storage).may_load(&staker.as_bytes())?.map_or_else(|| Uint128::zero(), |v| v.amount);

    let response_msg = QueryResponse::GetStakerLpTokenInfo {
        staked_lp_token: staker_amount,
        total_staked_lp_token: total_staked_r(deps.storage).may_load()?.map_or_else(|| Uint128::zero(), |v| v),
    };
    to_binary(&response_msg)
}

pub fn reward_token_list(storage:& dyn Storage) 
    -> StdResult<Binary> {
        let list: Vec<RewardTokenInfo> = get_reward_tokens_info(storage)?;
        let mut response: Vec<RewardTokenInfo> = vec![];
        for i in list.iter(){
            response.push(i.to_owned())
        }
        to_binary(&QueryResponse::GetRewardTokens{
            tokens: response
        })
}