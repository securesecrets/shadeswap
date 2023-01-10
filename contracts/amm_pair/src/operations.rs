use std::{
    collections::hash_map::DefaultHasher,
    convert::TryFrom,
    hash::{Hash, Hasher},
};

use cosmwasm_std::{
    to_binary, Addr, Attribute, BankMsg, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Storage, SubMsg, Uint128, Uint256, WasmMsg,
};
use shadeswap_shared::{
    core::{Fee, TokenAmount, TokenPairAmount, TokenType, ViewingKey},
    msg::{
        amm_pair::{ArbitrageCallback, SwapInfo, SwapResult, TradeHistory},
        staking::{InitMsg as StakingInitMsg, InvokeMsg as StakingInvokeMsg},
    },
    snip20::{
        helpers::{
            burn_msg, mint_msg, register_receive, send_msg, set_viewing_key_msg, transfer_from_msg,
        },
        ExecuteMsg as SNIP20ExecuteMsg,
    },
    utils::calc::sqrt,
    Contract,
};

use crate::{
    contract::ARBITRAGE_CONTRACT_REPLY_ID,
    contract::INSTANTIATE_STAKING_CONTRACT_REPLY_ID,
    query::{self, factory_config},
    state::{
        config_r, config_w, trade_count_r, trade_count_w, trade_history_w, whitelist_r,
        whitelist_w, Config,
    },
};

// Add address to whitelist to exclude from fees
pub fn add_whitelist_address(storage: &mut dyn Storage, address: Addr) -> StdResult<()> {
    let mut unwrap_data = match whitelist_r(storage).may_load() {
        Ok(v) => v.unwrap_or(Vec::new()),
        Err(_) => Vec::new(),
    };
    unwrap_data.push(address);
    whitelist_w(storage).save(&unwrap_data)
}

// Register an LP Token and initialize staking contract with token if initialized with one
pub fn register_lp_token(
    deps: DepsMut,
    env: &Env,
    lp_token_address: Contract,
) -> StdResult<Response> {
    let mut config = config_r(deps.storage).load()?;

    config.lp_token = Contract {
        address: lp_token_address.address.clone(),
        code_hash: lp_token_address.code_hash.clone(),
    };
    // store config against Smart contract address
    config_w(deps.storage).save(&config)?;

    let mut response = Response::new().add_message(register_receive(
        env.contract.code_hash.clone(),
        None,
        &lp_token_address.clone(),
    )?);

    // Initialize Staking Contract
    match config.staking_contract_init {
        Some(c) => {
            match config.factory_contract {
                Some(factory_contract) => {
                    // Gets config from factory
                    let factory_config = factory_config(deps.as_ref(), &factory_contract)?;
                    response = response.add_submessage(SubMsg::reply_on_success(
                        CosmosMsg::Wasm(WasmMsg::Instantiate {
                            code_id: c.contract_info.id,
                            label: format!(
                                "ShadeSwap-Pair-Staking-Contract-{}",
                                &env.contract.address
                            ),
                            msg: to_binary(&StakingInitMsg {
                                daily_reward_amount: c.daily_reward_amount,
                                reward_token: c.reward_token.clone(),
                                pair_contract: Contract {
                                    address: env.contract.address.clone(),
                                    code_hash: env.contract.code_hash.clone(),
                                },
                                prng_seed: config.prng_seed.clone(),
                                lp_token: Contract {
                                    address: lp_token_address.address.clone(),
                                    code_hash: lp_token_address.code_hash.clone(),
                                },
                                //default to same permit authenticator as factory
                                authenticator: factory_config.authenticator,
                                //default to same admin as factory
                                admin_auth: factory_config.admin_auth,
                                valid_to: c.valid_to,
                            })?,
                            code_hash: c.contract_info.code_hash.clone(),
                            funds: vec![],
                        }),
                        INSTANTIATE_STAKING_CONTRACT_REPLY_ID,
                    ));
                }
                None => {                    
                      response = response.add_submessage(SubMsg::reply_on_success(
                          CosmosMsg::Wasm(WasmMsg::Instantiate {
                              code_id: c.contract_info.id,
                              label: format!(
                                  "ShadeSwap-Pair-Staking-Contract-{}",
                                  &env.contract.address
                              ),
                              msg: to_binary(&StakingInitMsg {
                                  daily_reward_amount: c.daily_reward_amount,
                                  reward_token: c.reward_token.clone(),
                                  pair_contract: Contract {
                                      address: env.contract.address.clone(),
                                      code_hash: env.contract.code_hash.clone(),
                                  },
                                  prng_seed: config.prng_seed.clone(),
                                  lp_token: Contract {
                                      address: lp_token_address.address.clone(),
                                      code_hash: lp_token_address.code_hash.clone(),
                                  },
                                  //default to same permit authenticator as factory
                                  authenticator: None,
                                  //default to same admin as factory
                                  admin_auth: config.admin_auth,
                                  valid_to: c.valid_to,
                              })?,
                              code_hash: c.contract_info.code_hash.clone(),
                              funds: vec![],
                          }),
                          INSTANTIATE_STAKING_CONTRACT_REPLY_ID,
                      ));
                }
            }
        }
        _ => {
            ();
        }
    }

    Ok(response)
}

