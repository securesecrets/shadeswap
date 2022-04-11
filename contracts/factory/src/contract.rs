use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdError, StdResult, Storage, CosmosMsg, WasmMsg
};
use secret_toolkit::utils::InitCallback;

use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config_write, config_read, State};

use shadeswap_shared::{
    msg::{
        pair::{InitMsg as PairInitMsg}
    },
    fadroma::{
        scrt_callback::Callback,
        scrt_link::{ContractLink, ContractInstantiationInfo},
        scrt::{ Decimal, HumanAddr, Uint128},
        scrt_migrate::ContractStatusLevel,
    }
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        pair_contract: msg.pair_contract
    };

    config_write(&mut deps.storage).save(&state)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    return match msg {
        HandleMsg::CreatePair {} => create_pair(deps,env),
        HandleMsg::SetConfig { .. } => set_config(deps, env, msg)
    };
}

pub fn set_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    if let HandleMsg::SetConfig {
        pair_contract
    } = msg
    {
        let mut config = config_read(&deps.storage).load()?;

        if let Some(new_value) = pair_contract {
            config.pair_contract = new_value;
        }

        config_write(&mut deps.storage).save(&config)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: None,
        })
    } else {
        unreachable!()
    }
}

pub fn create_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {

   let mut config = config_read(&deps.storage).load()?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: config.pair_contract.id,
            callback_code_hash: config.pair_contract.code_hash,
            send: vec![],
            label: "test".to_string(),
            msg: to_binary(&PairInitMsg {
                count: 100 
            })?,
        })],
        log: vec![],
        data: None
    })
}

/*
pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}
*/