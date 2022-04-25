use shadeswap_shared::msg::amm_pair::{{InitMsg,QueryMsg,  HandleMsg, InvokeMsg,QueryMsgResponse}};
use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse,QueryMsg as FactoryQueryMsg };
use shadeswap_shared::amm_pair::{{AMMSettings, Fee}};
use shadeswap_shared::token_amount::{{TokenAmount}};
use shadeswap_shared::token_pair_amount::{{TokenPairAmount}};
use shadeswap_shared::token_type::{{TokenType}};
use crate::state::{Config, store_config, load_config};
use crate::help_math::{{substraction, multiply}};
use crate::state::swapdetails::{SwapInfo, SwapResult};
use shadeswap_shared::{ 
    fadroma::{
        scrt::{
            from_binary, log, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Decimal, Env,
            Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryRequest, QueryResult,
            StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery, 
            secret_toolkit::snip20,        
        },
        scrt_uint256::Uint256,
        scrt_callback::Callback,
        scrt_link::ContractLink,
        scrt_vk::ViewingKey,
    },
 
};


use composable_snip20::msg::{{InitMsg as Snip20ComposableMsg, InitConfig as Snip20ComposableConfig}};

const AMM_PAIR_CONTRACT_VERSION: u32 = 1;
pub const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    if msg.pair.0 == msg.pair.1 {
        return Err(StdError::generic_err(
            "Creating Pair Contract with the same token.",
        ));
    }

    let mut messages = vec![];

    let viewing_key = ViewingKey::new(&env, msg.prng_seed.as_slice(), msg.entropy.as_slice());

    register_pair_token(&env, &mut messages, &msg.pair.0, &viewing_key)?;
    register_pair_token(&env, &mut messages, &msg.pair.1, &viewing_key)?;

    // Create LP token and store it
    messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: msg.lp_token_contract.id,
        msg: to_binary(&Snip20ComposableMsg {
            name: format!(
                "ShadeSwap AMM Pair Contract Provider (LP) token for {}-{}",
                &msg.pair.0, &msg.pair.1
            ),
            admin: Some(env.contract.address.clone()),
            symbol: msg.symbol.to_string(),
            decimals: 18,
            callback: Some(Callback {
                msg: to_binary(&HandleMsg::OnLpTokenInitAddr)?,
                contract: ContractLink {
                    address: env.contract.address.clone(),
                    code_hash: env.contract_code_hash,
                },
            }),
            initial_balances: None,
            initial_allowances: None,
            prng_seed: msg.prng_seed,
            config: Some(
                Snip20ComposableConfig::builder()
                    .public_total_supply()
                    .enable_mint()
                    .enable_burn()
                    .build(),
            ),
        })?,
        send: vec![],
        label: format!(
            "{}-{}-ShadeSwap-Pair-Token-{}",
            &msg.pair.0, &msg.pair.1, &env.contract.address
        ),
        callback_code_hash: msg.lp_token_contract.code_hash.clone(),
    }));

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.callback.contract.address,
        callback_code_hash: msg.callback.contract.code_hash,
        msg: msg.callback.msg,
        send: vec![],
    }));

    let config = Config {
        symbol: msg.symbol,
        factory_info: msg.factory_info,
        lp_token_info: ContractLink {
            code_hash: msg.lp_token_contract.code_hash,
            // We get the address when the instantiated LP token calls OnLpTokenInit
            address: HumanAddr::default(),
        },
        pair: msg.pair,
        contract_addr: env.contract.address.clone(),
        viewing_key,
    };

    store_config(deps, &config)?;

    Ok(InitResponse {
        messages,
        log: vec![log("created_exchange_address", env.contract.address)],
    })
}

fn register_lp_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let mut config = load_config(&deps)?;

    // address must be default otherwise it has been initialized.
    if config.lp_token_info.address != HumanAddr::default() {
        return Err(StdError::unauthorized());
    }

    config.lp_token_info.address = env.message.sender.clone();

    // store config against Smart contract address
    store_config(deps, &config)?;

    Ok(HandleResponse {
        messages: vec![snip20::register_receive_msg(
            env.contract_code_hash,
            None,
            BLOCK_SIZE,
            config.lp_token_info.code_hash,
            env.message.sender.clone(),
        )?],
        log: vec![log("liquidity_token_addr", env.message.sender)],
        data: None,
    })
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

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::ReceiveCallback {
            from, amount, msg, ..
        } => receiver_callback(deps, env, from, amount, msg),
        HandleMsg::AddLiquidityToAMMContract {
            deposit,
            slippage,
        } => add_liquidity(deps, env, deposit, slippage),
        HandleMsg::OnLpTokenInitAddr => register_lp_token(deps, env),
        HandleMsg::SwapTokens {
            offer,
            expected_return,
            to,
        } => {

            if !offer.token.is_native_token() {
                return Err(StdError::unauthorized());
            }

            offer.assert_sent_native_token_balance(&env)?;
            let config_settings = load_config(deps)?;
            let sender = env.message.sender.clone();
            swap_tokens(
                &deps.querier,
                env,
                config_settings,
                sender,
                to,
                offer,
                expected_return,
            )
        }
    }
}

