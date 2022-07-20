use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use shadeswap_shared::msg::amm_pair::{{InitMsg,QueryMsg, SwapInfo, SwapResult, HandleMsg,TradeHistory, InvokeMsg,QueryMsgResponse}};
use shadeswap_shared::msg::factory::{QueryResponse as FactoryQueryResponse,QueryMsg as FactoryQueryMsg };
use shadeswap_shared::msg::staking::InvokeMsg as StakingInvokeMsg;
use shadeswap_shared::amm_pair::{{AMMSettings, AMMPair, Fee}};
use shadeswap_shared::token_amount::{{TokenAmount}};
use shadeswap_shared::token_pair_amount::{{TokenPairAmount}};
use shadeswap_shared::token_type::{{TokenType}};
use shadeswap_shared::token_pair::{{TokenPair}};
use shadeswap_shared::admin::{{apply_admin_guard, store_admin, load_admin, set_admin_guard}};
use shadeswap_shared::Pagination;
use crate::state::{{Config}};
use crate::state::amm_pair_storage::{store_config,store_custom_fee, load_custom_fee, is_address_in_whitelist, store_trade_counter,
     load_whitelist_address,add_whitelist_address,load_staking_contract, store_staking_contract, load_config, store_trade_history,remove_whitelist_address,
load_trade_counter, load_trade_history};
use crate::help_math::{{substraction, multiply,calculate_and_print_price}};
use crate::state::tradehistory::DirectionType;
use crate::state::PAGINATION_LIMIT;
use crate::state::CustomFee;
use shadeswap_shared::fadroma::{
    scrt::{
        from_binary, log, secret_toolkit::snip20, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg,
        Decimal, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryRequest,
        QueryResult, StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery,
    },
    scrt_callback::Callback,
    scrt_link::ContractLink,
    scrt_uint256::Uint256,
    scrt_vk::ViewingKey,
};
use shadeswap_shared::msg::staking::QueryMsg as StakingQueryMsg;
use shadeswap_shared::msg::staking::QueryResponse as StakingQueryResponse;
use shadeswap_shared::msg::staking::HandleMsg as StakingHandleMsg;
use shadeswap_shared::msg::router::HandleMsg as RouterHandleMsg;
use shadeswap_shared::msg::staking::InitMsg as StakingInitMsg;
use composable_snip20::msg::{
    InitConfig as Snip20ComposableConfig, InitMsg as Snip20ComposableMsg,
};

const AMM_PAIR_CONTRACT_VERSION: u32 = 1;
pub const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    if msg.pair.0 == msg.pair.1 {
        return Err(StdError::generic_err(
            "Creating pair.pair Contract with the same token.",
        ));
    }

    let mut messages = vec![];
    let viewing_key = create_viewing_key(&env, msg.prng_seed.clone(), msg.entropy.clone());
    register_pair_token(&env, &mut messages, &msg.pair.0, &viewing_key)?;
    register_pair_token(&env, &mut messages, &msg.pair.1, &viewing_key)?;   
    // Create LP token and store it
    messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: msg.lp_token_contract.id,
        msg: to_binary(&Snip20ComposableMsg {
            name: format!(
                "SHADESWAP Liquidity Provider (LP) token for {}-{}",
                &msg.pair.0, &msg.pair.1
            ),
            admin: Some(env.contract.address.clone()),
            symbol: "SWAP-LP".to_string(),
            decimals: 18,
            callback: Some(Callback {
                msg: to_binary(&HandleMsg::OnLpTokenInitAddr)?,
                contract: ContractLink {
                    address: env.contract.address.clone(),
                    code_hash: env.contract_code_hash.clone(),
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

    match msg.staking_contract {
        Some(c) => messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: c.contract_info.id,
            send: vec![],
            label:  format!("ShadeSwap-Pair-Staking-Contract-{}", &env.contract.address),
            callback_code_hash: c.contract_info.code_hash.clone(),
            msg: to_binary(&StakingInitMsg {
                staking_amount: c.amount,
                reward_token: c.reward_token.clone(),             
                contract: ContractLink {
                    address: env.contract.address.clone(),
                    code_hash: env.contract_code_hash.clone(),
                }
            })?
        })),
        None => println!("No staking contract"),
    }

    match msg.callback {
        Some(c) => messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: c.contract.address,
            callback_code_hash: c.contract.code_hash,
            msg: c.msg,
            send: vec![],
        })),
        None => println!("No callback given"),
    }

    let config = Config {
        factory_info: msg.factory_info.clone(),
        lp_token_info: ContractLink {
            code_hash: msg.lp_token_contract.code_hash,
            // We get the address when the instantiated LP token calls OnLpTokenInit
            address: HumanAddr::default(),
        },
        pair: msg.pair,
        contract_addr: env.contract.address.clone(),
        viewing_key: viewing_key,
    };

    store_config(deps, &config)?;       

    match msg.admin {
        Some(admin) =>  store_admin(deps, &admin)?,
        None => println!("No admin given"),
    }
   
    Ok(InitResponse {
        messages,
        log: vec![log("created_exchange_address", env.contract.address)],
    })
}

