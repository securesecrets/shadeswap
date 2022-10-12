use cosmwasm_std::{
    entry_point, Addr, Attribute, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, to_binary, Binary, StdError, from_binary};
use shadeswap_shared::{
    core::{admin_r, admin_w, apply_admin_guard, ContractLink, TokenType},
    query_auth::helpers::{authenticate_permit, PermitAuthentication},
    staking::{AuthQuery, ExecuteMsg, InitMsg, InvokeMsg, QueryData, QueryMsg},
    utils::{pad_query_result, pad_response_result},
};

use shadeswap_shared::staking::QueryResponse;

use crate::{
    operations::{
        claim_rewards, get_claim_reward_for_user, get_config, get_staking_stake_lp_token_info,
        set_reward_token, stake, store_init_reward_token_and_timestamp, unstake,
        update_authenticator,
    },
    state::{config_r, config_w, prng_seed_w, Config},
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
        amm_pair: _info.sender.clone(),
        daily_reward_amount: msg.daily_reward_amount,
        reward_token: msg.reward_token.to_owned(),
        lp_token: msg.lp_token,
        authenticator: msg.authenticator,
    };
    config_w(deps.storage).save(&config)?;
    admin_w(deps.storage).save(&_info.sender)?;
    prng_seed_w(deps.storage).save(&msg.prng_seed.as_slice().to_vec())?;

    // store reward token to the list
    let reward_token_address: ContractLink = match msg.reward_token {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => ContractLink {
            address: contract_addr.to_owned(),
            code_hash: token_code_hash.to_owned(),
        },
        TokenType::NativeToken { denom: _ } => {
            return Err(StdError::generic_err(
                "Invalid Token Type for Reward Token".to_string(),
            ))
        }
    };
    let current_timestamp = Uint128::new((env.block.time.seconds() * 1000) as u128);
    store_init_reward_token_and_timestamp(
        deps.storage,
        reward_token_address.to_owned(),
        msg.daily_reward_amount,
        current_timestamp,
    )?;

    let mut response = Response::new();
    response.data = Some(env.contract.address.as_bytes().into());
    Ok(response.add_attributes(vec![
        Attribute::new("staking_contract_addr", env.contract.address),
        Attribute::new("reward_token", reward_token_address.address.to_string()),
        Attribute::new("daily_reward_amount", msg.daily_reward_amount),
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
            } => unstake(deps, env, info, amount, remove_liqudity),
            ExecuteMsg::SetAuthenticator { authenticator } => {
                apply_admin_guard(&info.sender, deps.storage)?;
                update_authenticator(deps.storage, authenticator)
            }
            ExecuteMsg::SetAdmin { admin } => {
                apply_admin_guard(&info.sender, deps.storage)?;
                admin_w(deps.storage).save(&admin)?;
                Ok(Response::default())
            }
            ExecuteMsg::SetRewardToken {
                reward_token,
                daily_reward_amount,
                valid_to,
            } => {
                apply_admin_guard(&info.sender, deps.storage)?;
                set_reward_token(deps, env, info, reward_token, daily_reward_amount, valid_to)
            }
        },
        BLOCK_SIZE,
    )
}

fn receiver_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _from: Addr,
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
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => get_config(deps),
            QueryMsg::GetContractOwner {} => todo!(),
            QueryMsg::WithPermit { permit, query } => {
                let config = config_r(deps.storage).load()?;
                let res: PermitAuthentication<QueryData> =
                    authenticate_permit(deps, permit, &deps.querier, config.authenticator)?;

                if res.revoked {
                    return Err(StdError::generic_err("".to_string()));
                }

                auth_queries(deps, env, query, res.sender)
            }
            QueryMsg::GetAdmin {} => to_binary(&QueryResponse::GetAdmin {
                admin: admin_r(deps.storage).load()?,
            }),
        },
        BLOCK_SIZE,
    )
}

pub fn auth_queries(deps: Deps, _env: Env, msg: AuthQuery, user: Addr) -> StdResult<Binary> {
    match msg {
        AuthQuery::GetClaimReward { time } => get_claim_reward_for_user(deps, user, time),
        AuthQuery::GetStakerLpTokenInfo {} => get_staking_stake_lp_token_info(deps, user),
    }
}