fn swap_tokens(
    querier: &impl Querier,
    env: Env,
    config: Config<HumanAddr>,
    sender: HumanAddr,
    recipient: Option<HumanAddr>,
    offer: TokenAmount<HumanAddr>,
    expected_return: Option<Uint128>)-> StdResult<HandleResponse>{ 
  
    let amm_settings = query_factory_amm_settings(querier,config.factory_info.clone())?;
    let swap = initial_swap(querier, &amm_settings, &config, &offer)?;
    if let Some(expected_return) = expected_return {
        if swap.result.return_amount.lt(&expected_return) {
            return Err(StdError::generic_err(
                "Operation fell short of expected_return",
            ));
        }
    }

    let mut messages = Vec::with_capacity(2);
    // Send the resulting amount of the output token
    if let Some(shade_burner) = amm_settings.shadeswap_burner {
        if swap.provider_fee_amount > Uint128::zero() {
            match &offer.token{
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } =>{
                    messages.push(snip20::transfer_msg(
                        shade_burner,
                        swap.provider_fee_amount,
                        None,
                        BLOCK_SIZE,
                        token_code_hash.clone(),
                        contract_addr.clone(),
                    )?);
                }
                TokenType::NativeToken { denom } => {
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        from_address: env.contract.address.clone(),
                        to_address: shade_burner,
                        amount: vec![Coin {
                            denom: denom.clone(),
                            amount: swap.provider_fee_amount,
                        }],
                    }));
                }
            }
        }
    }

    let index = config.pair.get_token_index(&offer.token).unwrap(); // Safe, checked in do_swap
    let token = config.pair.get_token(index ^ 1).unwrap();

    let recipient = recipient.unwrap_or(sender);
    messages.push(token.create_send_msg(
        env.contract.address,
        recipient,
        swap.result.return_amount,
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "swap_tokens"),
            log("offer_token", offer.token),
            log("offer_amount", offer.amount),
            log("return_amount", swap.result.return_amount),
            log("spread_amount", swap.result.spread_amount),   
            log("shade_swap_fee", swap.swap_fee_amount),
            log("shade_total_fee", swap.total_fee_amount),
            log("shade_provider_fee", swap.provider_fee_amount),
        ],
        data: None,
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::PairInfo => {
            let config = load_config(deps)?;

            let balances = config.pair.query_balances(
                &deps.querier,
                config.contract_addr,
                config.viewing_key.0,
            )?;
            let total_liquidity = query_liquidity(&deps.querier, &config.lp_token_info)?;

            to_binary(&QueryMsgResponse::PairInfo {
                liquidity_token: config.lp_token_info,
                factory: config.factory_info,
                pair: config.pair,
                amount_0: balances[0],
                amount_1: balances[1],
                total_liquidity,
                contract_version: AMM_PAIR_CONTRACT_VERSION,
            })
        }       
    }
}

fn calculate_fee(amount: Uint256, fee: Fee) 
-> StdResult<Uint128>
{
  let nom = Uint256::from(fee.nom);
  let denom = Uint256::from(fee.denom);
  let amount = ((amount * nom)? /denom)?;
  Ok(amount.clamp_u128()?.into())
}

