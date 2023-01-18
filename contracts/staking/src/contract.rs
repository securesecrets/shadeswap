use cosmwasm_std::{
    entry_point, from_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};
use shadeswap_shared::{
    core::{TokenType},
    query_auth::helpers::{authenticate_permit, PermitAuthentication},
    snip20::helpers::{send_msg, register_receive},
    staking::{AuthQuery, ExecuteMsg, InitMsg, InvokeMsg, QueryData, QueryMsg},
    utils::{pad_query_result, pad_response_result},
    Contract, admin::helpers::{validate_admin, AdminPermissions},
};

use crate::{
    operations::{
        claim_rewards, set_reward_token, stake,
        unstake, update_authenticator, 
    },
    query,
    state::{config_r, config_w, prng_seed_w, Config, reward_token_list_w},
};

pub const BLOCK_SIZE: usize = 256;
pub const SHADE_STAKING_VIEWKEY: &str = "SHADE_STAKING_VIEWKEY";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    let config = Config {
        amm_pair: info.sender.clone(),
        daily_reward_amount: msg.daily_reward_amount,
        reward_token: msg.reward_token.to_owned(),
        lp_token: msg.lp_token.clone(),
        authenticator: msg.authenticator,
        admin_auth: msg.admin_auth,
    };
    config_w(deps.storage).save(&config)?;
    prng_seed_w(deps.storage).save(&msg.prng_seed.as_slice().to_vec())?;

    let mut messages: Vec<CosmosMsg> = vec![];

    messages.push(register_receive(
        env.contract.code_hash.clone(),
        None,
        &msg.lp_token
    )?);

    // store reward token to the list
    let reward_token_address: Contract = match msg.reward_token.clone() {
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

    let reward_token_list: Vec<String> = Vec::new();
    reward_token_list_w(deps.storage).save(&reward_token_list)?;

    set_reward_token(deps, &env, msg.daily_reward_amount, msg.reward_token, msg.valid_to)?;

    let mut response = Response::new();
    response.data = Some(env.contract.address.as_bytes().into());
    Ok(response.add_messages(messages).add_attributes(vec![
        Attribute::new("staking_contract_addr", env.contract.address),
        Attribute::new("reward_token", reward_token_address.address.to_string()),
        Attribute::new("daily_reward_amount", msg.daily_reward_amount),
    ]))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_response_result(
        match msg {
            // Allow another sender to unstake for a specific user for which they previously proxy staked for.
            ExecuteMsg::ProxyUnstake { for_addr, amount } => {
                let checked_for_addr = deps.api.addr_validate(&for_addr)?;
                unstake(deps, &env, &info.sender, &checked_for_addr, amount, Some(false))
            }
            ExecuteMsg::Receive {
                from, amount, msg, ..
            } => {
                let checked_from = deps.api.addr_validate(&from)?;
                receiver_callback(deps, env, info, checked_from, amount, msg)
            },
            ExecuteMsg::ClaimRewards {} => claim_rewards(deps, Uint128::new((env.block.time.seconds()) as u128),&info.sender, &env),
            ExecuteMsg::Unstake {
                amount,
                remove_liquidity,
            } => unstake(deps, &env, &info.sender,  &info.sender, amount, remove_liquidity),
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
                set_reward_token(deps, &env,  daily_reward_amount, reward_token ,valid_to)
            }
            // This can be used by admins to recover any funds that were sent accidentally to staking contract.
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
                        deps.api.addr_validate(&to)?,
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
                let checked_from = deps.api.addr_validate(&from)?;
                stake(deps, &env, &info, amount, &checked_from, &checked_from)
            }
            // Allow another sender to stake for a specific user. Only the sender can unstake the funds using proxy unstake
            InvokeMsg::ProxyStake { for_addr } => {
                if config.lp_token.address != info.sender {
                    return Err(StdError::generic_err("Sender was not LP Token".to_string()));
                }
                let checked_for_addr = deps.api.addr_validate(&for_addr)?;
                stake(deps, &env, &info, amount, &from, &checked_for_addr)
            }
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => query::config(deps),
            QueryMsg::WithPermit { permit, query } => {
                let config = config_r(deps.storage).load()?;
                let res: PermitAuthentication<QueryData> =
                    authenticate_permit(deps, permit, &deps.querier, config.authenticator)?;

                if res.revoked {
                    return Err(StdError::generic_err("Permit has been revoked".to_string()));
                }

                auth_queries(deps, env, query, res.sender)
            },
            QueryMsg::GetRewardTokens {  } => query::reward_token_list(deps.storage),
        },
        BLOCK_SIZE,
    )
}

pub fn auth_queries(deps: Deps, _env: Env, msg: AuthQuery, user: Addr) -> StdResult<Binary> {
    match msg {
        AuthQuery::GetClaimReward { time } => query::claim_reward_for_user(deps, user, time),
        AuthQuery::GetStakerLpTokenInfo {} => query::staking_stake_lp_token_info(deps, user),       
    }
}
