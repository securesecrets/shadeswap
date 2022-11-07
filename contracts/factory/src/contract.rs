use crate::{
    operations::{
        add_amm_pairs, create_pair, list_pairs, query_amm_pair_address, register_amm_pair,
        set_config,
    },
    state::{config_r, config_w, ephemeral_storage_r, ephemeral_storage_w, prng_seed_w, Config},
};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, SubMsgResult,
};
use shadeswap_shared::{
    admin::helpers::{validate_admin, AdminPermissions},
    amm_pair::AMMPair,
    core::ViewingKey,
    msg::factory::{ExecuteMsg, InitMsg, QueryMsg, QueryResponse},
    utils::{pad_query_result, pad_response_result},
    BLOCK_SIZE,
};

pub const INSTANTIATE_REPLY_ID: u64 = 1u64;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    prng_seed_w(deps.storage).save(&msg.prng_seed)?;
    config_w(deps.storage).save(&Config::from_init_msg(msg))?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_response_result(
        match msg {
            //Only admins can create pairs via factory
            ExecuteMsg::CreateAMMPair {
                pair,
                entropy,
                staking_contract,
            } => {
                let config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                create_pair(
                    deps,
                    env,
                    &info,
                    pair,
                    entropy,
                    staking_contract
                )
            }
            ExecuteMsg::SetConfig {
                pair_contract,
                lp_token_contract,
                amm_settings,
                api_key,
                admin_auth,
            } => {
                let config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                set_config(
                    pair_contract,
                    lp_token_contract,
                    amm_settings,
                    deps.storage,
                    api_key,
                    admin_auth
                )
            }
            ExecuteMsg::AddAMMPairs { amm_pairs } => {
                let config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                add_amm_pairs(deps.storage, amm_pairs)
            },
            ExecuteMsg::RegisterAMMPair { pair, signature } => {
                let config = ephemeral_storage_r(deps.storage).load()?;
                if config.key != signature {
                    return Err(StdError::generic_err("Invalid signature given".to_string()));
                }
                if pair != config.pair {
                    return Err(StdError::generic_err(
                        "Provided pair is not equal.".to_string(),
                    ));
                }
                ephemeral_storage_w(deps.storage).remove();
                register_amm_pair(
                    deps.storage,
                    env,
                    AMMPair {
                        pair: config.pair,
                        address: info.sender,
                        enabled: true,
                    },
                )
            }
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::GetConfig {} => {
                let Config {
                    pair_contract,
                    amm_settings,
                    lp_token_contract,
                    api_key: _,
                    authenticator,
                    admin_auth
                } = config_r(deps.storage).load()?;
                to_binary(&QueryResponse::GetConfig {
                    pair_contract,
                    amm_settings,
                    lp_token_contract,
                    authenticator,
                    admin_auth
                })
            }
            QueryMsg::ListAMMPairs { pagination } => list_pairs(deps, pagination),
            QueryMsg::GetAMMPairAddress { pair } => query_amm_pair_address(&deps, pair),
            QueryMsg::AuthorizeApiKey { api_key } => {
                let config = config_r(deps.storage).load()?;
                to_binary(&QueryResponse::AuthorizeApiKey {
                    authorized: config.api_key == ViewingKey(api_key),
                })
            }
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    pad_response_result(
        match (msg.id, msg.result) {
            (INSTANTIATE_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
                Some(x) => {
                    let contract_address = String::from_utf8(x.to_vec())?;
                    let config = ephemeral_storage_r(deps.storage).load()?;
                    register_amm_pair(
                        deps.storage,
                        _env,
                        AMMPair {
                            pair: config.pair,
                            address: deps.api.addr_validate(&contract_address)?,
                            enabled: true,
                        },
                    )?;
                    ephemeral_storage_w(deps.storage).remove();
                    Ok(Response::default())
                }
                None => Err(StdError::generic_err(format!("Expecting contract id"))),
            },
            _ => Err(StdError::generic_err(format!("Unknown reply id"))),
        },
        BLOCK_SIZE,
    )
}
