use std::{borrow::BorrowMut, str::from_utf8};

use shadeswap_shared::{
    amm_pair::AMMSettings,
    fadroma::{
        self,
        admin::{
            assert_admin, handle as admin_handle, load_admin, query as admin_query, save_admin,
            DefaultImpl as AdminImpl,
        },
        debug_print,
        require_admin::require_admin,
        scrt::{
            log, secret_toolkit::snip20, to_binary, Api, Binary, CosmosMsg, Env, Extern,
            HandleResponse, HumanAddr, InitResponse, Querier, StdError, StdResult, Storage,
            WasmMsg,
        },
        scrt_callback::Callback,
        scrt_link::ContractLink,
        scrt_migrate,
        scrt_migrate::get_status,
        scrt_storage::{load, remove, save},
        with_status, Canonize, ContractInfo, ContractInstantiationInfo, Empty, HandleResult,
        QueryRequest, Uint128, ViewingKey, WasmQuery,
    },
    msg::amm_pair::InvokeMsg,
    msg::factory::{QueryMsg as FactoryQueryMsg, QueryResponse as FactoryQueryResponse},
    TokenAmount, TokenPair, TokenType,
};

use crate::state::{config_read, config_write, Config};
use crate::{
    msg::{CountResponse, HandleMsg, InitMsg, QueryMsg},
    state::{read_token, write_new_token, CurrentSwapInfo},
};

/// Pad handle responses and log attributes to blocks
/// of 256 bytes to prevent leaking info based on response size
const BLOCK_SIZE: usize = 256;

pub const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    config_write(deps, &Config::from_init_msg(env.clone(), msg));

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::SwapTokens {
            offer,
            expected_return,
            path,
        } => {
            if !offer.token.is_native_token() {
                return Err(StdError::unauthorized());
            }
            offer.assert_sent_native_token_balance(&env)?;
            let config = config_read(deps)?;
            let sender = env.message.sender.clone();
            swap_exact_tokens_for_tokens(deps, env, offer, expected_return, &path, Some(sender))
        }
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => todo!(),
    }
}

pub fn swap_exact_tokens_for_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amountIn: TokenAmount<HumanAddr>,
    amountOutMin: Option<Uint128>,
    path: &Vec<TokenType<HumanAddr>>,
    to: Option<HumanAddr>,
) -> HandleResult {
    let mut messages: Vec<CosmosMsg> = vec![];
    let querier = &deps.querier;
    //Validates whether the amount received is greater then the amountOutMin
    let config = config_read(deps)?;
    let factory_config = query_factory_config(querier, config.factory_address.clone())?;
    let contract_address =   HumanAddr::from(env.contract.address.clone());
    save(
        &mut deps.storage,
        EPHEMERAL_STORAGE_KEY,
        &CurrentSwapInfo {
            amount: amountIn.clone(),
        },
    )?;

    for x in 0..(path.len() - 1) {
        let amm_address =
            query_token_addr(querier, &path[0], &path[1], config.factory_address.clone())?;

        match &path[0] {
            TokenType::NativeToken { .. } => {
                let msg = to_binary(&InvokeMsg::SwapTokens {
                    expected_return: None,
                    to: None,
                })?;

                messages.push(
                    WasmMsg::Execute {
                        contract_addr: amm_address.clone(),
                        callback_code_hash: factory_config.pair_contract.code_hash.clone(),
                        msg,
                        send: vec![],
                    }
                    .into(),
                );
            }
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                let search_view_key =
                    read_token(&deps.storage, &contract_addr.canonize(&deps.api)?);
                let mut view_key = "".to_string();
                match search_view_key {
                    Some(key) => view_key = from_utf8(key.as_slice()).unwrap().to_string(),
                    None => {
                        view_key = config.viewing_key.0.to_string();
                        write_new_token(
                            &mut deps.storage,
                            &contract_addr.canonize(&deps.api)?,
                            &config.viewing_key.clone(),
                        )
                    }
                }

                let balance_msg = snip20::QueryMsg::Balance {
                    address:contract_address.clone(),
                    key: String::from(view_key.clone()),
                };

                let msg = to_binary(&snip20::HandleMsg::Send {
                    recipient: amm_address.clone(),
                    amount: querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                        callback_code_hash: token_code_hash.to_string(),
                        contract_addr: HumanAddr::from(contract_addr.to_string()),
                        msg: to_binary(&snip20::QueryMsg::Balance {
                            address: contract_address.clone(),
                            key: String::from(view_key.clone()),
                        })?,
                    }))?,
                    msg: Some(
                        to_binary(&InvokeMsg::SwapTokens {
                            expected_return: None,
                            to: None,
                        })
                        .unwrap(),
                    ),
                    padding: None,
                })?;

                messages.push(
                    WasmMsg::Execute {
                        contract_addr: amm_address.clone(),
                        callback_code_hash: factory_config.pair_contract.code_hash.clone(),
                        msg,
                        send: vec![],
                    }
                    .into(),
                );
            }
        };
    }

    Ok(HandleResponse {
        messages: messages,
        log: vec![],
        data: None,
    })
}

fn query_factory_config(
    querier: &impl Querier,
    factory_address: ContractLink<HumanAddr>,
) -> StdResult<FactoryConfig> {
    let result: FactoryQueryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: factory_address.address.clone(),
        callback_code_hash: factory_address.code_hash.clone(),
        msg: to_binary(&FactoryQueryMsg::GetConfig {})?,
    }))?;

    match result {
        FactoryQueryResponse::GetConfig {
            pair_contract,
            amm_settings,
            lp_token_contract,
        } => Ok(FactoryConfig {
            pair_contract,
            amm_settings,
        }),
        _ => Err(StdError::generic_err(
            "An error occurred while trying to retrieve factory settings.",
        )),
    }
}

fn query_token_addr(
    querier: &impl Querier,
    token1: &TokenType<HumanAddr>,
    token2: &TokenType<HumanAddr>,
    factory_address: ContractLink<HumanAddr>,
) -> StdResult<HumanAddr> {
    let result: FactoryQueryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: factory_address.address.clone(),
        callback_code_hash: factory_address.code_hash.clone(),
        msg: to_binary(&FactoryQueryMsg::GetAMMPairAddress {
            pair: (TokenPair(token1.clone(), token2.clone())),
        })?,
    }))?;

    match result {
        FactoryQueryResponse::GetAMMPairAddress { address } => Ok(address),
        _ => Err(StdError::generic_err(
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
    amm_settings: AMMSettings<HumanAddr>,
}

fn register_custom_token(
    env: &Env,
    messages: &mut Vec<CosmosMsg>,
    token: &TokenType<HumanAddr>,
    viewing_key: &ViewingKey,
) -> StdResult<()> {
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = token
    {
        messages.push(snip20::set_viewing_key_msg(
            viewing_key.0.clone(),
            None,
            BLOCK_SIZE,
            token_code_hash.clone(),
            contract_addr.clone(),
        )?);
        messages.push(snip20::register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            BLOCK_SIZE,
            token_code_hash.clone(),
            contract_addr.clone(),
        )?);
    }

    Ok(())
}