pub fn create_viewing_key(env: &Env, seed: Binary, entroy: Binary) -> ViewingKey {
    ViewingKey::new(&env, seed.as_slice(), entroy.as_slice())
}

fn register_lp_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env   
) -> StdResult<HandleResponse> {    
    let mut config = load_config(&deps)?;
    // address must be default otherwise it has been initialized.
    if config.lp_token_info.address != HumanAddr::default() {
        return Err(StdError::unauthorized());
    }

    config.lp_token_info.address = env.message.sender.clone();
    // store config against Smart contract address
    store_config(deps, &config)?;

    let mut messages = Vec::new();
    // register pair contract for LP receiver
    messages.push(snip20::register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        BLOCK_SIZE,
        config.lp_token_info.code_hash.clone(),
        env.message.sender.clone(),
    )?);  

    Ok(HandleResponse {
        messages: messages,
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
        HandleMsg::Receive {
            from, amount, msg, ..
        } => receiver_callback(deps, env, from, amount, msg),
        HandleMsg::AddLiquidityToAMMContract { deposit, slippage, staking } => {
            add_liquidity(deps, env, deposit, slippage, staking)
        },
        HandleMsg::SetCustomPairFee {shade_dao_fee, lp_fee} => set_custom_fee(deps, env, shade_dao_fee, lp_fee),
        HandleMsg::SetStakingContract{contract} => set_staking_contract(deps, env, contract),
        HandleMsg::SetAMMPairAdmin {admin} => set_admin_guard(deps,env,admin),
        HandleMsg::OnLpTokenInitAddr => register_lp_token(deps, env),
        HandleMsg::AddWhiteListAddress{address} => add_address_to_whitelist(&mut deps.storage, address, env),
        HandleMsg::RemoveWhitelistAddresses{addresses} => remove_address_from_whitelist(&mut deps.storage, addresses, env),
        HandleMsg::SwapTokens {
            offer,
            expected_return,
            to,
            router_link,
            callback_signature,
        } => {
            // this is assert if token is SCRT if not then swapp will be called via SNIP20 Interface
            if !offer.token.is_native_token() {
                return Err(StdError::unauthorized());
            }

            offer.assert_sent_native_token_balance(&env)?;
            let config_settings = load_config(deps)?;
            let sender = env.message.sender.clone();
            swap(
                deps,
                env,
                config_settings,
                sender,
                to,
                offer,
                expected_return,
                router_link,
                callback_signature,
            )
        }
    }
}

pub fn query_calculate_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    offer: TokenAmount<HumanAddr>,
    feeless: Option<bool>
) -> StdResult<SwapInfo>{
    let config_settings = load_config(deps)?;
    let amm_settings = query_factory_amm_settings(&deps.querier, config_settings.factory_info.clone())?;
    let swap_result = calculate_swap_result(&deps.querier, &amm_settings, &config_settings,&offer,  &deps.storage, HumanAddr::default(), feeless)?;
    Ok(swap_result)
}


pub fn set_custom_fee<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    shade_dao_fee: Fee,
    lp_fee: Fee
) -> StdResult<HandleResponse> {
    apply_admin_guard(env.message.sender.clone(), &deps.storage)?;
    store_custom_fee(deps, &CustomFee{ 
        shade_dao_fee: shade_dao_fee.clone(),
        lp_fee: lp_fee.clone(),
        configured: true
    })?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "set_custom_fee"),
        ],
        data: None,
    })
}

