use cosmwasm_std::HandleResult;
use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdError, StdResult, Storage,
};

use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config_write, config_read, State};
use cosmwasm_std::{ContractInfo, Uint128};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {};

    config_write(&mut deps.storage).save(&state)?;

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

pub fn swap_exact_tokens_for_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amountIn: Uint128,
    amountOutMin: Uint128,
    path: &[ContractInfo],
    to: ContractInfo,
) -> HandleResult {

    //Validates whether the amount received is greater then the amountOutMin

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
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
