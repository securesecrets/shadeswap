use crate::{
    operations::{
        add_address_to_whitelist, add_liquidity, get_estimated_lp_token, get_shade_dao_info,
        load_trade_history_query, query_calculate_price, query_factory_authorize_api_key,
        query_token_symbol, query_total_supply, register_lp_token, register_pair_token,
        remove_addresses_from_whitelist, remove_liquidity, set_staking_contract, swap,
        swap_simulation, update_viewing_key,
    },
    state::{config_r, config_w, trade_count_r, whitelist_r, Config},
};

use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg,
};
use shadeswap_shared::{
    admin::helpers::{validate_admin, AdminPermissions},
    core::{create_viewing_key, TokenAmount, TokenType},
    lp_token::{InitConfig, InstantiateMsg},
    msg::amm_pair::{ExecuteMsg, InitMsg, InvokeMsg, QueryMsg, QueryMsgResponse},
    snip20::helpers::send_msg,
    utils::{pad_query_result, pad_response_result}, Contract,
};

const AMM_PAIR_CONTRACT_VERSION: u32 = 1;
pub const INSTANTIATE_LP_TOKEN_REPLY_ID: u64 = 1u64;
pub const INSTANTIATE_STAKING_CONTRACT_REPLY_ID: u64 = 2u64;
pub const BLOCK_SIZE: usize = 256;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    if msg.pair.0 == msg.pair.1 {
        return Err(StdError::generic_err(
            "Creating Pair Contract with the same two tokens.",
        ));
    }

    let mut response = Response::new();
    let mut messages = vec![];
    let viewing_key = create_viewing_key(&env, &_info, msg.prng_seed.clone(), msg.entropy.clone());
    register_pair_token(&env, &mut messages, &msg.pair.0, &viewing_key)?;
    register_pair_token(&env, &mut messages, &msg.pair.1, &viewing_key)?;
    response = response.add_messages(messages);

    let init_snip20_msg = InstantiateMsg {
        name: format!(
            "SHADESWAP Liquidity Provider (LP) token for {}-{}",
            &msg.pair.0, &msg.pair.1
        ),
        admin: Some(env.contract.address.clone()),
        symbol: format!(
            "{}/{} LP",
            query_token_symbol(deps.querier, &msg.pair.0)?,
            query_token_symbol(deps.querier, &msg.pair.1)?
        ),
        decimals: 18,
        initial_balances: None,
        prng_seed: msg.prng_seed.clone(),
        config: Some(InitConfig {
            public_total_supply: Some(true),
            enable_deposit: Some(false),
            enable_redeem: Some(false),
            enable_mint: Some(true),
            enable_burn: Some(true),
        }),
    };

    response = response.add_submessage(SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: msg.lp_token_contract.id,
            msg: to_binary(&init_snip20_msg)?,
            label: format!(
                "{}-{}-ShadeSwap-Pair-Token-{}",
                &msg.pair.0, &msg.pair.1, &env.contract.address
            ),
            code_hash: msg.lp_token_contract.code_hash.clone(),
            funds: vec![],
        }),
        INSTANTIATE_LP_TOKEN_REPLY_ID,
    ));

    match msg.callback {
        Some(c) => {
            response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: c.contract.address.to_string(),
                code_hash: c.contract.code_hash,
                msg: c.msg,
                funds: vec![],
            }))
        }
        None => (),
    }

    let config = Config {
        factory_contract: msg.factory_info.clone(),
        lp_token: Contract {
            code_hash: msg.lp_token_contract.code_hash,
            address: Addr::unchecked(""),
        },
        pair: msg.pair,
        viewing_key: viewing_key,
        custom_fee: msg.custom_fee.clone(),
        staking_contract: None,
        staking_contract_init: msg.staking_contract,
        prng_seed: msg.prng_seed,
        admin_auth: msg.admin_auth,
    };

    config_w(deps.storage).save(&config)?;
    Ok(response.add_attribute("created_exchange_address", env.contract.address.to_string()))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_response_result(
        match msg {
            ExecuteMsg::Receive {
                from, amount, msg, ..
            } => receiver_callback(deps, env, info, from, amount, msg),
            ExecuteMsg::AddLiquidityToAMMContract {
                deposit,
                expected_return,
                staking,
            } => add_liquidity(deps, env, &info, deposit, expected_return, staking),
            ExecuteMsg::SetCustomPairFee { custom_fee } => {
                let mut config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                config.custom_fee = custom_fee;
                config_w(deps.storage).save(&config)?;
                Ok(Response::default())
            }
            ExecuteMsg::AddWhiteListAddress { address } => {
                let config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                add_address_to_whitelist(deps.storage, address)
            }
            ExecuteMsg::RemoveWhitelistAddresses { addresses } => {
                let config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                remove_addresses_from_whitelist(deps.storage, addresses, env)
            }
            ExecuteMsg::SwapTokens {
                offer,
                expected_return,
                to,
            } => {
                if !offer.token.is_native_token() {
                    return Err(StdError::generic_err("Use the receive interface"));
                }
                offer.assert_sent_native_token_balance(&info)?;
                let config_settings = config_r(deps.storage).load()?;
                let sender = info.sender.clone();
                swap(
                    deps,
                    env,
                    config_settings,
                    sender,
                    to,
                    offer,
                    expected_return,
                )
            }
            ExecuteMsg::SetViewingKey { viewing_key } => update_viewing_key(env, deps, viewing_key),
            ExecuteMsg::SetConfig { admin_auth } => {
                let mut config = config_r(deps.storage).load()?;
                validate_admin(
                    &deps.querier,
                    AdminPermissions::ShadeSwapAdmin,
                    &info.sender,
                    &config.admin_auth,
                )?;
                if let Some(admin_auth) = admin_auth {
                    config.admin_auth = admin_auth;
                }
                Ok(Response::default())
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
                        to,
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
    let msg = msg.ok_or_else(|| {
        StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
    })?;

    let config = config_r(deps.storage).load()?;
    let from_caller = from.clone();

    pad_response_result(
        match from_binary(&msg)? {
            InvokeMsg::SwapTokens {
                to,
                expected_return
            } => {
                for token in config.pair.into_iter() {
                    match token {
                        TokenType::CustomToken { contract_addr, .. } => {
                            if *contract_addr == info.sender {
                                let offer = TokenAmount {
                                    token: token.clone(),
                                    amount,
                                };

                                return swap(
                                    deps,
                                    env,
                                    config,
                                    from,
                                    Some(to.ok_or_else(|| {
                                        StdError::generic_err(
                                            "No recipient sent with invoke.".to_string(),
                                        )
                                    })?),
                                    offer,
                                    expected_return
                                );
                            }
                        }
                        _ => continue,
                    }
                }

                Err(StdError::generic_err(
                    "No matching token in pair".to_string(),
                ))
            }
            InvokeMsg::RemoveLiquidity { from, single_sided, single_sided_withdraw_in_token0} => {
                if config.lp_token.address != info.sender {
                    return Err(StdError::generic_err(
                        "LP Token was not sent to remove liquidity.".to_string(),
                    ));
                }
                match from {
                    Some(address) => remove_liquidity(deps, env, amount, address, single_sided, single_sided_withdraw_in_token0),
                    None => remove_liquidity(deps, env, amount, from_caller, single_sided, single_sided_withdraw_in_token0),
                }
            }
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::GetPairInfo {} => {
                let config = config_r(deps.storage).load()?;
                let balances = config.pair.query_balances(
                    deps,
                    env.contract.address.to_string(),
                    config.viewing_key.0,
                )?;
                let total_liquidity = query_total_supply(deps, &config.lp_token)?;
                to_binary(&QueryMsgResponse::GetPairInfo {
                    liquidity_token: config.lp_token,
                    factory: config.factory_contract,
                    pair: config.pair,
                    amount_0: balances[0],
                    amount_1: balances[1],
                    total_liquidity,
                    contract_version: AMM_PAIR_CONTRACT_VERSION,
                })
            }
            QueryMsg::GetTradeHistory {
                api_key,
                pagination,
            } => {
                let config = config_r(deps.storage).load()?;
                query_factory_authorize_api_key(deps, &config.factory_contract, api_key)?;
                let data = load_trade_history_query(deps, pagination)?;
                to_binary(&QueryMsgResponse::GetTradeHistory { data })
            },
            QueryMsg::GetWhiteListAddress {} => {
                let stored_addr = whitelist_r(deps.storage).load()?;
                to_binary(&QueryMsgResponse::GetWhiteListAddress {
                    addresses: stored_addr,
                })
            }
            QueryMsg::GetTradeCount {} => {
                let count = trade_count_r(deps.storage).may_load()?.unwrap_or(0u64);
                to_binary(&QueryMsgResponse::GetTradeCount { count })
            }
            QueryMsg::GetStakingContract {} => {
                let staking_contract = config_r(deps.storage).load()?.staking_contract;
                to_binary(&QueryMsgResponse::StakingContractInfo {
                    staking_contract: staking_contract,
                })
            }
            QueryMsg::GetEstimatedPrice { offer, exclude_fee } => {
                let swap_result = query_calculate_price(deps, env, offer, exclude_fee)?;
                to_binary(&QueryMsgResponse::EstimatedPrice {
                    estimated_price: swap_result.price,
                })
            }
            QueryMsg::SwapSimulation { offer } => swap_simulation(deps, env, offer),
            QueryMsg::GetShadeDaoInfo {} => get_shade_dao_info(deps),
            QueryMsg::GetEstimatedLiquidity { deposit } => {
                get_estimated_lp_token(deps, env, deposit)
            }
            QueryMsg::GetConfig {} => {
                let config = config_r(deps.storage).load()?;
                return to_binary(&QueryMsgResponse::GetConfig {
                    factory_contract: config.factory_contract,
                    lp_token: config.lp_token,
                    staking_contract: config.staking_contract,
                    pair: config.pair,
                    custom_fee: config.custom_fee,
                });
            }
        },
        BLOCK_SIZE,
    )
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    pad_response_result(
        match (msg.id, msg.result) {
            (INSTANTIATE_LP_TOKEN_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
                Some(x) => {
                    let contract_address =
                        deps.api.addr_validate(&String::from_utf8(x.to_vec())?)?;
                    let config = config_r(deps.storage).load()?;
                    register_lp_token(
                        deps,
                        _env,
                        Contract {
                            address: contract_address,
                            code_hash: config.lp_token.code_hash,
                        },
                    )
                }
                None => Err(StdError::generic_err(format!("Unknown reply id"))),
            },
            (INSTANTIATE_STAKING_CONTRACT_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
                Some(x) => {
                    let contract_address = String::from_utf8(x.to_vec())?;
                    let config = config_r(deps.storage).load()?;
                    set_staking_contract(
                        deps.storage,
                        Some(Contract {
                            address: deps.api.addr_validate(&contract_address)?,
                            code_hash: config
                                .staking_contract_init
                                .ok_or(StdError::generic_err(
                                    "Staking contract does not match.".to_string(),
                                ))?
                                .contract_info
                                .code_hash,
                        }),
                    )
                }
                None => Err(StdError::generic_err(format!("Unknown reply id"))),
            },
            _ => Err(StdError::generic_err(format!("Unknown reply id"))),
        },
        BLOCK_SIZE,
    )
}