// Register VK and recieve for a given pair token
pub fn register_pair_token(
    env: &Env,
    messages: &mut Vec<CosmosMsg>,
    token: &TokenType,
    viewing_key: &ViewingKey,
) -> StdResult<()> {
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = token
    {
        messages.push(set_viewing_key_msg(
            viewing_key.0.clone(),
            None,
            &Contract {
                address: contract_addr.clone(),
                code_hash: token_code_hash.to_string(),
            },
        )?);
        messages.push(register_receive(
            env.contract.code_hash.clone(),
            None,
            &Contract {
                address: contract_addr.clone(),
                code_hash: token_code_hash.to_string(),
            },
        )?);
    }

    Ok(())
}

// Initiate a swap
pub fn swap(
    deps: DepsMut,
    env: Env,
    config: Config,
    sender: Addr,
    recipient: Option<Addr>,
    offer: TokenAmount,
    expected_return: Option<Uint128>,
    arbitrage_info: Option<ArbitrageCallback>,
) -> StdResult<Response> {
    let swaper_receiver = recipient.unwrap_or(sender.clone());

    let fee_info = query::fee_info(deps.as_ref(), &env)?;
    // check if user whitelist
    let is_user_whitelist = is_address_in_whitelist(deps.storage, &sender)?;
    let swap_result = calculate_swap_result(
        deps.as_ref(),
        &env,
        fee_info.lp_fee,
        fee_info.shade_dao_fee,
        &config,
        &offer,
        Some(is_user_whitelist),
    )?;

    // check for the slippage expected value compare to actual value
    if let Some(expected_return) = expected_return {
        if swap_result.result.return_amount.lt(&expected_return) {
            return Err(StdError::generic_err(
                "Operation fell short of expected_return",
            ));
        }
    }

    //get non-offer token
    let non_offer_token = if &config.pair.0 == &offer.token {
        &config.pair.1
    } else {
        &config.pair.0
    };

    // Send Shade_Dao_Fee back to shade_dao_address which is 0.1%
    let mut messages = Vec::with_capacity(2);
    if !swap_result.shade_dao_fee_amount.is_zero() && fee_info.shade_dao_address.to_string() != ""{
        add_send_token_to_address_msg(
            &mut messages,
            fee_info.shade_dao_address,
            &non_offer_token,
            swap_result.shade_dao_fee_amount,
        )?;
    }      

    // Send Token to Buyer or Swapper
    let index = config
        .pair
        .get_token_index(&offer.token)
        .expect("The token is not in this contract"); // Safe, checked in do_swap
    let token = config
        .pair
        .get_token(index ^ 1)
        .expect("The token is not in this contract");
    messages.push(token.create_send_msg(
        env.contract.address.to_string(),
        swaper_receiver.to_string(),
        swap_result.result.return_amount,
    )?);
    let mut action = "".to_string();
    if index == 0 {
        action = "BUY".to_string();
    }
    if index == 1 {
        action = "SELL".to_string();
    }
    let trade_history = TradeHistory {
        price: swap_result.price,
        amount_in: swap_result.result.return_amount,
        amount_out: offer.amount,
        timestamp: env.block.time.seconds(),
        height: env.block.height,
        direction: action.to_string(),
        lp_fee_amount: swap_result.lp_fee_amount,
        total_fee_amount: swap_result.total_fee_amount,
        shade_dao_fee_amount: swap_result.shade_dao_fee_amount,
    };

    store_trade_history(deps, &trade_history)?;

    let mut arb_msg = None;
    match config.arbitrage_contract {
        Some(arbitrage_contract) => {
            if let Some(arbitrage_info) = arbitrage_info {
                if arbitrage_info.execute {
                    let mut sub_msg = SubMsg::reply_always(
                        CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: arbitrage_contract.address.to_string(),
                            code_hash: arbitrage_contract.code_hash,
                            msg: arbitrage_info.msg,
                            funds: vec![],
                        }),
                        ARBITRAGE_CONTRACT_REPLY_ID,
                    );
                    sub_msg.gas_limit = arbitrage_info.gas_limit;
                    arb_msg = Some(sub_msg);
                }
            }
        }
        _ => arb_msg = None,
    }

    match arb_msg {
        Some(sub_msg) => Ok(Response::new()
            .add_messages(messages)
            .add_submessage(sub_msg)),
        None => Ok(Response::new().add_messages(messages)),
    }
}

