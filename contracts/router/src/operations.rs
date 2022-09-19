use std::str::FromStr;

use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, DepsMut, Env, Response, StdError, StdResult, Uint128, WasmMsg, Coin, Deps, QuerierWrapper, QueryRequest, WasmQuery, Uint256, MessageInfo,
};
use shadeswap_shared::{
    core::{ContractLink, TokenAmount, TokenType, ContractInstantiationInfo, TokenPair, ViewingKey},
    snip20::{helpers::{register_receive, set_viewing_key_msg}, self},
    Contract,
    msg::{
        amm_pair::{ExecuteMsg as AMMPairExecuteMsg, InvokeMsg as AMMPairInvokeMsg, QueryMsgResponse as AMMPairQueryReponse, QueryMsg as AMMPairQueryMsg, SwapResult},
        factory:: {QueryResponse as FactoryQueryResponse, QueryMsg as FactoryQueryMsg}
    }, router::QueryMsgResponse, amm_pair::AMMSettings
};

use crate::{
    state::{config_r, epheral_storage_r, epheral_storage_w, CurrentSwapInfo},
};

pub fn refresh_tokens(
    deps: DepsMut,
    env: Env,
    token_address: Addr,
    token_code_hash: String,
) -> StdResult<Response> {
    let mut msg = vec![];
    let config = config_r(deps.storage).load()?;
    register_pair_token(
        &env,
        &mut msg,
        &TokenType::CustomToken {
            contract_addr: token_address,
            token_code_hash: token_code_hash,
        },
        config.viewing_key,
    )?;

    Ok(Response::new().add_messages(msg))
}

pub fn next_swap(
    deps: DepsMut,
    env: Env,
    last_token_out: TokenAmount,
    signature: Binary,
) -> StdResult<Response> {
    let current_trade_info: Option<CurrentSwapInfo> =
        epheral_storage_r(deps.storage).may_load()?;
    let config = config_r(deps.storage).load()?;
    match current_trade_info {
        Some(info) => {
            if signature != info.signature {
                return Err(StdError::generic_err("".to_string()));
            }
            let pair_contract = query_pair_contract_config(
                &deps.querier,
                ContractLink {
                    address: info.paths[info.current_index as usize].clone(),
                    code_hash: config.pair_contract_code_hash.clone(),
                },
            )?;

            let mut next_token_in = pair_contract.pair.0.clone();

            if pair_contract.pair.1.clone() == last_token_out.token {
                next_token_in = pair_contract.pair.1;
            }

            let token_in: TokenAmount = TokenAmount {
                token: next_token_in.clone(),
                amount: last_token_out.amount,
            };

            if info.paths.len() > (info.current_index + 1) as usize {
                epheral_storage_w(deps.storage).save(&CurrentSwapInfo {
                    amount: info.amount.clone(),
                    paths: info.paths.clone(),
                    signature: info.signature.clone(),
                    recipient: info.recipient,
                    current_index: info.current_index + 1,
                    amount_out_min: info.amount_out_min,
                })?;
                Ok(Response::new().add_messages(get_trade_with_callback(
                    deps,
                    env,
                    token_in,
                    info.paths[(info.current_index + 1) as usize].clone(),
                    config.pair_contract_code_hash.clone(),
                    info.signature,
                )?))
            } else {
                if let Some(min_out) = info.amount_out_min {
                    if token_in.amount.lt(&min_out) {
                        return Err(StdError::generic_err(
                            "Operation fell short of expected_return. Actual: ".to_owned()
                                + &token_in.amount.to_string().to_owned()
                                + ", Expected: "
                                + &min_out.to_string().to_owned(),
                        ));
                    }
                }

                let clear_storage: Option<CurrentSwapInfo> = None;

                epheral_storage_w(deps.storage).remove();
                Ok(
                    Response::new().add_messages(vec![token_in.token.create_send_msg(
                        env.contract.address.to_string(),
                        info.recipient.to_string(),
                        token_in.amount,
                    )?]),
                )
            }
        }
        None => Err(StdError::generic_err(
            "There is currently no trade in progress.",
        )),
    }
}

pub fn swap_tokens_for_exact_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount_in: TokenAmount,
    amount_out_min: Option<Uint128>,
    paths: &Vec<Addr>,
    sender: Addr,
    recipient: Option<Addr>,
) -> StdResult<Response> {
    //Validates whether the amount received is greater then the amount_out_min
    let config = config_r(deps.storage).load()?;
    let signature = create_signature(&env, info)?;
    epheral_storage_w(deps.storage).save(&CurrentSwapInfo {
        amount: amount_in.clone(),
        amount_out_min: amount_out_min,
        paths: paths.clone(),
        signature: signature.clone(),
        recipient: recipient.unwrap_or(sender),
        current_index: 0,
    })?;

    Ok(Response::new().add_messages(get_trade_with_callback(
        deps,
        env,
        amount_in,
        paths[0].clone(),
        config.pair_contract_code_hash,
        signature.clone(),
    )?))
}

