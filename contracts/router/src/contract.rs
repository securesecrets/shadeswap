
use std::borrow::BorrowMut;

use shadeswap_shared::{
    fadroma::{
        admin::{
            assert_admin, handle as admin_handle, load_admin, query as admin_query, save_admin,
            DefaultImpl as AdminImpl,
        },
        require_admin::require_admin,
        scrt::{
            log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
            InitResponse, Querier, StdError, StdResult, Storage, WasmMsg
        },
        scrt_callback::Callback,
        scrt_link::ContractLink,
        scrt_migrate,
        scrt_migrate::get_status,
        scrt_storage::{load, remove, save},
        with_status, debug_print, HandleResult, WasmQuery, QueryRequest, ContractInstantiationInfo, self, Empty,
    },
    msg::factory:: {
        QueryMsg as FactoryQueryMsg,
        QueryResponse as FactoryQueryResponse
    }, TokenType, TokenPair,
    msg::amm_pair::{
        InvokeMsg
    }, amm_pair::AMMSettings
};

use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config_write, config_read, Config};
use cosmwasm_std::{ContractInfo, Uint128};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    config_write(deps, &Config::from_init_msg(msg));

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {}
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {  } => todo!(),
    }
}

pub fn swap_exact_tokens_for_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    querier: &impl Querier,
    amountIn: Uint128,
    amountOutMin: Uint128,
    path: &[TokenType<HumanAddr>],
    to: ContractInfo,
) -> HandleResult {
    let mut messages: Vec<CosmosMsg> = vec![];
    //Validates whether the amount received is greater then the amountOutMin
    let config = config_read(deps)?;
    let factory_config = query_factory_config(querier, config.factory_address.clone())?;

    for x in 0..(path.len()-1)
    {
        let amm_address = query_token_addr(querier,&path[0], &path[1], config.factory_address.clone())?;

        let msg = to_binary(&InvokeMsg::SwapTokens { expected_return: None, to: None, msg:None,
            router_link: ContractLink{code_hash: "Test".to_string(), address: HumanAddr("Test".to_string())} })?;

        messages.push(WasmMsg::Execute {
            contract_addr:      amm_address.clone(),
            callback_code_hash: factory_config.pair_contract.code_hash.clone(),
            msg,
            send: vec![],
        }.into());
    }

    Ok(HandleResponse {
        messages: messages,
        log: vec![],
        data: None,
    })
}

fn query_factory_config(
    querier: &impl Querier,
    factory_address: ContractLink<HumanAddr>
) -> StdResult<FactoryConfig>
{
    let result: FactoryQueryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr:      factory_address.address.clone(),
        callback_code_hash: factory_address.code_hash.clone(),
        msg: to_binary(& FactoryQueryMsg::GetConfig {})?
    }))?;

    match result {
        FactoryQueryResponse::GetConfig { pair_contract, amm_settings, lp_token_contract } => {
            Ok(FactoryConfig{ pair_contract, amm_settings})
        },
        _ =>  Err(StdError::generic_err(
            "An error occurred while trying to retrieve factory settings.",
        )),
    }
}

fn query_token_addr(
    querier: &impl Querier,
    token1: &TokenType<HumanAddr>,
    token2: &TokenType<HumanAddr>,
    factory_address: ContractLink<HumanAddr>
) -> StdResult<HumanAddr>
{
    let result: FactoryQueryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr:      factory_address.address.clone(),
        callback_code_hash: factory_address.code_hash.clone(),
        msg: to_binary(& FactoryQueryMsg::GetAMMPairAddress { pair: (
            TokenPair(token1.clone(), token2.clone())
        ) })?
    }))?;

    match result {
        FactoryQueryResponse::GetAMMPairAddress { address } => {
            Ok(address)
        },
        _ =>  Err(StdError::generic_err(
            "An error occurred while trying to retrieve factory settings.",
        )),
    }
}

pub fn swap_tokens_for_exact_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amountOut: Uint128,
    amountInMax: Uint128,
    path: &[ContractInfo],
    to: ContractInfo,
) -> HandleResult {
    
    //Validates whether the amount required to be paid is greater then the amount in max

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

struct FactoryConfig {
    pair_contract: ContractInstantiationInfo,
    amm_settings: AMMSettings<HumanAddr>
}