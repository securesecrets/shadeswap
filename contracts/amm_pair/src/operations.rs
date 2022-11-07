use std::{
    collections::hash_map::DefaultHasher,
    convert::TryFrom,
    hash::{Hash, Hasher},
};

use cosmwasm_std::{
    to_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, QuerierWrapper, QueryRequest, Response, StdError, StdResult, Storage, SubMsg,
    Uint128, Uint256, WasmMsg, WasmQuery,
};
use shadeswap_shared::{
    amm_pair::AMMSettings,
    core::{Fee, TokenAmount, TokenPairAmount, TokenType, ViewingKey},
    msg::{
        amm_pair::{QueryMsgResponse, SwapInfo, SwapResult, TradeHistory},
        factory::{QueryMsg as FactoryQueryMsg, QueryResponse as FactoryQueryResponse},
        staking::{InitMsg as StakingInitMsg, InvokeMsg as StakingInvokeMsg},
    },
    snip20::{
        helpers::{
            burn_msg, mint_msg, register_receive, send_msg, set_viewing_key_msg, token_info,
            transfer_from_msg,
        },
        ExecuteMsg as SNIP20ExecuteMsg,
    },
    utils::calc::sqrt,
    Contract, Pagination,
};

use crate::{
    contract::INSTANTIATE_STAKING_CONTRACT_REPLY_ID,
    state::{
        config_r, config_w, trade_count_r, trade_count_w, trade_history_r, trade_history_w,
        whitelist_r, whitelist_w, Config, PAGINATION_LIMIT,
    },
};

// WHITELIST
pub fn add_whitelist_address(storage: &mut dyn Storage, address: Addr) -> StdResult<()> {
    let mut unwrap_data = match whitelist_r(storage).may_load() {
        Ok(v) => v.unwrap_or(Vec::new()),
        Err(_) => Vec::new(),
    };
    unwrap_data.push(address);
    whitelist_w(storage).save(&unwrap_data)
}

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

fn load_trade_history(deps: Deps, count: u64) -> StdResult<TradeHistory> {
    let trade_history: TradeHistory =
        trade_history_r(deps.storage).load(count.to_string().as_bytes())?;
    Ok(trade_history)
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

pub fn register_lp_token(
    deps: DepsMut,
    env: Env,
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

    let factory_config = query_factory_config(deps.as_ref(), &config.factory_contract)?;

    match config.staking_contract_init {
        Some(c) => {
            response = response.add_submessage(SubMsg::reply_on_success(
                CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: c.contract_info.id,
                    label: format!("ShadeSwap-Pair-Staking-Contract-{}", &env.contract.address),
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
                        authenticator: factory_config.authenticator,
                        //default to same admin as amm_pair
                        admin_auth: factory_config.admin_auth,
                        valid_to: c.valid_to
                    })?,
                    code_hash: c.contract_info.code_hash.clone(),
                    funds: vec![],
                }),
                INSTANTIATE_STAKING_CONTRACT_REPLY_ID,
            ));
        }
        _ => {
            ();
        }
    }

    Ok(response)
}

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

pub fn query_calculate_price(
    deps: Deps,
    env: Env,
    offer: TokenAmount,
    exclude_fee: Option<bool>,
) -> StdResult<SwapInfo> {
    let config_settings = config_r(deps.storage).load()?;
    let amm_settings = query_factory_config(deps, &config_settings.factory_contract)?.amm_settings;
    let swap_result = calculate_swap_result(
        deps,
        &env,
        &amm_settings,
        &config_settings,
        &offer,        
        exclude_fee,
    )?;  
    Ok(swap_result)
}

pub fn swap(
    deps: DepsMut,
    env: Env,
    config: Config,
    sender: Addr,
    recipient: Option<Addr>,
    offer: TokenAmount,
    expected_return: Option<Uint128>,
) -> StdResult<Response> {
    let swaper_receiver = recipient.unwrap_or(sender);
    let amm_settings = query_factory_config(deps.as_ref(), &config.factory_contract)?.amm_settings;
    let swap_result = calculate_swap_result(
        deps.as_ref(),
        &env,
        &amm_settings,
        &config,
        &offer,
        None,
    )?;

    // check for the slippage expected value compare to actual value
    if let Some(expected_return) = expected_return {    
        if swap_result.result.return_amount.lt(&expected_return) {
            return Err(StdError::generic_err(
                "Operation fell short of expected_return",
            ));
        }
    }

    // Send Shade_Dao_Fee back to shade_dao_address which is 0.1%
    let mut messages = Vec::with_capacity(2);
    if swap_result.shade_dao_fee_amount > Uint128::zero() {
        match &offer.token {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                messages.push(send_msg(
                    amm_settings.shade_dao_address.address,
                    swap_result.shade_dao_fee_amount,
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
                    to_address: amm_settings.shade_dao_address.address.to_string(),
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount: swap_result.shade_dao_fee_amount,
                    }],
                }));
            }
        }
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
    println!("result {} ", swap_result.result.return_amount);  
    Ok(Response::new().add_messages(messages))
}

