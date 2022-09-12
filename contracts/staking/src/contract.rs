use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, StdResult, Response, Addr, CosmosMsg, WasmMsg, Attribute, Uint128, Binary, from_binary, StdError, Deps};
use shadeswap_shared::{staking::{InitMsg, ExecuteMsg, InvokeMsg, QueryMsg}, core::{ContractLink, admin_w}};

use crate::{state::{Config, config_w, prng_seed_w, config_r, stakers_r}, operations::{claim_rewards, set_lp_token, unstake, set_view_key, stake, get_claim_reward_for_user, get_staking_contract_owner, get_staking_stake_lp_token_info, get_staking_reward_token_balance, get_staker_reward_info, get_config}};



pub const BLOCK_SIZE: usize = 256;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InitMsg,
)-> StdResult<Response> {

    let config = Config {
        contract_owner: _info.sender.clone(),
        daily_reward_amount: msg.staking_amount,
        reward_token: msg.reward_token.clone(),
        lp_token: ContractLink { 
            address: Addr::unchecked("".to_string()),
            code_hash: "".to_string()
        }
    };
    config_w(deps.storage).save(&config)?;
    admin_w(deps.storage).save(&_info.sender)?;
    prng_seed_w(deps.storage).save(&msg.prng_seed.as_slice().to_vec())?;

    Ok(Response::new().add_attributes(
        vec![
           Attribute::new("staking_contract_addr", env.contract.address),
           Attribute::new("reward_token", msg.reward_token.to_string()),
           Attribute::new("daily_reward_amount", msg.staking_amount),
        ]))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive {
            from, amount, msg, ..
        } => receiver_callback(deps, env, info, from, amount, msg),      
        ExecuteMsg::ClaimRewards { } => {
            claim_rewards(deps, info, env)
        }
        ExecuteMsg::SetLPToken {lp_token} => set_lp_token(deps, env, lp_token),
        ExecuteMsg::Unstake {amount, remove_liqudity} => unstake(deps,env, info, amount, remove_liqudity),
        ExecuteMsg::SetVKForStaker { key} => set_view_key(deps, env, info, key),
    }    
}

fn receiver_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let msg = msg.ok_or_else(|| {
        StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
    })?;

    let config = config_r(deps.storage).load()?;
    match from_binary(&msg)? {       
        InvokeMsg::Stake { from, amount } => {
            if config.lp_token.address != info.sender {
                return Err(StdError::generic_err("".to_string()));
            }
            stake(deps, env,info, amount, from)
        }
    }
}


pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {    
        QueryMsg::GetConfig {  } => {get_config(deps)},
        QueryMsg::GetClaimReward{ staker, time, key  } =>{get_claim_reward_for_user(deps, staker, key,time)},
        QueryMsg::GetContractOwner {} => {get_staking_contract_owner(deps, env)},
        QueryMsg::GetStakerLpTokenInfo { key, staker } => {get_staking_stake_lp_token_info(deps, staker, key)},
        QueryMsg::GetRewardTokenBalance {key, address} => {get_staking_reward_token_balance(env, deps, key, address)},
        QueryMsg::GetStakerRewardTokenBalance { key, staker } => {get_staker_reward_info(deps, key, staker)},
    }
}

