use std::{borrow::BorrowMut, str::from_utf8};

use shadeswap_shared::{
    amm_pair::AMMSettings,
    fadroma::{
        self,
        admin::{
            assert_admin, handle as admin_handle, load_admin, query as admin_query, save_admin,
            DefaultImpl as AdminImpl,
        },
        debug_print, from_binary,
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
        QueryRequest, Uint128, ViewingKey, WasmQuery, secret_toolkit::snip20::BalanceResponse, BankMsg, Coin,
    },
    msg::{
        amm_pair::{
            HandleMsg as AMMPairHandleMsg, InvokeMsg as AMMPairInvokeMsg,
            QueryMsg as AMMPairQueryMsg, QueryMsgResponse as AMMPairQueryReponse,
        },
        router::{HandleMsg, InvokeMsg, QueryMsg},
    },
    msg::{
        factory::{QueryMsg as FactoryQueryMsg, QueryResponse as FactoryQueryResponse},
        router::InitMsg,
    },
    TokenAmount, TokenPair, TokenType,
};

use crate::state::{config_read, config_write, Config};
use crate::state::{read_token, write_new_token, CurrentSwapInfo};

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
        HandleMsg::Receive {
            from, amount, msg, ..
        } => receiver_callback(deps, env, from, amount, msg),
        HandleMsg::SwapTokensForExact {
            offer,
            expected_return,
            path,
            recipient,
        } => {
            if !offer.token.is_native_token() {
                return Err(StdError::unauthorized());
            }
            offer.assert_sent_native_token_balance(&env)?;
            let config = config_read(deps)?;
            let sender = env.message.sender.clone();
            swap_exact_tokens_for_tokens(
                deps,
                env,
                offer,
                expected_return,
                &path,
                sender,
                recipient,
            )
        }
        HandleMsg::SwapCallBack {
            last_token_in,
            signature,
        } => next_swap(deps, env, last_token_in, signature),
    }
}

fn receiver_callback<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    let msg = msg.ok_or_else(|| {
        StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
    })?;

    match from_binary(&msg)? {
        InvokeMsg::SwapTokensForExact {
            offer,
            expected_return,
            paths,
            recipient,
        } => {
            if let TokenType::CustomToken { contract_addr, .. } = offer.token.clone() {
                if contract_addr == env.message.sender {
                    let offer = TokenAmount {
                        token: offer.token.clone(),
                        amount,
                    };

                    return swap_exact_tokens_for_tokens(
                        deps,
                        env,
                        offer,
                        expected_return,
                        &paths,
                        from,
                        recipient,
                    );
                }
            }
            Err(StdError::unauthorized())
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

pub fn next_swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    last_token_out: TokenAmount<HumanAddr>,
    signature: Binary,
) -> HandleResult {
    let currentTradeInfo: Option<CurrentSwapInfo> = load(&deps.storage, EPHEMERAL_STORAGE_KEY)?;
    let config = config_read(deps)?;
    let factory_config = query_factory_config(&deps.querier, config.factory_address.clone())?;
    
    match currentTradeInfo {
        Some(info) => {
            if (signature != info.signature) {
                return Err(StdError::unauthorized());
            }
            let pair_contract = query_pair_contract_config(
                &deps.querier,
                ContractLink {
                    address: info.paths[info.current_index].clone(),
                    code_hash: factory_config.pair_contract.code_hash.clone(),
                },
            )?;

            let mut next_token_in = pair_contract.pair.0.clone();

            if (pair_contract.pair.0.clone() == last_token_out.token) {
                next_token_in = pair_contract.pair.1;
            }

            let mut tokenIn: TokenAmount<HumanAddr> = TokenAmount {
                token: next_token_in.clone(),
                amount: last_token_out.amount,
            };

            if(info.paths.len() > info.current_index + 1)
            {
                save(
                    &mut deps.storage,
                    EPHEMERAL_STORAGE_KEY,
                    &CurrentSwapInfo {
                        amount: info.amount.clone(),
                        paths: info.paths.clone(),
                        signature: info.signature.clone(),
                        recipient: info.recipient,
                        current_index: info.current_index + 1,
                    }
                )?;
                Ok(HandleResponse {
                    messages: get_trade_with_callback(
                        deps,
                        env,
                        tokenIn,
                        info.paths[info.current_index + 1].clone(),
                        factory_config.pair_contract.code_hash.clone(),
                        info.current_index + 1,
                        info.signature,
                    )?,
                    log: vec![],
                    data: None,
                })
            }
            else
            {
                Ok(HandleResponse {
                    messages: vec![tokenIn.token.create_send_msg(env.contract.address, info.recipient, tokenIn.amount)?],
                    log: vec![],
                    data: None,
                })
            }
        }
        None => Err(StdError::generic_err("")),
    }
}


pub fn swap_exact_tokens_for_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amountIn: TokenAmount<HumanAddr>,
    amountOutMin: Option<Uint128>,
    paths: &Vec<HumanAddr>,
    sender: HumanAddr,
    recipient: Option<HumanAddr>,
) -> HandleResult {
    let querier = &deps.querier;
    //Validates whether the amount received is greater then the amountOutMin
    let config = config_read(deps)?;
    let factory_config = query_factory_config(querier, config.factory_address.clone())?;
    let contract_address = HumanAddr::from(env.contract.address.clone());
    let signature = create_signature(&env)?;
    save(
        &mut deps.storage,
        EPHEMERAL_STORAGE_KEY,
        &CurrentSwapInfo {
            amount: amountIn.clone(),
            paths: paths.clone(),
            signature: signature.clone(),
            recipient: recipient.unwrap_or(sender),
            current_index: 0,
        },
    )?;

    Ok(HandleResponse {
        messages: get_trade_with_callback(
            deps,
            env,
            amountIn,
            paths[0].clone(),
            factory_config.pair_contract.code_hash,
            0,
            signature.clone(),
        )?,
        log: vec![],
        data: None,
    })
}