pub fn set_staking_contract(
    storage: &mut dyn Storage,
    staking_contract: Option<Contract>,
) -> StdResult<Response> {
    let mut config = config_w(storage).load()?;

    config.staking_contract = staking_contract;

    config_w(storage).save(&config)?;

    // send lp contractLink to staking contract
    Ok(Response::new().add_attribute("action", "set_staking_contract"))
}

pub fn get_shade_dao_info(deps: Deps) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;
    let amm_settings = query_factory_config(deps, &config.factory_contract)?.amm_settings;
    let shade_dao_info = QueryMsgResponse::ShadeDAOInfo {
        shade_dao_address: amm_settings.shade_dao_address.address.to_string(),
        shade_dao_fee: amm_settings.shade_dao_fee,
        admin_auth: config.admin_auth,
        lp_fee: amm_settings.lp_fee,
    };
    to_binary(&shade_dao_info)
}

pub fn swap_simulation(deps: Deps, env: Env, offer: TokenAmount) -> StdResult<Binary> {
    let config_settings = config_r(deps.storage).load()?;
    let amm_settings = query_factory_config(deps, &config_settings.factory_contract)?.amm_settings; 
    let swap_result = calculate_swap_result(
        deps,
        &env,
        &amm_settings,
        &config_settings,
        &offer,
        None    
    )?;
    let simulation_result = QueryMsgResponse::SwapSimulation {
        total_fee_amount: swap_result.total_fee_amount,
        lp_fee_amount: swap_result.lp_fee_amount,
        shade_dao_fee_amount: swap_result.shade_dao_fee_amount,
        result: swap_result.result,
        price: swap_result.price,
    };
    to_binary(&simulation_result)
}

pub fn get_estimated_lp_token(
    deps: Deps,
    env: Env,
    deposit: TokenPairAmount
) -> StdResult<Binary> {
    let config = config_r(deps.storage).load()?;
    let Config {
        pair,
        viewing_key,
        lp_token,
        ..
    } = config;

    if pair != deposit.pair {
        return Err(StdError::generic_err(
            "The provided tokens dont match those managed by the contract.",
        ));
    }

    let pool_balances =
        deposit
            .pair
            .query_balances(deps, env.contract.address.to_string(), viewing_key.0)?;

    let pair_contract_pool_liquidity = query_total_supply(deps, &lp_token)?;
    let lp_tokens = calculate_lp_tokens(
        &deposit,
        pool_balances,
        pair_contract_pool_liquidity,
    )?;
    let response_msg = QueryMsgResponse::EstimatedLiquidity {
        lp_token: lp_tokens,
        total_lp_token: pair_contract_pool_liquidity,
    };
    to_binary(&response_msg)
}

pub fn load_trade_history_query(
    deps: Deps,
    pagination: Pagination,
) -> StdResult<Vec<TradeHistory>> {
    let count = trade_count_r(deps.storage).may_load()?.unwrap_or(0u64);

    if pagination.start >= count {
        return Ok(vec![]);
    }

    let limit = pagination.limit.min(PAGINATION_LIMIT);
    let end = (pagination.start + limit as u64).min(count);

    let mut result = Vec::with_capacity((end - pagination.start) as usize);

    for i in pagination.start..end {
        let temp_index = i + 1;
        let trade_history: TradeHistory = load_trade_history(deps, temp_index)?;
        result.push(trade_history);
    }

    Ok(result)
}

fn calculate_fee(amount: Uint128, fee: Fee) -> StdResult<Uint128> {
    let amount = amount.multiply_ratio(fee.nom, fee.denom);
    Ok(amount)
}