// Set staking contract within the config
pub fn set_staking_contract(
    storage: &mut dyn Storage,
    staking_contract: Option<Contract>,
) -> StdResult<Response> {
    let mut config = config_w(storage).load()?;

    config.staking_contract = staking_contract;

    config_w(storage).save(&config)?;

    // send lp contractLink to staking contract
    Ok(Response::new())
}

// Calculate the outcome given an offer
pub fn calculate_swap_result(
    deps: Deps,
    env: &Env,
    lp_fee: Fee,
    shade_dao_fee: Fee,
    config: &Config,
    offer: &TokenAmount,
    exclude_fee: Option<bool>,
) -> StdResult<SwapInfo> {
    if !config.pair.contains(&offer.token) {
        return Err(StdError::generic_err(format!(
            "The required token {}, is not presented in this contract.",
            offer.token
        )));
    }

    let amount = Uint128::from(offer.amount);
    let tokens_pool = calculate_token_pool_balance(deps, env, config, offer)?;
    let token_in_pool = tokens_pool[0];
    let token_out_pool = tokens_pool[1];

    let swap_return_before_fee = calculate_price(amount, token_in_pool, token_out_pool)?;

    let mut lp_fee_amount = Uint128::zero();
    let mut shade_dao_fee_amount = Uint128::zero();

    if exclude_fee.is_none() || !exclude_fee.unwrap() {
        //unwrap safe because of conditional short circuiting
        match &config.custom_fee {
            Some(f) => {
                lp_fee_amount = calculate_fee(swap_return_before_fee, f.lp_fee)?;
                shade_dao_fee_amount = calculate_fee(swap_return_before_fee, f.shade_dao_fee)?;
            }
            None => {
                lp_fee_amount = calculate_fee(swap_return_before_fee, lp_fee)?;
                shade_dao_fee_amount = calculate_fee(swap_return_before_fee, shade_dao_fee)?;
            }
        }
    }
    let total_fee_amount = lp_fee_amount + shade_dao_fee_amount;
    let final_swap_return = swap_return_before_fee - total_fee_amount;

    let result_swap = SwapResult {
        return_amount: final_swap_return,
    };

    Ok(SwapInfo {
        lp_fee_amount: lp_fee_amount,
        shade_dao_fee_amount: shade_dao_fee_amount,
        total_fee_amount: total_fee_amount,
        result: result_swap,
        price: Decimal::from_ratio(final_swap_return, amount).to_string(),
    })
}

pub fn add_address_to_whitelist(storage: &mut dyn Storage, address: Addr) -> StdResult<Response> {
    add_whitelist_address(storage, address)?;
    Ok(Response::default()
        .add_attributes(vec![Attribute::new("action", "save_address_to_whitelist")]))
}

pub fn remove_addresses_from_whitelist(
    storage: &mut dyn Storage,
    addresses_to_remove: Vec<Addr>,
) -> StdResult<Response> {
    let mut addresses = whitelist_r(storage).load()?;
    for address in addresses_to_remove {
        addresses.retain(|x| x != &address);
    }
    whitelist_w(storage).save(&addresses)?;
    Ok(Response::default().add_attribute("action", "remove_address_from_whitelist"))
}