fn get_or_create_view_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    contract_addr: &HumanAddr,
) -> StdResult<String> {
    let config = config_read(deps)?;
    let search_view_key = read_token(&deps.storage, &contract_addr.canonize(&deps.api)?);
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
    return Ok(view_key.clone());
}

fn get_trade_with_callback<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    tokenIn: TokenAmount<HumanAddr>,
    path: HumanAddr,
    code_hash: String,
    current_index: usize,
    signature: Binary,
) -> StdResult<Vec<CosmosMsg>> {
    let mut messages: Vec<CosmosMsg> = vec![];
    let config = config_read(deps)?;
    let querier = &deps.querier;

    match &tokenIn.token {
        TokenType::NativeToken { .. } => {
            let msg = to_binary(&AMMPairHandleMsg::SwapTokens {
                expected_return: None,
                to: None,
                router_link: Some(ContractLink {
                    address: env.contract.address.clone(),
                    code_hash: env.contract_code_hash.clone(),
                }),
                offer: tokenIn,
                callback_signature: Some(signature)
            })?;

            messages.push(
                WasmMsg::Execute {
                    contract_addr: path.clone(),
                    callback_code_hash: code_hash,
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
            let msg = to_binary(&snip20::HandleMsg::Send {
                recipient: path.clone(),
                amount: tokenIn.amount,
                msg: Some(
                    to_binary(&AMMPairInvokeMsg::SwapTokens {
                        expected_return: None,
                        to: None,
                        router_link: Some(ContractLink {
                            address: env.contract.address.clone(),
                            code_hash: env.contract_code_hash.clone(),
                        }),
                        callback_signature: Some(signature),
                    })
                    .unwrap(),
                ),
                padding: None,
            })?;

            messages.push(
                WasmMsg::Execute {
                    contract_addr: path.clone(),
                    callback_code_hash: code_hash.clone(),
                    msg,
                    send: vec![],
                }
                .into(),
            );
        }
    };
    return Ok(messages);
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

fn query_pair_contract_config(
    querier: &impl Querier,
    pair_contract_address: ContractLink<HumanAddr>,
) -> StdResult<PairConfig> {
    let result: AMMPairQueryReponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract_address.address.clone(),
        callback_code_hash: pair_contract_address.code_hash.clone(),
        msg: to_binary(&AMMPairQueryMsg::GetPairInfo {})?,
    }))?;

    match result {
        AMMPairQueryReponse::GetPairInfo {
            liquidity_token,
            factory,
            pair,
            amount_0,
            amount_1,
            total_liquidity,
            contract_version,
        } => Ok(PairConfig {
            liquidity_token: liquidity_token,
            factory: factory,
            pair: pair,
            amount_0: amount_0,
            amount_1: amount_1,
            total_liquidity: total_liquidity,
            contract_version: contract_version,
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

struct PairConfig {
    liquidity_token: ContractLink<HumanAddr>,
    factory: ContractLink<HumanAddr>,
    pair: TokenPair<HumanAddr>,
    amount_0: Uint128,
    amount_1: Uint128,
    total_liquidity: Uint128,
    contract_version: u32,
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

pub(crate) fn create_signature(env: &Env) -> StdResult<Binary> {
    to_binary(
        &[
            env.message.sender.0.as_bytes(),
            &env.block.height.to_be_bytes(),
            &env.block.time.to_be_bytes(),
        ]
        .concat(),
    )
}
