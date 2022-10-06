use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Attribute, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use shadeswap_shared::{
    core::{admin_w, ContractLink},
    msg::amm_pair::ExecuteMsg as AmmPairExecuteMsg,
    staking::{ExecuteMsg, InitMsg, InvokeMsg, QueryMsg, AuthQuery, QueryData}, query_auth::helpers::{authenticate_permit, PermitAuthentication}, utils::{pad_response_result, pad_query_result},
};

use crate::{
    operations::{
        claim_rewards, get_claim_reward_for_user, get_config, get_staker_reward_info,
        get_staking_contract_owner, get_staking_stake_lp_token_info, set_view_key, stake, unstake,
    },
    state::{config_r, config_w, prng_seed_w, stakers_r, Config},
};

pub const BLOCK_SIZE: usize = 256;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    let config = Config {
        contract_owner: _info.sender.clone(),
        daily_reward_amount: msg.staking_amount,
        reward_token: msg.reward_token.clone(),
        lp_token: msg.lp_token,
        authenticator: None
    };
    config_w(deps.storage).save(&config)?;
    admin_w(deps.storage).save(&_info.sender)?;
    prng_seed_w(deps.storage).save(&msg.prng_seed.as_slice().to_vec())?;

    let mut response = Response::new();
    response.data = Some(env.contract.address.as_bytes().into());

    Ok(response
        .add_attributes(vec![
            Attribute::new("staking_contract_addr", env.contract.address),
            Attribute::new("reward_token", msg.reward_token.to_string()),
            Attribute::new("daily_reward_amount", msg.staking_amount),
        ]))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_response_result(
    match msg {
        ExecuteMsg::Receive {
            from, amount, msg, ..
        } => receiver_callback(deps, env, info, from, amount, msg),
        ExecuteMsg::ClaimRewards {} => claim_rewards(deps, info, env),
        ExecuteMsg::Unstake {
            amount,
            remove_liqudity,
        } => unstake(deps, env, info, amount, remove_liqudity)
    }, BLOCK_SIZE)
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
    pad_response_result(
    match from_binary(&msg)? {
        InvokeMsg::Stake { from } => {
            if config.lp_token.address != info.sender {
                return Err(StdError::generic_err("Sender was not LP Token".to_string()));
            }
            stake(deps, env, info, amount, from)
        }
    }, BLOCK_SIZE)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(match msg {
        QueryMsg::GetConfig {} => get_config(deps),
        QueryMsg::GetContractOwner {} => todo!(),
        QueryMsg::WithPermit { permit, query } => {
            let res: PermitAuthentication<QueryData> = authenticate_permit(deps, permit, &deps.querier, None)?;

            if res.revoked {
                return Err(StdError::generic_err("".to_string()));
            }

            auth_queries(deps, env, query, res.sender)
        },
    }, BLOCK_SIZE)
}

pub fn auth_queries(deps: Deps, env: Env, msg: AuthQuery, user: Addr) -> StdResult<Binary> {
    match msg {
        AuthQuery::GetClaimReward { time } => {
            get_claim_reward_for_user(deps, user, time)
        },
        AuthQuery::GetStakerLpTokenInfo { } => {
            get_staking_stake_lp_token_info(deps, user)
        }
    }
}