fn initial_swap(
    querier: &impl Querier,
    settings: &AMMSettings<HumanAddr>,
    config: &Config<HumanAddr>,  
    offer: &TokenAmount<HumanAddr>
) -> StdResult<SwapInfo> {
    if !config.pair.contains(&offer.token) {
        return Err(StdError::generic_err(format!(
            "The required token {}, is not presented in this contract.",
            offer.token
        )));
    }
  
    let amount = Uint256::from(offer.amount);
    let swap_fee = settings.swap_fee;
    let provider_fee = settings.shadeswap_fee;
    let provider_fee_amount = calculate_fee(amount, provider_fee)?;     
    let swap_fee_amount = calculate_fee(amount,swap_fee)?;
    let total_fee_amount = provider_fee_amount + swap_fee_amount;
    let deducted_offer_amount = Uint256::from((offer.amount - total_fee_amount)?);
    let tokens_balances = config.pair.query_balances(
        querier,
        config.contract_addr.clone(),
        config.viewing_key.0.clone(),
    )?;
    let index = config.pair.get_token_index(&offer.token).unwrap();
    let token0_pool = tokens_balances[index];
    let token1_pool = tokens_balances[index ^ 1];

    // conver tand get avialble balance
    let token0_pool = Uint256::from(token0_pool);
    let token1_ppol = Uint256::from(token1_pool);
    let total_pool = (token0_pool * token1_ppol)?;

    let return_amount = (token1_ppol - (total_pool / (token0_pool + deducted_offer_amount)?)?)?;    
    let spread_amount = ((deducted_offer_amount * token1_ppol)? / token0_pool)?;
    let spread_amount = (spread_amount - return_amount).unwrap_or(Uint256::zero());

    let result_swap = SwapResult {
        return_amount: return_amount.clamp_u128()?.into(),
        spread_amount: spread_amount.clamp_u128()?.into(),
    };

    Ok(SwapInfo {
        swap_fee_amount: swap_fee_amount,
        provider_fee_amount: provider_fee_amount,
        total_fee_amount: (provider_fee_amount + swap_fee_amount),
        result: result_swap,
    })
}

fn remove_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    recipient: HumanAddr,
) -> StdResult<HandleResponse>{
    let config = load_config(&deps)?;
    let Config {
        pair,
        contract_addr,
        viewing_key,
        lp_token_info,
        ..
    } = config;

  
    let liquidity_pair_contract = query_liquidity_pair_contract(&deps.querier, &lp_token_info)?;
    let pool_balances = pair.query_balances(&deps.querier, contract_addr, viewing_key.0)?;
    
    let withdraw_amount = Uint256::from(amount);
    let total_liquidity = Uint256::from(liquidity_pair_contract);

    let mut pool_withdrawn: [Uint128; 2] = [Uint128::zero(), Uint128::zero()];

    for (i, pool_amount) in pool_balances.iter().enumerate() {
        let pool_amount = Uint256::from(*pool_amount);
        pool_withdrawn[i] = ((pool_amount * withdraw_amount)? / total_liquidity)?
            .clamp_u128()?
            .into();
    }

    let mut pair_messages: Vec<CosmosMsg> = Vec::with_capacity(3);

    for (i, token) in pair.into_iter().enumerate() {
        pair_messages.push(token.create_send_msg(
            env.contract.address.clone(),
            recipient.clone(),
            pool_withdrawn[i],
        )?);
    }

    pair_messages.push(snip20::burn_msg(
        amount,
        None,
        BLOCK_SIZE,
        lp_token_info.code_hash,
        lp_token_info.address,
    )?);

    Ok(HandleResponse {
        messages: pair_messages,
        log: vec![
            log("action", "remove_liquidity"),
            log("withdrawn_share", amount),
            log("refund_assets", format!("{}, {}", &pair.0, &pair.1)),
        ],
        data: None,
    })

}

fn add_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    deposit: TokenPairAmount<HumanAddr>,
    slippage: Option<Decimal>,
) -> StdResult<HandleResponse> {
    let config = load_config(&deps)?;

    let Config {
        pair,
        contract_addr,
        viewing_key,
        lp_token_info,
        ..
    } = config;

    if pair != deposit.pair {
        return Err(StdError::generic_err(
            "The provided tokens dont match those managed by the contract.",
        ));
    }
    let mut pair_messages: Vec<CosmosMsg> = vec![];
    let mut pool_balances = deposit.pair.query_balances(&deps.querier, contract_addr, viewing_key.0)?;
    for (i, (amount, token)) in deposit.into_iter().enumerate() {
        match &token {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                pair_messages.push(snip20::transfer_from_msg(
                    env.message.sender.clone(),
                    env.contract.address.clone(),
                    amount,
                    None,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone(),
                )?);
            }
            TokenType::NativeToken { .. } => {
                // If the asset is native token, balance is already increased.
                // To calculate properly we should subtract user deposit from the pool.
                token.assert_sent_native_token_balance(&env, amount)?;
                pool_balances[i] = (pool_balances[i] - amount)?;
            }
        }
    }

    assert_slippage_acceptance(
        slippage,
        &[deposit.amount_0, deposit.amount_1],
        &pool_balances,
    )?;

    let pair_contract_pool_liquidity = query_liquidity_pair_contract(&deps.querier, &lp_token_info)?;
    // if miniting pool first time
    let lp_tokens = if pair_contract_pool_liquidity == Uint128::zero() {      
        let deposit_token0_amount = Uint256::from(deposit.amount_0);
        let deposit_token1_amount = Uint256::from(deposit.amount_1);
        (deposit_token0_amount * deposit_token1_amount)?.sqrt()?.clamp_u128()?
    } else {
        // Total % of Pool
        let total_share = Uint256::from(pair_contract_pool_liquidity);
        // Deposit amounts of the tokens
        let deposit_token0_amount = Uint256::from(deposit.amount_0);        
        let deposit_token1_amount = Uint256::from(deposit.amount_1);
        let token0_pool = Uint256::from(pool_balances[0]);
        let token1_pool = Uint256::from(pool_balances[1]);  
        // Calcualte new % of Pool
        let percent_token0_pool = ((deposit_token0_amount * total_share)? / token0_pool)?;
        let percent_token1_pool = ((deposit_token1_amount * total_share)? / token1_pool)?;
        std::cmp::min(percent_token0_pool, percent_token1_pool).clamp_u128()?
    };

    pair_messages.push(snip20::mint_msg(
        env.message.sender,
        Uint128(lp_tokens),
        None,
        BLOCK_SIZE,
        lp_token_info.code_hash,
        lp_token_info.address,
    )?);

    Ok(HandleResponse {
        messages : pair_messages,
        log: vec![
            log("action", "add_liquidity_to_pair_contract"),
            log("assets", format!("{}, {}", deposit.pair.0, deposit.pair.1)),
            log("share_pool", lp_tokens),
        ],
        data: None,
    })
}