pub fn swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config<HumanAddr>,
    sender: HumanAddr,
    recipient: Option<HumanAddr>,
    offer: TokenAmount<HumanAddr>,
    expected_return: Option<Uint128>,
    router_link: Option<ContractLink<HumanAddr>>,
    callback_signature: Option<Binary>,
) -> StdResult<HandleResponse> {
    let swaper_receiver = recipient.unwrap_or(sender);
    let amm_settings = query_factory_amm_settings(&deps.querier,config.factory_info.clone())?;
    let swap_result = calculate_swap_result(&deps.querier, &amm_settings, &config, &offer,&mut deps.storage, swaper_receiver.clone(), None)?;

    // check for the slippage expected value compare to actual value
    if let Some(expected_return) = expected_return {
        if swap_result.result.return_amount.lt(&expected_return) {
            return Err(StdError::generic_err(
                "Operation fell short of expected_return",
            ));
        }
    }

    // // Send Shade_Dao_Fee back to shade_dao_address which is 0.1%
    let mut messages = Vec::with_capacity(3);
    if swap_result.shade_dao_fee_amount > Uint128::zero() {
        match &offer.token {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                messages.push(snip20::transfer_msg(
                    amm_settings.shade_dao_address.address,
                    swap_result.shade_dao_fee_amount,
                    None,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone(),
                )?);
            }
            TokenType::NativeToken { denom } => {
                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: amm_settings.shade_dao_address.address.clone(),
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount: swap_result.shade_dao_fee_amount,
                    }],
                }));
            }
        }
    }

    // Send Token to Buyer or Swapper
    let index = config.pair.get_token_index(&offer.token).unwrap(); // Safe, checked in do_swap
    let token = config.pair.get_token(index ^ 1).unwrap();
    messages.push(token.create_send_msg(
        env.contract.address,
        swaper_receiver.clone(),
        swap_result.result.return_amount,
    )?);
    let mut action = "".to_string();
    if index == 0 {
        action = "BUY".to_string();
    } 
    if index == 1 {
        action = "SELL".to_string();
    }      
    
    // Push Trade History
    let mut hasher = DefaultHasher::new();
    swaper_receiver.hash(&mut hasher);
    let hash_address = hasher.finish();
    let trade_history =  TradeHistory
    {
        price: swap_result.price,
        amount_in: swap_result.result.return_amount,
        amount_out: offer.amount,
        timestamp: env.block.time,
        height: env.block.height,
        direction: action.to_string(),
        lp_fee_amount: swap_result.lp_fee_amount,
        total_fee_amount: swap_result.total_fee_amount,
        shade_dao_fee_amount: swap_result.shade_dao_fee_amount,
        trader: hash_address.to_string(),
    };

    store_trade_history(deps, &trade_history)?;

    if !router_link.is_none() {
        // push message back to router
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: router_link.clone().unwrap().address,
            callback_code_hash: router_link.clone().unwrap().code_hash,
            send: vec![],
            msg: to_binary(&RouterHandleMsg::SwapCallBack {
                last_token_out: TokenAmount {
                    token: token.clone(),
                    amount: swap_result.result.return_amount,
                },
                signature: callback_signature.unwrap(),
            })?,
        }));
    }

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "swap"),
            log("offer_token", offer.token),
            log("offer_amount", offer.amount),
            log("return_amount", swap_result.result.return_amount),
            log("lp_fee", swap_result.lp_fee_amount),
            log("shade_dao_fee", swap_result.shade_dao_fee_amount),
            log("shade_total_fee", swap_result.total_fee_amount),
        ],
        data: None,
    })
}

