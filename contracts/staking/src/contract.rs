use cosmwasm_std::{
    entry_point, from_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};
use shadeswap_shared::{
    core::{TokenType},
    query_auth::helpers::{authenticate_permit, PermitAuthentication},
    snip20::helpers::send_msg,
    staking::{AuthQuery, ExecuteMsg, InitMsg, InvokeMsg, QueryData, QueryMsg},
    utils::{pad_query_result, pad_response_result},
    Contract, admin::helpers::{validate_admin, AdminPermissions},
};

use crate::{
    operations::{
        claim_rewards, get_claim_reward_for_user, get_config, get_staking_stake_lp_token_info,
        proxy_stake, proxy_unstake, set_reward_token, stake, store_init_reward_token_and_timestamp,
        unstake, update_authenticator, get_reward_token_to_list,
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
        admin_auth: msg.admin_auth,
    };
    config_w(deps.storage).save(&config)?;
    prng_seed_w(deps.storage).save(&msg.prng_seed.as_slice().to_vec())?;

    // store reward token to the list
    let reward_token_address: Contract = match msg.reward_token {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => Contract {
            address: contract_addr.to_owned(),
            code_hash: token_code_hash.to_owned(),
        },
        TokenType::NativeToken { denom: _ } => {
            return Err(StdError::generic_err(
                "Invalid Token Type for Reward Token".to_string(),
            ))
        }
    };    
    store_init_reward_token_and_timestamp(
        deps.storage,
        reward_token_address.to_owned(),
        msg.daily_reward_amount,        
        msg.valid_to
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
            ExecuteMsg::ProxyUnstake { for_addr, amount } => {
                proxy_unstake(deps, env, info, for_addr, amount)
            }
            ExecuteMsg::Receive {
                from, amount, msg, ..
            } => receiver_callback(deps, env, info, from, amount, msg),
            ExecuteMsg::ClaimRewards {} => claim_rewards(deps, info, env),
            ExecuteMsg::Unstake {
                amount,
                remove_liqudity,
            } => unstake(deps, env, info, amount, remove_liqudity),
            ExecuteMsg::SetAuthenticator { authenticator } => {
                let config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                update_authenticator(deps.storage, authenticator)
            }
            ExecuteMsg::SetConfig { admin_auth } => {
                let mut config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                if let Some(admin_auth) = admin_auth {
                    config.admin_auth = admin_auth;
                }
                Ok(Response::default())
            }
            ExecuteMsg::SetRewardToken {
                reward_token,
                daily_reward_amount,
                valid_to,
            } => {
                let config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                set_reward_token(deps, env, info, reward_token, daily_reward_amount, valid_to)
            }
            ExecuteMsg::RecoverFunds {
                token,
                amount,
                to,
                msg,
            } => {
                let config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                let send_msg = match token {
                    TokenType::CustomToken {
                        contract_addr,
                        token_code_hash,
                    } => vec![send_msg(
                        to,
                        amount,
                        msg,
                        None,
                        None,
                        &Contract {
                            address: contract_addr,
                            code_hash: token_code_hash,
                        },
                    )?],
                    TokenType::NativeToken { denom } => vec![CosmosMsg::Bank(BankMsg::Send {
                        to_address: to.to_string(),
                        amount: vec![Coin::new(amount.u128(), denom)],
                    })],
                };

                Ok(Response::new().add_messages(send_msg))
            }
        },
        BLOCK_SIZE,
    )
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
            InvokeMsg::ProxyStake { for_addr } => {
                if config.lp_token.address != info.sender {
                    return Err(StdError::generic_err("Sender was not LP Token".to_string()));
                }
                proxy_stake(deps, env, info, amount, from, for_addr)
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
        },
        BLOCK_SIZE,
    )
}

pub fn auth_queries(deps: Deps, _env: Env, msg: AuthQuery, user: Addr) -> StdResult<Binary> {
    match msg {
        AuthQuery::GetClaimReward { time } => get_claim_reward_for_user(deps, user, time),
        AuthQuery::GetStakerLpTokenInfo {} => get_staking_stake_lp_token_info(deps, user),
        AuthQuery::GetRewardTokens {  } => get_reward_token_to_list(deps.storage),
    }
}
