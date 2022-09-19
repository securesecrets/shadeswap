use cosmwasm_std::{
    entry_point, from_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use shadeswap_shared::{core::admin_w, router::InitMsg};
use shadeswap_shared::{
    core::{ContractLink, TokenAmount, TokenType},
    router::{ExecuteMsg, InvokeMsg, QueryMsg},
};

use crate::{
    operations::{
        create_viewing_key, next_swap, query_pair_contract_config, refresh_tokens, swap_simulation,
        swap_tokens_for_exact_tokens,
    },
    state::{config_r, config_w, Config},
};

/// Pad handle responses and log attributes to blocks
/// of 256 bytes to prevent leaking info based on response size
const BLOCK_SIZE: usize = 256;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    config_w(deps.storage).save(&Config {
        viewing_key: msg.viewing_key.unwrap_or(create_viewing_key(
            &env,
            &_info,
            msg.prng_seed.clone(),
            msg.entropy.clone(),
        )),
        pair_contract_code_hash: msg.pair_contract_code_hash,
    })?;

    admin_w(deps.storage).save(&_info.sender.clone())?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
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
    }
}

fn receiver_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    match msg {
        Some(content) => match from_binary(&content)? {
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
                Err(StdError::generic_err("".to_string()))
            }
        },
        None => Ok(Response::default()),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SwapSimulation { offer, path } => swap_simulation(deps, path, offer),
    }
}