// Executes a virtual swap of the excess provided token for the other, balancing the lp provided
//if the messages param is provided, a message is added which sends the shade dao fee to the dao
pub fn lp_virtual_swap(
    deps: Deps,
    env: &Env,
    sender: Addr,
    lp_fee: Fee,
    shade_dao_fee: Fee,
    shade_dao_address: Addr,
    config: &Config,
    deposit: &TokenPairAmount,
    total_lp_token_supply: Uint128,
    pool_balances: [Uint128; 2],
    messages: Option<&mut Vec<CosmosMsg>>,
) -> StdResult<TokenPairAmount> {
    let mut new_deposit = deposit.clone();

    if !total_lp_token_supply.is_zero() {
        //determine which token should be swapped for other
        let ten_to_18th = Uint128::from(1_000_000_000_000_000_000u128);
        let token0_ratio = (deposit.amount_0 * ten_to_18th) / pool_balances[0]; //actual decimal doesn't matter here since these values are only compared to each other, never used in math
        let token1_ratio = (deposit.amount_1 * ten_to_18th) / pool_balances[1];
        let is_user_whitelist = is_address_in_whitelist(deps.storage, &sender)?;
        if token0_ratio > token1_ratio {
            let extra_token0_amount = deposit.amount_0
                - pool_balances[0].multiply_ratio(deposit.amount_1, pool_balances[1]);
            let half_of_extra = extra_token0_amount / Uint128::from(2u32);

            if half_of_extra > Uint128::zero() {
                let offer = TokenAmount {
                    token: new_deposit.pair.0.clone(),
                    amount: half_of_extra,
                };

                let swap = calculate_swap_result(
                    deps,
                    env,
                    lp_fee,
                    shade_dao_fee,
                    &config,
                    &offer,
                    Some(is_user_whitelist),
                )?;
                if let Some(msgs) = messages {        
                    if !swap.shade_dao_fee_amount.is_zero() && shade_dao_address.to_string() != ""{
                        add_send_token_to_address_msg(
                            msgs,
                            shade_dao_address,
                            &new_deposit.pair.1.clone(),
                            swap.shade_dao_fee_amount,
                        )?;  
                    }                 
                }

                new_deposit.amount_0 = deposit.amount_0 - half_of_extra;
                new_deposit.amount_1 = deposit.amount_1 + swap.result.return_amount;
            }
        } else if token1_ratio > token0_ratio {
            let extra_token1_amount = deposit.amount_1
                - pool_balances[1].multiply_ratio(deposit.amount_0, pool_balances[0]);
            let half_of_extra = extra_token1_amount / Uint128::from(2u32);

            if half_of_extra > Uint128::zero() {
                let offer = TokenAmount {
                    token: new_deposit.pair.1.clone(),
                    amount: half_of_extra,
                };

                let swap = calculate_swap_result(
                    deps,
                    env,
                    lp_fee,
                    shade_dao_fee,
                    &config,
                    &offer,
                    Some(is_user_whitelist),
                )?;
                if let Some(msgs) = messages {
                    if !swap.shade_dao_fee_amount.is_zero() && shade_dao_address.to_string() != ""{
                        add_send_token_to_address_msg(
                            msgs,
                            shade_dao_address,
                            &new_deposit.pair.0.clone(),
                            swap.shade_dao_fee_amount,
                        )?; 
                    }                                    
                }

                new_deposit.amount_0 = deposit.amount_0 + swap.result.return_amount;
                new_deposit.amount_1 = deposit.amount_1 - half_of_extra;
            }
        }
    }
    Ok(new_deposit)
}