pub fn set_staking_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>, 
    env: Env,
    contract: ContractLink<HumanAddr>
)-> StdResult<HandleResponse>{      
    // only callback can call this method
    let contract_info = load_staking_contract(&deps)?;    
    if contract_info.address != HumanAddr::default(){
        return Err(StdError::unauthorized())
    }
    store_staking_contract(deps, &contract.clone())?;
    let config = load_config(deps)?;

    let mut messages = Vec::new();  
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract.address.clone(),
        callback_code_hash: contract.code_hash.clone(),
        send: vec![],
        msg: to_binary(&StakingHandleMsg::SetLPToken {
           lp_token: config.lp_token_info.clone()
        })?,
    }));

    // send lp contractLink to staking contract 
    Ok(HandleResponse {
        messages: messages,
        log: vec![
            log("action", "set_staking_contract"),
            log("contract_address", contract.address.clone()),
            log("contract_hash", contract.code_hash.to_string().clone()),
        ],
        data: None,
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::GetPairInfo => {
            let config = load_config(deps)?;
            let balances = config.pair.query_balances(
                &deps.querier,
                config.contract_addr,
                config.viewing_key.0,
            )?;
            let total_liquidity = query_liquidity(&deps.querier, &config.lp_token_info)?;
            to_binary(&QueryMsgResponse::GetPairInfo {
                liquidity_token: config.lp_token_info,
                factory: config.factory_info,
                pair: config.pair,
                amount_0: balances[0],
                amount_1: balances[1],
                total_liquidity,
                contract_version: AMM_PAIR_CONTRACT_VERSION,
            })
        }       
        QueryMsg::GetTradeHistory { pagination } => {
            let data = load_trade_history_query(&deps, pagination)?;
            to_binary(&QueryMsgResponse::GetTradeHistory { data })
        },
        QueryMsg::GetAdmin{} =>{
            let admin_address = load_admin(&deps.storage)?;
            to_binary(&QueryMsgResponse::GetAdminAddress{
                address: admin_address
            })
        },
        QueryMsg::GetWhiteListAddress => {
            let stored_addr = load_whitelist_address(&deps.storage)?;
            to_binary(&QueryMsgResponse::GetWhiteListAddress {
                addresses: stored_addr,
            })
        }
        QueryMsg::GetTradeCount => {
            let count = load_trade_counter(&deps.storage)?;
            to_binary(&QueryMsgResponse::GetTradeCount { count })
        },
        QueryMsg::GetStakingContract => {
            let staking_contract = load_staking_contract(&deps)?;
            to_binary(&QueryMsgResponse::StakingContractInfo{
                staking_contract: staking_contract
            })
        },         
        QueryMsg::GetEstimatedPrice {offer, feeless} => {
           let swap_result = query_calculate_price(&deps,offer, feeless)?;
           to_binary(&QueryMsgResponse::EstimatedPrice { estimated_price : swap_result.price })
        },
        QueryMsg::SwapSimulation{ offer}=> swap_simulation(deps, offer),
        QueryMsg::GetShadeDaoInfo{} => get_shade_dao_info(deps),
        QueryMsg::GetEstimatedLiquidity { deposit, slippage} => get_estimated_lp_token(deps, deposit, slippage),
    }
}

fn get_shade_dao_info<S:Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Binary>{
    let config_settings = load_config(deps)?;
    let admin = load_admin(&deps.storage)?;
    let amm_settings = query_factory_amm_settings(&deps.querier, config_settings.factory_info.clone())?;
    let shade_dao_info = QueryMsgResponse::ShadeDAOInfo{
        shade_dao_address: amm_settings.shade_dao_address.address.clone(),
        shade_dao_fee: amm_settings.shade_dao_fee.clone(),
        admin_address: admin.clone(),
        lp_fee: amm_settings.lp_fee.clone(),
    };
    to_binary(&shade_dao_info)
}

fn swap_simulation<S:Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,  
    offer: TokenAmount<HumanAddr>
) -> StdResult<Binary>{
    let config_settings = load_config(deps)?;
    let amm_settings = query_factory_amm_settings(&deps.querier, config_settings.factory_info.clone())?;
    let swap_result = calculate_swap_result(&deps.querier, &amm_settings, &config_settings,&offer,  &deps.storage, HumanAddr::default(), None)?;
    let simulation_result = QueryMsgResponse::SwapSimulation{
        total_fee_amount: swap_result.total_fee_amount,
        lp_fee_amount: swap_result.lp_fee_amount,
        shade_dao_fee_amount: swap_result.shade_dao_fee_amount,
        result: swap_result.result,
        price: swap_result.price,
    };
    to_binary(&simulation_result)
}

