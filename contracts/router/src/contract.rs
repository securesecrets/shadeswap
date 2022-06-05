use shadeswap_shared::{
    amm_pair::AMMSettings,
    fadroma::{
        debug_print, from_binary,
        scrt::{
            secret_toolkit::snip20, to_binary, Api, Binary, CosmosMsg, Env, Extern,
            HandleResponse, HumanAddr, InitResponse, Querier, StdError, StdResult, Storage,
            WasmMsg,
        },
        scrt_link::ContractLink,
        scrt_storage::{load, save}, ContractInstantiationInfo, HandleResult,
        QueryRequest, Uint128, ViewingKey, WasmQuery,
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
};
use shadeswap_shared::token_pair::TokenPair;
use shadeswap_shared::token_amount::TokenAmount;
use shadeswap_shared::token_type::TokenType;

use crate::state::{config_read, config_write, Config, CurrentSwapInfo};

/// Pad handle responses and log attributes to blocks
/// of 256 bytes to prevent leaking info based on response size
const BLOCK_SIZE: usize = 256;

pub const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    config_write(deps, &Config{ factory_address: msg.factory_address, viewing_key: create_viewing_key(&env, msg.prng_seed.clone(), msg.entropy.clone()) })?;

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
            let sender = env.message.sender.clone();
            swap_tokens_for_exact_tokens(
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
        HandleMsg::RegisterSNIP20Token { token, token_code_hash } => {
            refresh_tokens(deps, env, token, token_code_hash)
        }
    }
}

fn refresh_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token_address: HumanAddr,
    token_code_hash: String
) -> StdResult<HandleResponse> {
    let mut msg = vec![];
    let config = config_read(deps)?;
    register_pair_token(&env, &mut msg, &TokenType::CustomToken { contract_addr: token_address, token_code_hash: token_code_hash }, &config.viewing_key)?;

    Ok(HandleResponse {
        messages: msg,
        log: vec![],
        data: None,
    })
}

fn receiver_callback<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    
    match msg {
        Some(content) => {
            match from_binary(&content)? {
                InvokeMsg::SwapTokensForExact {
                    expected_return,
                    paths,
                    to,
                } => {
                    let config = config_read(deps)?;
                    let factory_config = query_factory_config(&deps.querier, config.factory_address.clone())?;
                    let pair_config = query_pair_contract_config(&deps.querier, ContractLink{ address: paths[0].clone(), code_hash: factory_config.pair_contract.code_hash })?;
                    for token in pair_config.pair.into_iter() {
                        match token {
                            TokenType::CustomToken { contract_addr, .. } => {
                                if *contract_addr == env.message.sender {
                                    let offer = TokenAmount {
                                        token: token.clone(),
                                        amount,
                                    };
        
                                    return swap_tokens_for_exact_tokens(
                                        deps,
                                        env,
                                        offer,
                                        expected_return,
                                        &paths,
                                        from,
                                        to,
                                    );
                                }
                            }
                            _ => continue,
                        }
                    }
                    Err(StdError::unauthorized())
                }
            }
        },
        None => Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: None,
        }),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
    }
}

pub fn next_swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    last_token_out: TokenAmount<HumanAddr>,
    signature: Binary,
) -> HandleResult {
    /*Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })*/
    let current_trade_info: Option<CurrentSwapInfo> = load(&deps.storage, EPHEMERAL_STORAGE_KEY)?;
    let config = config_read(deps)?;
    let factory_config = query_factory_config(&deps.querier, config.factory_address.clone())?;
    
    match current_trade_info {
        Some(info) => {
            if signature != info.signature {
                return Err(StdError::unauthorized());
            }
            let pair_contract = query_pair_contract_config(
                &deps.querier,
                ContractLink {
                    address: info.paths[info.current_index as usize].clone(),
                    code_hash: factory_config.pair_contract.code_hash.clone(),
                },
            )?;

            let mut next_token_in = pair_contract.pair.0.clone();

            if pair_contract.pair.1.clone() == last_token_out.token {
                next_token_in = pair_contract.pair.1;
            }

            let token_in: TokenAmount<HumanAddr> = TokenAmount {
                token: next_token_in.clone(),
                amount: last_token_out.amount,
            };

            if info.paths.len() > (info.current_index + 1) as usize
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
                        amount_out_min: info.amount_out_min
                    }
                )?;
                Ok(HandleResponse {
                    messages: get_trade_with_callback(
                        deps,
                        env,
                        token_in,
                        info.paths[(info.current_index + 1) as usize].clone(),
                        factory_config.pair_contract.code_hash.clone(),
                        info.signature,
                    )?,
                    log: vec![],
                    data: None,
                })
            }
            else
            {
                if let Some(min_out) = info.amount_out_min {
                    if  token_in.amount.lt(&min_out) {
                        return Err(StdError::generic_err(
                            "Operation fell short of expected_return",
                        ));
                    }
                }
                
                Ok(HandleResponse {
                    messages: vec![token_in.token.create_send_msg(env.contract.address, info.recipient, token_in.amount)?],
                    log: vec![],
                    data: None,
                })
            }
        }
        None => Err(StdError::generic_err("")),
    }
}


pub fn swap_tokens_for_exact_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount_in: TokenAmount<HumanAddr>,
    amount_out_min: Option<Uint128>,
    paths: &Vec<HumanAddr>,
    sender: HumanAddr,
    recipient: Option<HumanAddr>,
) -> HandleResult {
    let querier = &deps.querier;
    //Validates whether the amount received is greater then the amount_out_min
    let config = config_read(deps)?;
    let factory_config = query_factory_config(querier, config.factory_address.clone())?;
    let signature = create_signature(&env)?;
    save(
        &mut deps.storage,
        EPHEMERAL_STORAGE_KEY,
        &CurrentSwapInfo {
            amount: amount_in.clone(),
            amount_out_min: amount_out_min,
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
            amount_in,
            paths[0].clone(),
            factory_config.pair_contract.code_hash,
            signature.clone(),
        )?,
        log: vec![],
        data: None,
    })
}

fn get_trade_with_callback<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token_in: TokenAmount<HumanAddr>,
    path: HumanAddr,
    code_hash: String,
    signature: Binary,
) -> StdResult<Vec<CosmosMsg>> {
    let mut messages: Vec<CosmosMsg> = vec![];

    match &token_in.token {
        TokenType::NativeToken { .. } => {
            let msg = to_binary(&AMMPairHandleMsg::SwapTokens {
                expected_return: None,
                to: None,
                router_link: Some(ContractLink {
                    address: env.contract.address.clone(),
                    code_hash: env.contract_code_hash.clone(),
                }),
                offer: token_in,
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
                amount: token_in.amount,
                msg: Some(
                    to_binary(&AMMPairInvokeMsg::SwapTokens {
                        expected_return: None,
                        to: Some(env.contract.address.clone()),
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
                    contract_addr: contract_addr.clone(),
                    callback_code_hash: token_code_hash.clone(),
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
            lp_token_contract: _,
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

fn register_pair_token(
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


pub fn create_viewing_key(env: &Env, seed: Binary, entroy: Binary) -> ViewingKey {
    //return ViewingKey("password".to_string());
    ViewingKey::new(&env, seed.as_slice(), entroy.as_slice())
}
