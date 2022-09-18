use crate::{
    operations::{add_amm_pairs, create_pair, register_amm_pair, set_config, list_pairs, query_amm_pair_address, query_amm_settings},
    state::{config_r, config_w, ephemeral_storage_w, prng_seed_w, Config, ephemeral_storage_r},
};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Querier,
    Reply, Response, StdError, StdResult, Storage, SubMsgResult, WasmMsg,
};
use shadeswap_shared::{
    amm_pair::{self, AMMPair},
    core::{admin_w, apply_admin_guard, Callback, ContractLink, admin_r},
    msg::{
        amm_pair::InitMsg as AMMPairInitMsg,
        factory::{ExecuteMsg, InitMsg, QueryMsg, QueryResponse},
    }
};

pub const INSTANTIATE_REPLY_ID: u64 = 1u64;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    prng_seed_w(deps.storage).save(&msg.prng_seed)?;
    config_w(deps.storage).save(&Config::from_init_msg(msg))?;
    admin_w(deps.storage).save(&_info.sender)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    return match msg {
        ExecuteMsg::CreateAMMPair {
            pair,
            entropy,
            staking_contract,
        } => create_pair(deps, env, pair, info.sender, entropy, staking_contract),
        ExecuteMsg::SetConfig { .. } => set_config(deps, env, msg),
        ExecuteMsg::AddAMMPairs { amm_pairs } => {
            apply_admin_guard(&info.sender, deps.storage)?;
            add_amm_pairs(deps.storage, amm_pairs)
        }
        ExecuteMsg::SetFactoryAdmin { admin } => {
            apply_admin_guard(&info.sender, deps.storage)?;
            admin_w(deps.storage).save(&Addr::unchecked(admin))?;
            Ok(Response::default())
        }
    };
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {
            let Config {
                pair_contract,
                amm_settings,
                lp_token_contract,
            } = config_r(deps.storage).load()?;

            to_binary(&QueryResponse::GetConfig {
                pair_contract,
                amm_settings,
                lp_token_contract,
            })
        },
        QueryMsg::ListAMMPairs { pagination } => list_pairs(deps, pagination),
        QueryMsg::GetAMMPairAddress { pair } => query_amm_pair_address(&deps, pair),
        QueryMsg::GetAMMSettings {} => query_amm_settings(deps),
        QueryMsg::GetAdmin {} => {
            let admin_address = admin_r(deps.storage).load()?;
            to_binary(&QueryResponse::GetAdminAddress {
                address: admin_address.to_string(),
            })
        }
    }
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    //Ok(Response::default())
    match (msg.id, msg.result) {
        (INSTANTIATE_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                let contract_address = String::from_utf8(x.to_vec())?;
                let config = ephemeral_storage_r(deps.storage).load()?;
                register_amm_pair(deps.storage, _env, AMMPair{ pair: config.pair, address: Addr::unchecked(contract_address), enabled: true } )?;
                ephemeral_storage_w(deps.storage).remove();
                Ok(Response::default())
            }
            None => Err(StdError::generic_err(format!("Expecting contract id"))),
        },
        _ => Err(StdError::generic_err(format!("Unknown reply id"))),
    }
}