fn get_estimated_lp_token<S:Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    deposit: TokenPairAmount<HumanAddr>,
    slippage: Option<Decimal>
) -> StdResult<Binary>{
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

    let mut pool_balances =
        deposit
            .pair
            .query_balances(&deps.querier, contract_addr, viewing_key.0)?;

    assert_slippage_acceptance(
        slippage,
        &[deposit.amount_0, deposit.amount_1],
        &pool_balances,
    )?;

    let pair_contract_pool_liquidity =
        query_liquidity_pair_contract(&deps.querier, &lp_token_info)?;
    let mut lp_tokens: u128 = u128::MIN;
    if pair_contract_pool_liquidity == Uint128::zero() {
        // If user mints new liquidity pool -> liquidity % = sqrt(x * y) where
        // x and y is amount of token0 and token1 provided
        let deposit_token0_amount = Uint256::from(deposit.amount_0);
        let deposit_token1_amount = Uint256::from(deposit.amount_1);
        lp_tokens = (deposit_token0_amount * deposit_token1_amount)?
            .sqrt()?
            .clamp_u128()?
    } else {
        // Total % of Pool
        let total_share = Uint256::from(pair_contract_pool_liquidity);
        // Deposit amounts of the tokens
        let deposit_token0_amount = Uint256::from(deposit.amount_0);
        let deposit_token1_amount = Uint256::from(deposit.amount_1);

        // get token pair balance
        let token0_pool = Uint256::from(pool_balances[0]);
        let token1_pool = Uint256::from(pool_balances[1]);
        // Calcualte new % of Pool
        let percent_token0_pool = ((deposit_token0_amount * total_share)? / token0_pool)?;
        let percent_token1_pool = ((deposit_token1_amount * total_share)? / token1_pool)?;
        lp_tokens = std::cmp::min(percent_token0_pool, percent_token1_pool).clamp_u128()?
    };

    let response_msg = QueryMsgResponse::EstimatedLiquidity { lp_token: Uint128(lp_tokens), total_lp_token: pair_contract_pool_liquidity };
    to_binary(&response_msg)

}

fn load_trade_history_query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination,
) -> StdResult<Vec<TradeHistory>> {
    let count = load_trade_counter(&deps.storage)?;

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

fn calculate_fee(amount: Uint256, fee: Fee) -> StdResult<Uint128> {
    let nom = Uint256::from(fee.nom);
    let denom = Uint256::from(fee.denom);
    let amount = ((amount * nom)? / denom)?;   
    Ok(amount.clamp_u128()?.into())
}

pub fn calculate_swap_result(
    querier: &impl Querier,
    settings: &AMMSettings<HumanAddr>,
    config: &Config<HumanAddr>,
    offer: &TokenAmount<HumanAddr>,
    storage: &impl Storage,
    recipient: HumanAddr,
    feeless: Option<bool>
) -> StdResult<SwapInfo> {
    if !config.pair.contains(&offer.token) {
        return Err(StdError::generic_err(format!(
            "The required token {}, is not presented in this contract.",
            offer.token
        )));
    }

    let amount = Uint256::from(offer.amount);
    // conver tand get avialble balance
    let tokens_pool = get_token_pool_balance(querier, config, offer)?;
    let token0_pool = tokens_pool[0];
    let token1_pool = tokens_pool[1];
    // calculate price   

    // calculate fee
    let lp_fee = settings.lp_fee;
    let shade_dao_fee = settings.shade_dao_fee;
    let mut lp_fee_amount = Uint128(0u128);
    let mut shade_dao_fee_amount = Uint128(0u128);
    // calculation fee
    let discount_fee = is_address_in_whitelist(storage, recipient)?;
    if discount_fee == false {        
        let custom_fee: CustomFee = load_custom_fee(storage)?;
        if  custom_fee.configured == true  {
            lp_fee_amount = calculate_fee(Uint256::from(offer.amount), custom_fee.lp_fee)?;
            shade_dao_fee_amount = calculate_fee(Uint256::from(offer.amount), custom_fee.shade_dao_fee)?;
        }
        else{
            lp_fee_amount = calculate_fee(Uint256::from(offer.amount), lp_fee)?;
            shade_dao_fee_amount = calculate_fee(Uint256::from(offer.amount), shade_dao_fee)?;
        }        
        lp_fee_amount = calculate_fee(Uint256::from(offer.amount), lp_fee)?;
        shade_dao_fee_amount = calculate_fee(Uint256::from(offer.amount), shade_dao_fee)?;
    }
    // total fee
    let total_fee_amount = lp_fee_amount + shade_dao_fee_amount;

    // sub fee from offer amount  
    let mut deducted_offer_amount = (offer.amount - total_fee_amount)?; 
    if let Some(true) = feeless {
        deducted_offer_amount = offer.amount;
    }

    let swap_amount = calculate_price(Uint256::from(deducted_offer_amount), token0_pool, token1_pool)?;
    let result_swap = SwapResult {
        return_amount: swap_amount.clamp_u128()?.into(),       
    };

    let token_index = config.pair.get_token_index(&offer.token).unwrap();  

    Ok(SwapInfo {
        lp_fee_amount: lp_fee_amount,
        shade_dao_fee_amount: shade_dao_fee_amount,
        total_fee_amount: total_fee_amount,
        result: result_swap,
        price: calculate_and_print_price(swap_amount.clamp_u128()?.into(), amount.clamp_u128()?.into(), token_index)?,
    })
}

pub fn add_address_to_whitelist(storage: &mut impl Storage, address: HumanAddr, env :Env) -> StdResult<HandleResponse>{
    apply_admin_guard(env.message.sender.clone(), storage)?;
    add_whitelist_address(storage, address.clone())?;  
    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "save_address_to_whitelist"),
            log("whitelist_address", address.as_str().clone()),
        ],
        data: None,
    })
}