fn assert_slippage_acceptance(
    slippage:Option<Decimal>,
    deposits: &[Uint128; 2],
    pools: &[Uint128;2]
) -> StdResult<()> {

    if slippage.is_none() {
        return Ok(());
    }

    let slippage_amount = substraction(Decimal::one(), slippage.unwrap())?;

    if multiply(Decimal::from_ratio(deposits[0], deposits[1]), 
        slippage_amount) > Decimal::from_ratio(pools[0], pools[1])
        || multiply(Decimal::from_ratio(deposits[1], deposits[0]),
            slippage_amount,
        ) > Decimal::from_ratio(pools[1], pools[0])
    {
        return Err(StdError::generic_err(
            "Operation exceeds max slippage acceptance",
        ));
    }

    Ok(())
}


fn query_liquidity_pair_contract(
    querier: &impl Querier,
    lp_token_linke: &ContractLink<HumanAddr>,
) -> StdResult<Uint128> {
    let result = snip20::token_info_query(
        querier,
        BLOCK_SIZE,
        lp_token_linke.code_hash.clone(),
        lp_token_linke.address.clone(),
    )?;

    //If this happens, the LP token has been incorrectly configured
    if result.total_supply.is_none() {
        unreachable!("LP token has no available supply.");
    }

    Ok(result.total_supply.unwrap())
}


fn query_factory_amm_settings(
    querier: &impl Querier,
    factory: ContractLink<HumanAddr>
) -> StdResult<AMMSettings<HumanAddr>> {

    let result: FactoryQueryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        callback_code_hash: factory.code_hash,
        contract_addr: factory.address,
        msg: to_binary(&FactoryQueryMsg::GetAMMSettings)?,
    }))?;

    match result {
        FactoryQueryResponse::GetAMMSettings { settings } => Ok(settings),
        _ => Err(StdError::generic_err(
            "An error occurred while trying to retrieve factory settings.",
        )),
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

    let config = load_config(deps)?;

    match from_binary(&msg)? {
        InvokeMsg::SwapTokens {
            to,
            expected_return,
        } => {
            for token in config.pair.into_iter() {
                match token {
                    TokenType::CustomToken { contract_addr, .. } => {
                        if *contract_addr == env.message.sender {
                            let offer = TokenAmount {
                                token: token.clone(),
                                amount,
                            };

                            return swap_tokens(
                                &deps.querier,
                                env,
                                config,
                                from,
                                to,
                                offer,
                                expected_return,
                            );
                        }
                    }
                    _ => continue,
                }
            }

            Err(StdError::unauthorized())
        }
        InvokeMsg::RemoveLiquidity { recipient } => {
            if config.lp_token_info.address != env.message.sender {
                return Err(StdError::unauthorized());
            }

            remove_liquidity(deps, env, amount, recipient)
        }
    }
}
fn query_liquidity(
    querier: &impl Querier,
    lp_token_info: &ContractLink<HumanAddr>,
) -> StdResult<Uint128> {
    let result = snip20::token_info_query(
        querier,
        BLOCK_SIZE,
        lp_token_info.code_hash.clone(),
        lp_token_info.address.clone(),
    )?;

    //If this happens, the LP token has been incorrectly configured
    if result.total_supply.is_none() {
        unreachable!("LP token has no available supply.");
    }

    Ok(result.total_supply.unwrap())
}