// Remove liquidity from the LP Pool
pub fn remove_liquidity(
    deps: DepsMut,
    env: Env,
    amount: Uint128,
    from: Addr,
    single_sided_withdraw_type: Option<TokenType>,
    single_sided_expected_return: Option<Uint128>,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;

    let liquidity_pair_contract = query::total_supply(deps.as_ref(), &config.lp_token)?;
    let pool_balances = config.pair.query_balances(
        deps.as_ref(),
        env.contract.address.to_string(),
        config.viewing_key.0.clone(),
    )?;
    let withdraw_amount = amount;
    let total_liquidity = liquidity_pair_contract;

    let mut pool_withdrawn: [Uint128; 2] = [Uint128::zero(), Uint128::zero()];

    for (i, pool_amount) in pool_balances.iter().enumerate() {
        let pool_amount = *pool_amount;
        pool_withdrawn[i] = pool_amount.multiply_ratio(withdraw_amount, total_liquidity)
    }

    //if user wants purely one token, virtually swap entire withdraw into that token
    if let Some(withdraw_type) = single_sided_withdraw_type {
        let fee_info = query::fee_info(deps.as_ref(), &env)?;
        let withdraw_in_token0: bool = if config.pair.contains(&withdraw_type) {
            Ok(config.pair.0 == withdraw_type)
        } else {
            Err(StdError::generic_err(
                "Single sided withdraw token type was set, but token is not included in this pair",
            ))
        }?;

        if withdraw_in_token0 {
            let offer = TokenAmount {
                token: config.pair.1.clone(),
                amount: pool_withdrawn[1],
            };

            let swap = calculate_swap_result(
                deps.as_ref(),
                &env,
                fee_info.lp_fee,
                fee_info.shade_dao_fee,
                &config,
                &offer,
                Some(false),
            )?;

            pool_withdrawn[0] += swap.result.return_amount;
            pool_withdrawn[1] = Uint128::zero();

            if let Some(min_return) = single_sided_expected_return {
                if pool_withdrawn[0] < min_return {
                    return Err(StdError::generic_err(
                        "Single sided withdraw returned less than the expected amount",
                    ));
                }
            }
        } else {
            //withdraw in token 1
            let offer = TokenAmount {
                token: config.pair.0.clone(),
                amount: pool_withdrawn[0],
            };
            let swap = calculate_swap_result(
                deps.as_ref(),
                &env,
                fee_info.lp_fee,
                fee_info.shade_dao_fee,
                &config,
                &offer,
                Some(false),
            )?;

            pool_withdrawn[0] = Uint128::zero();
            pool_withdrawn[1] += swap.result.return_amount;

            if let Some(min_return) = single_sided_expected_return {
                if pool_withdrawn[1] < min_return {
                    return Err(StdError::generic_err(
                        "Single sided withdraw returned less than the expected amount",
                    ));
                }
            }
        }
    }

    let mut pair_messages: Vec<CosmosMsg> = Vec::with_capacity(4);

    for (i, token) in config.pair.into_iter().enumerate() {
        if !pool_withdrawn[i].is_zero() {
            pair_messages.push(token.create_send_msg(
                env.contract.address.to_string(),
                from.to_string(),
                pool_withdrawn[i],
            )?);
        }
    }

    pair_messages.push(burn_msg(
        amount,
        None,
        None,
        &Contract {
            address: config.lp_token.address,
            code_hash: config.lp_token.code_hash,
        },
    )?);
    Ok(Response::new()
        .add_messages(pair_messages)
        .add_attributes(vec![
            Attribute::new("action", "remove_liquidity"),
            Attribute::new("withdrawn_share", amount),
            Attribute::new(
                "refund_assets",
                format!("{}, {}", &config.pair.0, &config.pair.1),
            ),
            Attribute::new("refund_amount0", pool_withdrawn[0]),
            Attribute::new("refund_amount1", pool_withdrawn[1]),
        ]))
}

// Calculate the price given LP information
pub fn calculate_price(
    amount: Uint128,
    token_in_pool_balance: Uint128,
    token_out_pool_balance: Uint128,
) -> StdResult<Uint128> {
    Ok(token_out_pool_balance.multiply_ratio(amount, token_in_pool_balance + amount))
}