pub fn remove_address_from_whitelist(storage: &mut impl Storage, list: Vec<HumanAddr>,
    env :Env) -> StdResult<HandleResponse>{
    apply_admin_guard(env.message.sender.clone(), storage)?;
    remove_whitelist_address(storage, list.clone())?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("action", "remove_address_from_whitelist")],
        data: None,
    })
}

fn get_token_pool_balance(
    querier: &impl Querier,  
    config: &Config<HumanAddr>,
    swap_offer: &TokenAmount<HumanAddr>,
) -> StdResult<[Uint256; 2]> {
    let tokens_balances = config.pair.query_balances(
        querier,
        config.contract_addr.clone(),
        config.viewing_key.0.clone(),
    )?;
    let index = config.pair.get_token_index(&swap_offer.token).unwrap();
    let token0_pool = tokens_balances[index];
    let token1_pool = tokens_balances[index ^ 1];

    // conver tand get avialble balance
    let token0_pool = Uint256::from(token0_pool);
    let token1_pool = Uint256::from(token1_pool);
    Ok([token0_pool, token1_pool])
}

fn remove_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    from: HumanAddr,
) -> StdResult<HandleResponse> {    
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

    let mut pair_messages: Vec<CosmosMsg> = Vec::with_capacity(4);

    for (i, token) in pair.into_iter().enumerate() {
        pair_messages.push(token.create_send_msg(
            env.contract.address.clone(),
            from.clone(),
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

pub fn calculate_price(
    amount: Uint256,
    token0_pool_balance: Uint256,
    token1_pool_balance: Uint256,
) -> StdResult<Uint256> {
    Ok(((token1_pool_balance * amount)? / (token0_pool_balance + amount)?)?)
}

fn add_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    deposit: TokenPairAmount<HumanAddr>,
    slippage: Option<Decimal>,
    staking: Option<bool>
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

    // let staking_contract = load_staking_contract(&deps)?;
    let mut pair_messages: Vec<CosmosMsg> = vec![];
    let mut pool_balances =
        deposit
            .pair
            .query_balances(&deps.querier, contract_addr, viewing_key.0)?;
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

    let pair_contract_pool_liquidity =
        query_liquidity_pair_contract(&deps.querier, &lp_token_info)?;
    let mut lp_tokens: u128 = u128::MIN;
    if pair_contract_pool_liquidity == Uint128::zero() {
        // If user mints new liquidity pool -> liquidity % = sqrt(x * y) where
        // x and y is amount of token0 and token1 provided
        let deposit_token0_amount = Uint256::from(deposit.amount_0);
        let deposit_token1_amount = Uint256::from(deposit.amount_1);
        lp_tokens = (deposit_token0_amount * deposit_token1_amount)?
            .sqrt()?
            .clamp_u128()?
    } else {
        // Total % of Pool
        let total_share = Uint256::from(pair_contract_pool_liquidity);
        // Deposit amounts of the tokens
        let deposit_token0_amount = Uint256::from(deposit.amount_0);
        let deposit_token1_amount = Uint256::from(deposit.amount_1);

        // get token pair balance
        let token0_pool = Uint256::from(pool_balances[0]);
        let token1_pool = Uint256::from(pool_balances[1]);
        // Calcualte new % of Pool
        let percent_token0_pool = ((deposit_token0_amount * total_share)? / token0_pool)?;
        let percent_token1_pool = ((deposit_token1_amount * total_share)? / token1_pool)?;
        lp_tokens = std::cmp::min(percent_token0_pool, percent_token1_pool).clamp_u128()?
    };

    let mut add_to_staking = false;
    // check if user wants add his LP token to Staking
    if let Some(true) = staking {
        // check if the Staking Contract has been set for AMM Pairs
        add_to_staking = true;
        let staking_contract = load_staking_contract(deps)?;
        if staking_contract.address == HumanAddr::default() {
            return Err(StdError::generic_err(
                "Staking Contract has not been set for AMM Pairs",
            ));          
        } 
           
        pair_messages.push(snip20::mint_msg(
            env.contract.address.clone(),
            Uint128(lp_tokens),
            None,
            BLOCK_SIZE,
            lp_token_info.code_hash.clone(),
            lp_token_info.address.clone(),
        )?);
      
        let invoke_msg = to_binary(&StakingInvokeMsg::Stake {
            from: env.message.sender.clone(),  
            amount: Uint128(lp_tokens)           
        })
        .unwrap();

        let receive_msg = to_binary(&StakingHandleMsg::Receive { 
            from:  env.message.sender.clone(), msg: Some(invoke_msg.clone()), amount: Uint128(lp_tokens) })?;
        // SEND LP Token to Staking Contract with Staking Message
        let msg = to_binary(&snip20::HandleMsg::Send {
            recipient: staking_contract.address.clone(),
            amount: Uint128(lp_tokens),
            msg: Some(invoke_msg.clone()),
            padding: None,
        })?;
        
        pair_messages.push(
            WasmMsg::Execute {
                contract_addr:  lp_token_info.address.clone(),
                callback_code_hash: lp_token_info.code_hash.clone(),
                msg,
                send: vec![],
                }
            .into(),
        );
    }
    else {
        add_to_staking = false;
        pair_messages.push(snip20::mint_msg(
            env.message.sender.clone(),
            Uint128(lp_tokens),
            None,
            BLOCK_SIZE,
            lp_token_info.code_hash.clone(),
            lp_token_info.address.clone(),
        )?);
    }  
   

    Ok(HandleResponse {
        messages: pair_messages,
        log: vec![
            log("staking", format!("{}", add_to_staking)),
            log("action", "add_liquidity_to_pair_contract"),
            log("assets", format!("{}, {}", deposit.pair.0, deposit.pair.1)),
            log("share_pool", lp_tokens),
        ],
        data: None,
    })
}


fn assert_slippage_acceptance(
    slippage: Option<Decimal>,
    deposits: &[Uint128; 2],
    pools: &[Uint128; 2],
) -> StdResult<()> {
    if slippage.is_none() {
        return Ok(());
    }

    let slippage_amount = substraction(Decimal::one(), slippage.unwrap())?;

    if multiply(
        Decimal::from_ratio(deposits[0], deposits[1]),
        slippage_amount,
    ) > Decimal::from_ratio(pools[0], pools[1])
        || multiply(
            Decimal::from_ratio(deposits[1], deposits[0]),
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
    factory: ContractLink<HumanAddr>,
) -> StdResult<AMMSettings<HumanAddr>> {
    let result: FactoryQueryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        callback_code_hash: factory.code_hash,
        contract_addr: factory.address,
        msg: to_binary(&FactoryQueryMsg::GetAMMSettings {})?,
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
    let from_caller = from.clone();
    match from_binary(&msg)? {
        InvokeMsg::SwapTokens {
            to,
            expected_return,
            router_link,
            callback_signature,
        } => {
            for token in config.pair.into_iter() {
                match token {
                    TokenType::CustomToken { contract_addr, .. } => {
                        if *contract_addr == env.message.sender {
                            let offer = TokenAmount {
                                token: token.clone(),
                                amount,
                            };

                            return swap(
                                deps,
                                env,
                                config,
                                from,
                                to,
                                offer,
                                expected_return,
                                router_link,
                                callback_signature,
                            );
                        }
                    }
                    _ => continue,
                }
            }

            Err(StdError::unauthorized())
        }
        InvokeMsg::RemoveLiquidity { from } => {
            if config.lp_token_info.address != env.message.sender {
                return Err(StdError::unauthorized());
            }
            match from {
                Some(address) =>  remove_liquidity(deps, env, amount, address),
                None =>  remove_liquidity(deps, env, amount, from_caller),
            }                 
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
