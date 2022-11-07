use std::str::FromStr;

use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QuerierWrapper,
    QueryRequest, Response, StdError, StdResult, Storage, SubMsg, Uint128, Uint256, WasmMsg,
    WasmQuery,
};
use shadeswap_shared::{
    amm_pair::AMMSettings,
    core::{ContractInstantiationInfo, TokenAmount, TokenPair, TokenType},
    msg::amm_pair::{
        ExecuteMsg as AMMPairExecuteMsg, InvokeMsg as AMMPairInvokeMsg,
        QueryMsg as AMMPairQueryMsg, QueryMsgResponse as AMMPairQueryReponse, SwapResult,
    },
    router::{Hop, QueryMsgResponse},
    snip20::{
        self,
        helpers::{register_receive, set_viewing_key_msg},
    },
    Contract,
};

use crate::{
    contract::{SHADE_ROUTER_KEY, SWAP_REPLY_ID},
    state::{config_r, config_w, epheral_storage_r, epheral_storage_w, CurrentSwapInfo},
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

pub fn next_swap(deps: DepsMut, env: Env, mut response: Response) -> StdResult<Response> {
    let current_trade_info: Option<CurrentSwapInfo> = epheral_storage_r(deps.storage).may_load()?;
    if let Some(mut info) = current_trade_info {       
        let token_in: TokenAmount = TokenAmount {
            token: info.next_token_in.clone(),
            amount: info.next_token_in.query_balance(
                deps.as_ref(),
                env.contract.address.to_string(),
                SHADE_ROUTER_KEY.to_owned(),
            )?,
        };
        println!("FSDFSd");
        if info.path.len() > (info.current_index + 1) as usize {        
            let next_pair_contract = query_pair_contract_config(
                &deps.querier,
                Contract {
                    address: info.path[info.current_index as usize + 1].addr.clone(),
                    code_hash: info.path[info.current_index as usize + 1].code_hash.clone(),
                },
            )?;

            info.current_index = info.current_index + 1;

            if next_pair_contract.pair.0 == info.next_token_in {
                info.next_token_in = next_pair_contract.pair.1;
            } else {
                info.next_token_in = next_pair_contract.pair.0;
            }
            epheral_storage_w(deps.storage).save(&info)?;
           
            response = get_trade_with_callback(
                deps,
                env,
                token_in,
                info.path[(info.current_index) as usize].clone(),
                response,
            )?;
            Ok(response)
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

            epheral_storage_w(deps.storage).remove();
            response = response.add_messages(vec![token_in.token.create_send_msg(
                env.contract.address.to_string(),
                info.recipient.to_string(),
                token_in.amount,
            )?]);

            Ok(response)
        }
    } else {
        Err(StdError::generic_err(
            "There is currently no trade in progress.",
        ))
    }
}

pub fn swap_tokens_for_exact_tokens(
    deps: DepsMut,
    env: Env,
    amount_in: TokenAmount,
    amount_out_min: Option<Uint128>,
    path: &Vec<Hop>,
    sender: Addr,
    recipient: Option<Addr>,
    mut response: Response,
) -> StdResult<Response> {
    //Validates whether the amount received is greater then the amount_out_min

    let next_pair_contract = query_pair_contract_config(
        &deps.querier,
        Contract {
            address: path[0].addr.clone(),
            code_hash: path[0].code_hash.clone(),
        },
    )?;

    let next_token_in;
    if next_pair_contract.pair.0 == amount_in.token {
        next_token_in = next_pair_contract.pair.1;
    } else {
        next_token_in = next_pair_contract.pair.0;
    }

    epheral_storage_w(deps.storage).save(&CurrentSwapInfo {
        amount: amount_in.clone(),
        amount_out_min: amount_out_min,
        path: path.clone(),
        recipient: recipient.unwrap_or(sender),
        current_index: 0,
        next_token_in: next_token_in,
    })?;

    response = get_trade_with_callback(
        deps,
        env,
        amount_in,
        path[0].clone(),
        response,
    )?;
 
    Ok(response)
}

fn get_trade_with_callback(
    _deps: DepsMut,
    env: Env,
    token_in: TokenAmount,
    hop: Hop,
    mut response: Response,
) -> StdResult<Response> {

    match &token_in.token {
        TokenType::NativeToken { denom } => {
            let msg = to_binary(&AMMPairExecuteMsg::SwapTokens {
                expected_return: None,
                to: None,
                offer: token_in.clone()
            })?;

            response = response.add_submessage(SubMsg::reply_always(
                WasmMsg::Execute {
                    contract_addr: hop.addr.to_string(),
                    code_hash: hop.code_hash,
                    msg,
                    funds: vec![Coin {
                        denom: denom.clone(),
                        amount: token_in.amount,
                    }],
                },
                SWAP_REPLY_ID,
            ));
        }
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => {
            let msg = to_binary(&snip20::ExecuteMsg::Send {
                recipient: hop.addr.to_string(),
                amount: token_in.amount,
                msg: Some(to_binary(&AMMPairInvokeMsg::SwapTokens {
                    expected_return: None,
                    to: Some(env.contract.address.clone()),
                })?),
                padding: None,
                recipient_code_hash: None,
                memo: None,
            })?;

            response = response.add_submessage(SubMsg::reply_always(
                WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    code_hash: token_code_hash.clone(),
                    msg,
                    funds: vec![],
                },
                SWAP_REPLY_ID,
            ));
        }
    };
    return Ok(response);
}

pub fn update_viewing_key(storage: &mut dyn Storage, viewing_key: String) -> StdResult<Response> {
    let mut config = config_w(storage).load()?;
    config.viewing_key = viewing_key;
    config_w(storage).save(&config)?;
    Ok(Response::default())
}

pub fn query_pair_contract_config(
    querier: &QuerierWrapper,
    pair_contract_address: Contract,
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

pub fn swap_simulation(deps: Deps, path: Vec<Hop>, offer: TokenAmount) -> StdResult<Binary> {
    let mut sum_total_fee_amount: Uint128 = Uint128::zero();
    let mut sum_lp_fee_amount: Uint128 = Uint128::zero();
    let mut sum_shade_dao_fee_amount: Uint128 = Uint128::zero();
    let mut next_in = offer.clone();
    let querier = &deps.querier;
    let config = config_r(deps.storage).load()?;

    for hop in path {
        let contract = Contract {
            address: hop.addr,
            code_hash: hop.code_hash,
        };
        let contract_info: AMMPairQueryReponse =
            querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract.address.to_string(),
                code_hash: contract.code_hash.clone(),
                msg: to_binary(&AMMPairQueryMsg::GetPairInfo {})?,
            }))?;

        match contract_info {
            AMMPairQueryReponse::GetPairInfo {
                liquidity_token: _,
                factory: _,
                pair,
                amount_0: _,
                amount_1: _,
                total_liquidity: _,
                contract_version: _,
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
                        price: _,
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
                        sum_total_fee_amount =
                            total_fee_amount.checked_add(sum_total_fee_amount)?;
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
    pub admin_auth: Contract,
}

pub struct PairConfig {
    pub liquidity_token: Contract,
    pub factory: Contract,
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