// Add liquidity to pool
pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    deposit: TokenPairAmount,
    expected_return: Option<Uint128>,
    staking: Option<bool>,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;

    if config.pair != deposit.pair {
        return Err(StdError::generic_err(
            "The provided tokens dont match those managed by the contract.",
        ));
    }

    let mut pair_messages: Vec<CosmosMsg> = vec![];
    let mut pool_balances = deposit.pair.query_balances(
        deps.as_ref(),
        env.contract.address.to_string(),
        config.viewing_key.0.clone(),
    )?;
    for (i, (amount, token)) in deposit.into_iter().enumerate() {
        match &token {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                pair_messages.push(transfer_from_msg(
                    info.sender.to_string(),
                    env.contract.address.to_string(),
                    amount,
                    None,
                    None,
                    &Contract {
                        address: contract_addr.clone(),
                        code_hash: token_code_hash.clone(),
                    },
                )?);
            }
            TokenType::NativeToken { .. } => {
                // If the asset is native token, balance is already increased.
                // To calculate properly we should subtract user deposit from the pool.
                token.assert_sent_native_token_balance(info, amount)?;
                pool_balances[i] = pool_balances[i] - amount;
            }
        }
    }

    let fee_info = query::fee_info(deps.as_ref(), &env)?;

    let pair_contract_pool_liquidity = query::total_supply(deps.as_ref(), &config.lp_token)?;

    let new_deposit = lp_virtual_swap(
        deps.as_ref(),
        &env,
        info.sender.clone(),
        fee_info.lp_fee,
        fee_info.shade_dao_fee,
        fee_info.shade_dao_address,
        &config,
        &deposit,
        pair_contract_pool_liquidity,
        pool_balances,
        Some(&mut pair_messages),
    )?;

    let lp_tokens = calculate_lp_tokens(&new_deposit, pool_balances, pair_contract_pool_liquidity)?;

    if let Some(e) = expected_return {
        if e > lp_tokens {
            return Err(StdError::generic_err(format!(
                "Operation returns less then expected ({} < {}).",
                e, lp_tokens
            )));
        }
    }

    let add_to_staking;
    // check if user wants add his LP token to Staking
    match staking {
        Some(s) => {
            if s {
                // check if the Staking Contract has been set for AMM Pairs
                match config.staking_contract {
                    Some(stake) => {
                        add_to_staking = true;
                        pair_messages.push(mint_msg(
                            env.contract.address.clone(),
                            lp_tokens,
                            None,
                            None,
                            &Contract {
                                address: config.lp_token.address.clone(),
                                code_hash: config.lp_token.code_hash.clone(),
                            },
                        )?);
                        let invoke_msg = to_binary(&StakingInvokeMsg::Stake {
                            from: info.sender.to_string(),
                        })?;
                        // SEND LP Token to Staking Contract with Staking Message
                        let msg = to_binary(&SNIP20ExecuteMsg::Send {
                            recipient: stake.address.to_string(),
                            recipient_code_hash: Some(stake.code_hash.clone()),
                            amount: lp_tokens,
                            msg: Some(invoke_msg.clone()),
                            memo: None,
                            padding: None,
                        })?;
                        pair_messages.push(
                            WasmMsg::Execute {
                                contract_addr: config.lp_token.address.to_string(),
                                code_hash: config.lp_token.code_hash.clone(),
                                msg,
                                funds: vec![],
                            }
                            .into(),
                        );
                    }
                    None => {
                        return Err(StdError::generic_err(
                            "Staking Contract has not been set for AMM Pairs",
                        ))
                    }
                }
            } else {
                add_to_staking = false;
                pair_messages.push(mint_msg(
                    info.sender.clone(),
                    lp_tokens,
                    None,
                    None,
                    &Contract {
                        address: config.lp_token.address.clone(),
                        code_hash: config.lp_token.code_hash.clone(),
                    },
                )?);
            }
        }
        None => {
            add_to_staking = false;
            pair_messages.push(mint_msg(
                info.sender.clone(),
                lp_tokens,
                None,
                None,
                &Contract {
                    address: config.lp_token.address.clone(),
                    code_hash: config.lp_token.code_hash.clone(),
                },
            )?);
        }
    }

    Ok(Response::new()
        .add_messages(pair_messages)
        .add_attributes(vec![
            Attribute::new("staking", format!("{}", add_to_staking)),
            Attribute::new("action", "add_liquidity_to_pair_contract"),
            Attribute::new("assets", format!("{}, {}", deposit.pair.0, deposit.pair.1)),
            Attribute::new("share_pool", lp_tokens),
        ]))
}