fn get_trade_with_callback(
    deps: DepsMut,
    env: Env,
    token_in: TokenAmount,
    path: Addr,
    code_hash: String,
    signature: Binary,
) -> StdResult<Vec<CosmosMsg>> {
    let mut messages: Vec<CosmosMsg> = vec![];

    match &token_in.token {
        TokenType::NativeToken { denom } => {
            let msg = to_binary(&AMMPairExecuteMsg::SwapTokens {
                expected_return: None,
                to: None,
                router_link: Some(ContractLink {
                    address: env.contract.address.clone(),
                    code_hash: env.contract.code_hash.clone(),
                }),
                offer: token_in.clone(),
                callback_signature: Some(signature),
            })?;

            messages.push(
                WasmMsg::Execute {
                    contract_addr: path.to_string(),
                    code_hash: code_hash,
                    msg,
                    funds: vec![Coin {
                        denom: denom.clone(),
                        amount: token_in.amount,
                    }],
                }
                .into(),
            );
        }
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => {
            let msg = to_binary(&snip20::ExecuteMsg::Send {
                recipient: path.to_string(),
                amount: token_in.amount,
                msg: Some(
                    to_binary(&AMMPairInvokeMsg::SwapTokens {
                        expected_return: None,
                        to: Some(env.contract.address.to_string()),
                        router_link: Some(ContractLink {
                            address: env.contract.address.clone(),
                            code_hash: env.contract.code_hash.clone(),
                        }),
                        callback_signature: Some(signature),
                    })
                    .unwrap(),
                ),
                padding: None,
                recipient_code_hash: None,
                memo: None,
            })?;

            messages.push(
                WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    code_hash: token_code_hash.clone(),
                    msg,
                    funds: vec![],
                }
                .into(),
            );
        }
    };
    return Ok(messages);
}

pub fn query_pair_contract_config(
    querier: &QuerierWrapper,
    pair_contract_address: ContractLink,
) -> StdResult<PairConfig> {
    let result: AMMPairQueryReponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract_address.address.to_string(),
        code_hash: pair_contract_address.code_hash.clone(),
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
            "An error occurred while trying to retrieve pair contract settings.",
        )),
    }
}

pub fn swap_simulation(deps: Deps, path: Vec<Addr>, offer: TokenAmount) -> StdResult<Binary> {
    let mut sum_total_fee_amount: Uint128 = Uint128::zero();
    let mut sum_lp_fee_amount: Uint128 = Uint128::zero();
    let mut sum_shade_dao_fee_amount: Uint128 = Uint128::zero();
    let mut next_in = offer.clone();
    let querier = &deps.querier;
    let config = config_r(deps.storage).load()?;

    for hop in path {
        let contract = ContractLink {
            address: hop,
            code_hash: config.pair_contract_code_hash.clone(),
        };
        let contract_info: AMMPairQueryReponse =
            querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract.address.to_string(),
                code_hash: contract.code_hash.clone(),
                msg: to_binary(&AMMPairQueryMsg::GetPairInfo {})?,
            }))?;

        match contract_info {
            AMMPairQueryReponse::GetPairInfo {
                liquidity_token,
                factory,
                pair,
                amount_0,
                amount_1,
                total_liquidity,
                contract_version,
            } => {
                let result: AMMPairQueryReponse =
                    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                        contract_addr: contract.address.to_string(),
                        code_hash: contract.code_hash.clone(),
                        msg: to_binary(&AMMPairQueryMsg::SwapSimulation {
                            offer: next_in.clone(),
                        })?,
                    }))?;
                match result {
                    AMMPairQueryReponse::SwapSimulation {
                        total_fee_amount,
                        lp_fee_amount,
                        shade_dao_fee_amount,
                        result,
                        price,
                    } => {
                        if pair.1 == next_in.token {
                            next_in = TokenAmount {
                                token: pair.0,
                                amount: result.return_amount,
                            };
                        } else {
                            next_in = TokenAmount {
                                token: pair.1,
                                amount: result.return_amount,
                            };
                        }
                        sum_total_fee_amount = total_fee_amount.checked_add(sum_total_fee_amount)?;
                        sum_lp_fee_amount = lp_fee_amount.checked_add(sum_lp_fee_amount)?;
                        sum_shade_dao_fee_amount =
                            shade_dao_fee_amount.checked_add(sum_shade_dao_fee_amount)?;
                    }
                    _ => panic!("Failed to complete hop."),
                };
            }
            _ => panic!("Failed to complete hop."),
        }
    }

    to_binary(&QueryMsgResponse::SwapSimulation {
        total_fee_amount: sum_total_fee_amount,
        lp_fee_amount: sum_lp_fee_amount,
        shade_dao_fee_amount: sum_shade_dao_fee_amount,
        result: SwapResult {
            return_amount: next_in.amount,
        },
        price: (Uint256::from_str(&next_in.amount.to_string())?
            / Uint256::from_str(&offer.amount.to_string())?)
        .to_string(),
    })
}

pub struct FactoryConfig {
    pub pair_contract: ContractInstantiationInfo,
    pub amm_settings: AMMSettings,
}

pub struct PairConfig {
    pub liquidity_token: ContractLink,
    pub factory: ContractLink,
    pub pair: TokenPair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
}

pub(crate) fn create_signature(env: &Env, info: MessageInfo) -> StdResult<Binary> {
    to_binary(
        &[
            info.sender.as_bytes(),
            &env.block.height.to_be_bytes(),
            &env.block.time.seconds().to_be_bytes(),
        ]
        .concat(),
    )
}

fn register_pair_token(
    env: &Env,
    messages: &mut Vec<CosmosMsg>,
    token: &TokenType,
    viewing_key: String,
) -> StdResult<()> {
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = token
    {
        messages.push(set_viewing_key_msg(
            viewing_key.clone(),
            None,
            &Contract {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
        )?);
        messages.push(register_receive(
            env.contract.code_hash.clone(),
            None,
            &Contract {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
        )?);
    }

    Ok(())
}

pub fn create_viewing_key(env: &Env, info: &MessageInfo, seed: Binary, entroy: Binary) -> String {
    ViewingKey::new(&env, info, seed.as_slice(), entroy.as_slice()).to_string()
}
