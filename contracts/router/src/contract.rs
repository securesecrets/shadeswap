use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, CosmosMsg, BankMsg, Coin,
};
use shadeswap_shared::Contract;
use shadeswap_shared::core::apply_admin_guard;
use shadeswap_shared::router::QueryMsgResponse;
use shadeswap_shared::snip20::helpers::send_msg;
use shadeswap_shared::utils::{pad_query_result, pad_response_result};
use shadeswap_shared::{core::admin_w, router::InitMsg};
use shadeswap_shared::{
    core::{ContractLink, TokenAmount, TokenType},
    router::{ExecuteMsg, InvokeMsg, QueryMsg},
};


use crate::{
    operations::{
        next_swap, query_pair_contract_config, refresh_tokens, swap_simulation,
        swap_tokens_for_exact_tokens,
    },
    state::{config_r, config_w, Config},
};

/// Pad handle responses and log attributes to blocks
/// of 256 bytes to prevent leaking info based on response size
const BLOCK_SIZE: usize = 256;
const SHADE_ROUTER_KEY: &str = "SHADE_ROUTER_KEY";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    config_w(deps.storage).save(&Config {
        viewing_key: SHADE_ROUTER_KEY.to_string(),
        pair_contract_code_hash: msg.pair_contract_code_hash,
    })?;

    admin_w(deps.storage).save(&info.sender)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_response_result(
        match msg {
            ExecuteMsg::Receive {
                from, amount, msg, ..
            } => receiver_callback(deps, env, info, from, amount, msg),
            ExecuteMsg::SwapTokensForExact {
                offer,
                expected_return,
                path,
                recipient,
            } => {
                if !offer.token.is_native_token() {
                    return Err(StdError::generic_err(
                        "Sent a non-native token. Should use the receive interface in SNIP20.",
                    ));
                }
                offer.assert_sent_native_token_balance(&info)?;
                let sender = info.sender.clone();
                swap_tokens_for_exact_tokens(
                    deps,
                    env,
                    info,
                    offer,
                    expected_return,
                    &path,
                    sender,
                    recipient,
                )
            }
            ExecuteMsg::SwapCallBack {
                last_token_out,
                signature,
            } => next_swap(deps, env, last_token_out, signature),
            ExecuteMsg::RegisterSNIP20Token {
                token_addr,
                token_code_hash,
            } => refresh_tokens(deps, env, token_addr, token_code_hash),
            ExecuteMsg::RecoverFunds {
                token,
                amount,
                to,
                msg,
            } => {
                apply_admin_guard(&info.sender, deps.storage)?;
                let send_msg = match token {
                    TokenType::CustomToken { contract_addr, token_code_hash } => vec![send_msg(
                        to,
                        amount,
                        msg,
                        None,
                        None,
                        &Contract{
                            address: contract_addr,
                            code_hash: token_code_hash
                        }
                    )?],
                    TokenType::NativeToken { denom } => vec![CosmosMsg::Bank(BankMsg::Send {
                        to_address: to.to_string(),
                        amount: vec![Coin::new(amount.u128(), denom)],
                    })],
                };

                Ok(Response::new().add_messages(send_msg))
            }
        },
        BLOCK_SIZE,
    )
}

fn receiver_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    pad_response_result(
        if let Some(content) = msg {
            match from_binary(&content)? {
                InvokeMsg::SwapTokensForExact {
                    expected_return,
                    paths,
                    recipient,
                } => {
                    let config = config_r(deps.storage).load()?;
                    let pair_config = query_pair_contract_config(
                        &deps.querier,
                        ContractLink {
                            address: paths[0].clone(),
                            code_hash: config.pair_contract_code_hash,
                        },
                    )?;
                    for token in pair_config.pair.into_iter() {
                        match token {
                            TokenType::CustomToken { contract_addr, .. } => {
                                if *contract_addr == info.sender {
                                    let offer = TokenAmount {
                                        token: token.clone(),
                                        amount,
                                    };

                                    return swap_tokens_for_exact_tokens(
                                        deps,
                                        env,
                                        info,
                                        offer,
                                        expected_return,
                                        &paths,
                                        from,
                                        recipient,
                                    );
                                }
                            }
                            _ => continue,
                        }
                    }
                    return Err(StdError::generic_err(
                        "No matching token in pair".to_string(),
                    ));
                }
            }
        } else {
            Ok(Response::default())
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::SwapSimulation { offer, path } => swap_simulation(deps, path, offer),
            QueryMsg::GetConfig {} => return Ok(to_binary(&QueryMsgResponse::GetConfig{ 
                pair_contract_code_hash: config_r(deps.storage).load()?.pair_contract_code_hash, 
            })?),
        },
        BLOCK_SIZE,
    )
}