pub fn update_viewing_key(env: Env, deps: DepsMut, viewing_key: String, config: &mut Config) -> StdResult<Response> {
    let mut messages = vec![];
    let new_viewing_key = ViewingKey(viewing_key);
    register_pair_token(&env, &mut messages, &config.pair.0, &new_viewing_key)?;
    register_pair_token(&env, &mut messages, &config.pair.1, &new_viewing_key)?;

    config.viewing_key = new_viewing_key;
    config_w(deps.storage).save(&config)?;
    let mut response = Response::new();
    response = response.add_messages(messages);
    Ok(response)
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

// Checks whether address is in whitelist
pub fn is_address_in_whitelist(storage: &dyn Storage, address: &Addr) -> StdResult<bool> {
    let addrs = whitelist_r(storage).may_load()?;
    match addrs {
        Some(a) => {
            if a.contains(address) {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
        None => return Ok(false),
    }
}

fn store_trade_history(deps: DepsMut, trade_history: &TradeHistory) -> StdResult<()> {
    let count: u64 = match trade_count_r(deps.storage).may_load() {
        Ok(it) => it.unwrap_or(0),
        Err(_) => 0,
    };
    let update_count = count + 1;
    trade_count_w(deps.storage).save(&update_count)?;
    trade_history_w(deps.storage).save(update_count.to_string().as_bytes(), &trade_history)
}

fn add_send_token_to_address_msg(
    messages: &mut Vec<CosmosMsg>,
    address: Addr,
    token: &TokenType,
    amount: Uint128,
) -> StdResult<()> {
    if amount > Uint128::zero() {
        match &token {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                messages.push(send_msg(
                    address,
                    amount,
                    None,
                    None,
                    None,
                    &Contract {
                        address: contract_addr.clone(),
                        code_hash: token_code_hash.to_string(),
                    },
                )?);
            }
            TokenType::NativeToken { denom } => {
                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: address.to_string(),
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount,
                    }],
                }));
            }
        }
    }
    Ok(())
}

fn calculate_fee(amount: Uint128, fee: Fee) -> StdResult<Uint128> {
    if fee.denom == 0u16 {
        return Ok(Uint128::zero())
    }
    let amount = amount.multiply_ratio(fee.nom, fee.denom);
    Ok(amount)
}

fn calculate_token_pool_balance(
    deps: Deps,
    env: &Env,
    config: &Config,
    swap_offer: &TokenAmount,
) -> StdResult<[Uint128; 2]> {
    let tokens_balances = config.pair.query_balances(
        deps,
        env.contract.address.to_string(),
        config.viewing_key.0.clone(),
    )?;
    if let Some(index) = config.pair.get_token_index(&swap_offer.token) {
        let token_in_pool = tokens_balances[index];
        let token_out_pool = tokens_balances[index ^ 1];

        Ok([token_in_pool, token_out_pool])
    } else {
        Err(StdError::generic_err(
            "The offered token is not traded on this contract".to_string(),
        ))
    }
}

pub fn calculate_lp_tokens(
    deposit: &TokenPairAmount,
    pool_balances: [Uint128; 2],
    pair_contract_pool_liquidity: Uint128,
) -> Result<Uint128, StdError> {
    let lp_tokens: Uint128;
    if pair_contract_pool_liquidity == Uint128::zero() {
        // If user mints new liquidity pool -> liquidity % = sqrt(x * y) where
        // x and y is amount of token0 and token1 provided
        let deposit_token0_amount = Uint256::from(deposit.amount_0);
        let deposit_token1_amount = Uint256::from(deposit.amount_1);
        lp_tokens = Uint128::try_from(sqrt(deposit_token0_amount * deposit_token1_amount)?)?;
    } else {
        // Total % of Pool
        let total_share = pair_contract_pool_liquidity;
        // Deposit amounts of the tokens
        let deposit_token0_amount = deposit.amount_0;
        let deposit_token1_amount = deposit.amount_1;

        // get token pair balance
        let token0_pool = pool_balances[0];
        let token1_pool = pool_balances[1];
        // Calcualte new % of Pool
        let percent_token0_pool = deposit_token0_amount.multiply_ratio(total_share, token0_pool);
        let percent_token1_pool = deposit_token1_amount.multiply_ratio(total_share, token1_pool);
        lp_tokens = std::cmp::min(percent_token0_pool, percent_token1_pool);
    };
    Ok(lp_tokens)
}
