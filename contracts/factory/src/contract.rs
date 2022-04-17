use crate::state::load_amm_pairs;
use crate::state::save_amm_pairs;
use secret_toolkit::utils::HandleCallback;
use secret_toolkit::utils::InitCallback;
use shadeswap_shared::Pagination;
use shadeswap_shared::TokenPair;
use shadeswap_shared::amm_pair::AMMPair;
use shadeswap_shared::msg::factory::QueryResponse;
use shadeswap_shared::{
    fadroma::{
        admin::{
            assert_admin, handle as admin_handle, load_admin, query as admin_query, save_admin,
            DefaultImpl as AdminImpl,
        },
        require_admin::require_admin,
        scrt::{
            log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
            InitResponse, Querier, StdError, StdResult, Storage, WasmMsg,
        },
        scrt_callback::Callback,
        scrt_link::ContractLink,
        scrt_migrate,
        scrt_migrate::get_status,
        scrt_storage::{load, remove, save},
        with_status,
    },
    msg::amm_pair::InitMsg as AMMPairInitMsg,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{config_read, config_write, Config};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    config_write(deps, &Config::from_init_msg(msg));

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    return match msg {
        HandleMsg::CreateAMMPair {} => create_pair(deps, env),
        HandleMsg::SetConfig { .. } => set_config(deps, env, msg),
        HandleMsg::AddAMMPairs { ammPairs } => add_amm_pairs(deps, env, ammPairs),
        HandleMsg:: RegisterAMMPair{} => todo!()
    };
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => get_config(deps),
        QueryMsg::ListAMMPairs { pagination } => list_pairs(deps, pagination),
        QueryMsg::GetAMMPairAddress {  } => todo!(),
        QueryMsg::GetAMMSettings => todo!(),
    }
}

pub fn add_amm_pairs<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amm_pairs: Vec<AMMPair<HumanAddr>>
) -> StdResult<HandleResponse> {

    save_amm_pairs(deps, amm_pairs)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

pub fn list_pairs<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination,
) -> StdResult<Binary> {
    let amm_pairs = load_amm_pairs(deps, pagination)?;

    to_binary(&QueryResponse::ListAMMPairs {  amm_pairs })
}

pub fn set_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    if let HandleMsg::SetConfig { pair_contract } = msg {
        let mut config = config_read(&deps)?;

        if let Some(new_value) = pair_contract {
            config.pair_contract = new_value;
        }

        config_write(deps, &config)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: None,
        })
    } else {
        unreachable!()
    }
}

pub fn get_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let Config {
        pair_contract,
        amm_settings
    } = config_read(deps)?;

    to_binary(&QueryResponse::Config {
        pair_contract,
        amm_settings
    })
}

pub fn create_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {
    let mut config = config_read(&deps)?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: config.pair_contract.id,
            callback_code_hash: config.pair_contract.code_hash,
            send: vec![],
            label: "test".to_string(),
            msg: to_binary(&AMMPairInitMsg {})?,
        })],
        log: vec![],
        data: None,
    })
}
