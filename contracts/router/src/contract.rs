use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Response, StdError, StdResult, Uint128, SubMsgResult, Reply,
};
use shadeswap_shared::Contract;
use shadeswap_shared::admin::helpers::{validate_admin, AdminPermissions};
use shadeswap_shared::router::{QueryMsgResponse, InitMsg};
use shadeswap_shared::snip20::helpers::send_msg;
use shadeswap_shared::utils::{pad_query_result, pad_response_result};
use shadeswap_shared::{
    core::{TokenAmount, TokenType},
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
pub const SHADE_ROUTER_KEY: &str = "SHADE_ROUTER_KEY";
pub const SWAP_REPLY_ID: u64 = 1u64;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    config_w(deps.storage).save(&Config {
        viewing_key: SHADE_ROUTER_KEY.to_string(),
        admin_auth: msg.admin_auth,
    })?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_response_result(
        match msg {
            ExecuteMsg::Receive {
                from, amount, msg, ..
            } => {
                let checked_address = deps.api.addr_validate(&from)?;
                receiver_callback(deps, env, info, checked_address, amount, msg)
            }
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
                let checked_address = match recipient {
                    Some(x) => Some(deps.api.addr_validate(&x)?),
                    None => None,
                };
                let response = Response::new();
                Ok(swap_tokens_for_exact_tokens(
                    deps,
                    env,
                    offer,
                    expected_return,
                    &path,
                    sender,
                    checked_address,
                    response
                )?)
            }
            ExecuteMsg::RegisterSNIP20Token {
                token_addr,
                token_code_hash,
            } => {
                let checked_token_addr = deps.api.addr_validate(&token_addr)?;
                refresh_tokens(deps, env, checked_token_addr, token_code_hash)
            }
            ExecuteMsg::RecoverFunds {
                token,
                amount,
                to,
                msg,
            } => {
                let config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                let send_msg = match token {
                    TokenType::CustomToken {
                        contract_addr,
                        token_code_hash,
                    } => vec![send_msg(
                        deps.api.addr_validate(&to)?,
                        amount,
                        msg,
                        None,
                        None,
                        &Contract {
                            address: contract_addr,
                            code_hash: token_code_hash,
                        },
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
                    path,
                    recipient,
                } => {
                    let pair_config = query_pair_contract_config(
                        &deps.querier,
                        Contract {
                            address: deps.api.addr_validate(&path[0].addr.to_string())?,
                            code_hash: path[0].code_hash.clone(),
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

                                    let checked_address = match recipient {
                                        Some(x) => Some(deps.api.addr_validate(&x)?),
                                        None => None,
                                    };

                                    let response = Response::new();
                                    return Ok(swap_tokens_for_exact_tokens(
                                        deps,
                                        env,
                                        offer,
                                        expected_return,
                                        &path,
                                        from,
                                        checked_address,
                                        response,
                                    )?);
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
            })?),
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    pad_response_result(match (msg.id) {
        (SWAP_REPLY_ID) => {
            let response = Response::new();
            Ok(next_swap(deps, env, response)?)
        },
        _ => Ok(Response::default())
    }, BLOCK_SIZE)
}