pub fn calculate_swap_result(
    deps: Deps,
    env: &Env,
    settings: &AMMSettings,
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
    // conver tand get avialble balance
    let tokens_pool = get_token_pool_balance(deps, env, config, offer)?;
    let token0_pool = tokens_pool[0];
    let token1_pool = tokens_pool[1];   
    // calculate fee
    let lp_fee = settings.lp_fee;
    let shade_dao_fee = settings.shade_dao_fee;
    let lp_fee_amount ;
    let shade_dao_fee_amount ;
    // calculation fee
    match &config.custom_fee {
        Some(f) => {
            lp_fee_amount = calculate_fee(offer.amount, f.lp_fee)?;
            shade_dao_fee_amount = calculate_fee(offer.amount, f.shade_dao_fee)?;
        }
        None => {
            lp_fee_amount = calculate_fee(offer.amount, lp_fee)?;
            shade_dao_fee_amount = calculate_fee(offer.amount, shade_dao_fee)?;
        }
    }
    // total fee
    let total_fee_amount = lp_fee_amount + shade_dao_fee_amount;

    // sub fee from offer amount
    let mut deducted_offer_amount = offer.amount - total_fee_amount;
    if let Some(true) = exclude_fee {
        deducted_offer_amount = offer.amount;
    }

    let swap_amount = calculate_price(deducted_offer_amount, token0_pool, token1_pool)?;
    let result_swap = SwapResult {
        return_amount: swap_amount,
    };
    Ok(SwapInfo {
        lp_fee_amount: lp_fee_amount,
        shade_dao_fee_amount: shade_dao_fee_amount,
        total_fee_amount: total_fee_amount,
        result: result_swap,
        price: Decimal::from_ratio(swap_amount, amount).to_string(),
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
    _env: Env,
) -> StdResult<Response> {
    let mut addresses = whitelist_r(storage).load()?;
    for address in addresses_to_remove {
        addresses.retain(|x| x != &address);
    }
    whitelist_w(storage).save(&addresses)?;
    Ok(Response::default().add_attribute("action", "remove_address_from_whitelist"))
}

fn get_token_pool_balance(
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
    let token0_pool = tokens_balances[index];
    let token1_pool = tokens_balances[index ^ 1];

    // conver tand get avialble balance
    let token0_pool = token0_pool;
    let token1_pool = token1_pool;
    Ok([token0_pool, token1_pool])
    }
    else
    {
        Err(StdError::generic_err("The offered token is not traded on this contract".to_string()))
    }
}

pub fn remove_liquidity(
    deps: DepsMut,
    env: Env,
    amount: Uint128,
    from: Addr,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    let Config {
        pair,
        viewing_key,
        lp_token,
        ..
    } = config;

    let liquidity_pair_contract = query_total_supply(deps.as_ref(), &lp_token)?;
    let pool_balances = pair.query_balances(
        deps.as_ref(),
        env.contract.address.to_string(),
        viewing_key.0,
    )?;
    
  
    let withdraw_amount = amount;
    let total_liquidity = liquidity_pair_contract;

    let mut pool_withdrawn: [Uint128; 2] = [Uint128::zero(), Uint128::zero()];

    for (i, pool_amount) in pool_balances.iter().enumerate() {
        let pool_amount = *pool_amount;
        pool_withdrawn[i] = pool_amount.multiply_ratio(withdraw_amount, total_liquidity)
    }

    let mut pair_messages: Vec<CosmosMsg> = Vec::with_capacity(4);

    for (i, token) in pair.into_iter().enumerate() {
        pair_messages.push(token.create_send_msg(
            env.contract.address.to_string(),
            from.to_string(),
            pool_withdrawn[i],
        )?);
    }

    pair_messages.push(burn_msg(
        amount,
        None,
        None,
        &Contract {
            address: lp_token.address,
            code_hash: lp_token.code_hash,
        },
    )?);
    Ok(Response::new()
        .add_messages(pair_messages)
        .add_attributes(vec![
            Attribute::new("action", "remove_liquidity"),
            Attribute::new("withdrawn_share", amount),
            Attribute::new("refund_assets", format!("{}, {}", &pair.0, &pair.1)),
        ]))
    
}

pub fn calculate_price(
    amount: Uint128,
    token0_pool_balance: Uint128,
    token1_pool_balance: Uint128,
) -> StdResult<Uint128> {
    Ok(token1_pool_balance.multiply_ratio(amount, token0_pool_balance + amount))
}

pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    deposit: TokenPairAmount,
    expected_return: Option<Uint128>,
    staking: Option<bool>,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    let Config {
        pair,
        viewing_key,
        lp_token,
        staking_contract,
        ..
    } = config;

    if pair != deposit.pair {
        return Err(StdError::generic_err(
            "The provided tokens dont match those managed by the contract.",
        ));
    }

    let mut pair_messages: Vec<CosmosMsg> = vec![];
    let mut pool_balances = deposit.pair.query_balances(
        deps.as_ref(),
        env.contract.address.to_string(),
        viewing_key.0,
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
    
    let pair_contract_pool_liquidity =
    query_total_supply(deps.as_ref(), &lp_token)?;
    let lp_tokens = calculate_lp_tokens(
        &deposit,
        pool_balances,
        pair_contract_pool_liquidity,
    )?;

  
    if let Some(e) = expected_return 
    {
        if e > lp_tokens {
            return Err(StdError::generic_err(format!("Operation returns less then expected ({} < {}).", e, lp_tokens)));
        }
    }

    let add_to_staking;
    // check if user wants add his LP token to Staking
    match staking {
        Some(s) => {
            if s {
                // check if the Staking Contract has been set for AMM Pairs
                match staking_contract {
                    Some(stake) => {
                        add_to_staking = true;
                        pair_messages.push(mint_msg(
                            env.contract.address.clone(),
                            lp_tokens,
                            None,
                            None,
                            &Contract {
                                address: lp_token.address.clone(),
                                code_hash: lp_token.code_hash.clone(),
                            },
                        )?);
                        let invoke_msg = to_binary(&StakingInvokeMsg::Stake {
                            from: info.sender.clone(),
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
                                contract_addr: lp_token.address.to_string(),
                                code_hash: lp_token.code_hash.clone(),
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
                        address: lp_token.address.clone(),
                        code_hash: lp_token.code_hash.clone(),
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
                    address: lp_token.address.clone(),
                    code_hash: lp_token.code_hash.clone(),
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

fn calculate_lp_tokens(
    deposit: &TokenPairAmount,
    pool_balances: [Uint128; 2],
    pair_contract_pool_liquidity: Uint128,
) -> Result<Uint128, StdError> {

    let lp_tokens: Uint128 ;
    if pair_contract_pool_liquidity.is_zero() {
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

pub fn update_viewing_key(env: Env, deps: DepsMut, viewing_key: String) -> StdResult<Response> {
    let mut config = config_r(deps.storage).load()?;

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

pub fn query_token_symbol(querier: QuerierWrapper, token: &TokenType) -> StdResult<String> {
    match token {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => {
            return Ok(token_info(
                &querier,
                &Contract {
                    address: contract_addr.clone(),
                    code_hash: token_code_hash.clone(),
                },
            )?
            .symbol);
        }
        TokenType::NativeToken { denom: _ } => Ok("SCRT".to_string()),
    }
}

struct FactoryConfig {
    amm_settings: AMMSettings,
    authenticator: Option<Contract>,
    admin_auth: Contract
}

fn query_factory_config(deps: Deps, factory: &Contract) -> StdResult<FactoryConfig> {
    let result: FactoryQueryResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: factory.address.to_string(),
            msg: to_binary(&FactoryQueryMsg::GetConfig {})?,
            code_hash: factory.code_hash.to_string(),
        }))?;

    match result {
        FactoryQueryResponse::GetConfig {
            pair_contract: _,
            amm_settings,
            lp_token_contract: _,
            authenticator,
            admin_auth
        } => Ok(FactoryConfig {
            amm_settings,
            authenticator,
            admin_auth
        }),
        _ => Err(StdError::generic_err(
            "An error occurred while trying to retrieve factory settings.",
        )),
    }
}

pub fn query_factory_authorize_api_key(
    deps: Deps,
    factory: &Contract,
    api_key: String,
) -> StdResult<bool> {
    let result: FactoryQueryResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: factory.address.to_string(),
            msg: to_binary(&FactoryQueryMsg::AuthorizeApiKey { api_key: api_key })?,
            code_hash: factory.code_hash.to_string(),
        }))?;

    match result {
        FactoryQueryResponse::AuthorizeApiKey { authorized } => {
            if !authorized {
                return Err(StdError::generic_err(
                    "Authorization failed, key is incorrect.",
                ));
            }
            Ok(authorized)
        }
        _ => Err(StdError::generic_err(
            "Authorization failed, could not query factory successfully.",
        )),
    }
}

pub fn query_total_supply(deps: Deps, lp_token_info: &Contract) -> StdResult<Uint128> {
    let result = token_info(
        &deps.querier,
        &Contract {
            address: lp_token_info.address.clone(),
            code_hash: lp_token_info.code_hash.clone(),
        },
    )?;

    if let Some(ts) = result.total_supply {
        Ok(ts)
    }
    else
    {
        return Err(StdError::generic_err("LP token has no available supply."));
    }
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
