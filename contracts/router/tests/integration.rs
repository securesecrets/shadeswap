use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use shadeswap_shared::msg::router::{InitMsg, ExecuteMsg, QueryMsg};
use cosmwasm_std::{
    to_binary, Addr, Empty,
};

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn router_integration_tests_with_snip20_token() {    
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::amm_pairs::amm_pairs_lib::amm_pairs_lib::{init_amm_pair, create_amm_settings, create_amm_pairs, create_custom_token, amm_pair_contract_store_in, add_liquidity_to_amm_pairs, get_amm_pair_info_query_liquidity};
    use multi_test::staking::staking_lib::staking_lib::{staking_contract_store_in, create_staking_info_contract};
    use router::contract::{instantiate, query, execute, reply};
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, mint_deposit_snip20, store_init_factory_contract, 
        convert_to_contract_link, snip20_lp_token_contract_store, create_token_pair, increase_allowance, snip_20_balance_query, create_token_pair_with_native, set_viewing_key, get_snip20_balance};
    use cosmwasm_std::{Uint128, Coin, ContractInfo, BlockInfo, StdError};
    use multi_test::util_addr::util_addr::{OWNER, STAKER_A};     
    use shadeswap_shared::core::{TokenAmount, ContractInstantiationInfo};
    use shadeswap_shared::msg::amm_pair::{InvokeMsg};
    use shadeswap_shared::msg::router::QueryMsgResponse;
    use shadeswap_shared::router::Hop;
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract};
    use multi_test::factory::factory_lib::factory_lib::{init_factory, create_amm_pairs_to_factory, list_amm_pairs_from_factory};
    use cosmwasm_std::Timestamp;
    
    pub fn router_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }
    
    let shade_dao_fee_address = Addr::unchecked("secret15yh35cflwdz6nn6zszvm90mf36a69uhm6rq6th");
    let staker_a_addr = Addr::unchecked(STAKER_A.to_owned());     
    let owner_addr = Addr::unchecked(OWNER);   
    let mut router = App::default();

    router.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(1 as u64),
        chain_id: "chain_id".to_string(),
    }); 

    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_addr.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });

    // GENERATE TOKEN PAIRS & REWARD TOKEN
    let token_0_contract = generate_snip20_contract(&mut router, "ETH".to_string(),"ETH".to_string(),18).unwrap();    
    let token_1_contract = generate_snip20_contract(&mut router, "USDT".to_string(),"USDT".to_string(),18).unwrap();    
    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
   
    // MINT AND DEPOSIT FOR LIQUIDITY
    mint_deposit_snip20(&mut router,&token_0_contract,&owner_addr,Uint128::new(10000000000u128), &owner_addr);
    mint_deposit_snip20(&mut router,&token_1_contract,&owner_addr,Uint128::new(10000000000u128), &owner_addr);   
    mint_deposit_snip20(&mut router,&reward_contract,&owner_addr,Uint128::new(10000000000u128), &owner_addr);       
    
    roll_blockchain(&mut router, 1).unwrap();
    
   // INIT LP, STAKING, AMM PAIRS
    let admin_contract = init_admin_contract(&mut router, &owner_addr).unwrap(); //store_init_factory_contract(&mut router, &convert_to_contract_link(&admin_contract)).unwrap();
    let amm_contract_info = router.store_code(amm_pair_contract_store_in());
    let lp_token_info = router.store_code(snip20_lp_token_contract_store());
    let staking_info = router.store_code(staking_contract_store_in());
    
     // STORE ROUTER CONTRACT
     let router_contract_info = router.store_code(router_contract_store());
     roll_blockchain(&mut router, 1).unwrap();

    // INIT ROUTER CONTRACTs
    let init_msg = InitMsg {
        prng_seed: to_binary("password").unwrap(),
        entropy: to_binary("password").unwrap(),       
        admin_auth: convert_to_contract_link(&admin_contract),
    };    

    roll_blockchain(&mut router, 1).unwrap();
    let router_contract = router
        .instantiate_contract(
            router_contract_info,
            owner_addr.to_owned(),
            &init_msg,
            &[],
            "router",
            Some(OWNER.to_string()),
        ).unwrap();    

    // CREATE FACTORY
    roll_blockchain(&mut router, 1).unwrap(); 
    let factory_contract = init_factory(
        &mut router,
        &convert_to_contract_link(&admin_contract),
        &OWNER,
        false,
        create_amm_settings(
            3,
            100,
            8,
            100, 
            &shade_dao_fee_address
        ),
        ContractInstantiationInfo{
            code_hash: amm_contract_info.code_hash.clone(),
            id: amm_contract_info.code_id,
        },        
        ContractInstantiationInfo{
            code_hash: lp_token_info.code_hash.clone(),
            id: lp_token_info.code_id,
        },
        "seed",
        "api_key",
        None
    ).unwrap();

    // CREATE AMM_PAIR SNIP20 vs SNIP20
    create_amm_pairs_to_factory(
        &mut router,
        &factory_contract,
        &create_token_pair(
            &convert_to_contract_link(&token_0_contract), 
            &convert_to_contract_link(&token_1_contract)
        ),
        "seed",
        &create_staking_info_contract(
            staking_info.code_id, 
            &staking_info.code_hash, 
            Uint128::new(30000u128), 
            TokenType::CustomToken { 
                contract_addr: reward_contract.address.clone(), 
                token_code_hash: reward_contract.code_hash.clone() },
            Uint128::new(30000000000u128)
        ),
        &router_contract, 
        &owner_addr).unwrap();
       
    // LIST AMM PAIR
    let amm_pairs = list_amm_pairs_from_factory(
        &mut router,
        &factory_contract,
        0, 30
    ).unwrap();
    
    // ASSERT AMM PAIRS == 1
    assert_eq!(amm_pairs.len(), 1);

    // INCREASE ALLOWANCE FOR AMM PAIR
    increase_allowance(&mut router, &token_0_contract, Uint128::new(10000000000000000u128),&amm_pairs[0].address , &owner_addr).unwrap();
    increase_allowance(&mut router, &token_1_contract, Uint128::new(10000000000000000u128),&amm_pairs[0].address , &owner_addr).unwrap();

    // ADD LIQUIDITY TO AMM_PAIR SNIP20 vs SNIP20
    add_liquidity_to_amm_pairs(
        &mut router,
        &ContractInfo{
            address: amm_pairs[0].address.clone(),
            code_hash: "".to_string(),
        },
        &amm_pairs[0].pair,
        Uint128::new(1000000000u128),
        Uint128::new(1000000000u128),
        Some(Uint128::new(1000000000u128)),
        Some(true),
        Vec::new(),
        &owner_addr
    ).unwrap();

    // ASSERT LIQUIDITY BALANCE
    let balance = get_amm_pair_info_query_liquidity(
        &mut router, 
        &ContractInfo{ 
            address: amm_pairs[0].address.to_owned(),
            code_hash: amm_contract_info.code_hash.clone()
        }
    ).unwrap();
    assert_eq!(balance, Uint128::new(1000000u128));

    // REGISTER SNIP 20 ROUTER
    roll_blockchain(&mut router, 1).unwrap(); 
    let msg = ExecuteMsg::RegisterSNIP20Token { 
        token_addr: token_0_contract.address.to_string() , 
        token_code_hash: token_0_contract.code_hash.to_owned() 
    };
    
    let _ = router.execute_contract(
        owner_addr.to_owned(), 
        &router_contract, 
        &msg, 
        &[]).unwrap();

    roll_blockchain(&mut router, 1).unwrap();      
   
    // ASSERT SWAPSIMULATION SNIP20 -> SNIP20
    let offer = TokenAmount{
        token: TokenType::CustomToken { 
            contract_addr: token_0_contract.address.to_owned(), 
            token_code_hash: token_0_contract.code_hash.to_owned() 
        },
        amount: Uint128::new(1000u128)
    };
    let swap_query = QueryMsg::SwapSimulation { 
        offer: offer.to_owned(),
        path: vec![Hop{addr: amm_pairs[0].address.to_owned(), code_hash: amm_contract_info.code_hash.clone()}] 
    };    
    let query_response:QueryMsgResponse = router.query_test(
        router_contract.to_owned(),
        to_binary(&swap_query).unwrap()
    ).unwrap();

    match query_response {
        QueryMsgResponse::SwapSimulation {
            total_fee_amount,
            lp_fee_amount,
            shade_dao_fee_amount,
            result,
            price ,
        } => {
        // Verify result not actual amount
           assert_ne!(total_fee_amount, Uint128::zero());
           assert_ne!(lp_fee_amount, Uint128::zero());
           assert_ne!(shade_dao_fee_amount, Uint128::zero());
           assert_ne!(result.return_amount, Uint128::zero());
           assert_eq!(price, "1".to_string());
        },
        _ => panic!("Query Responsedoes not match")
    }

    // ASSERT SWAPTOKENS 
    roll_blockchain(&mut router, 1).unwrap(); 
    let invoke_msg = to_binary(&InvokeMsg::SwapTokens { 
        expected_return: Some(Uint128::new(100u128)), 
        to: Some(staker_a_addr.to_owned()), 
    }).unwrap();
   
    let msg = snip20_reference_impl::msg::ExecuteMsg::Send {
        recipient: amm_pairs[0].address.to_owned(),
        recipient_code_hash: Some(amm_contract_info.code_hash.clone()),
        amount: Uint128::new(1000u128),
        msg: Some(invoke_msg),
        memo: None,
        padding: None,
    };

    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &token_0_contract,
        &msg,
        &[], // 
    )
    .unwrap();                

    // ASSERT SWAPTOKENSFOREXACT THROW ERR WITH CUSTOM TOKEN
    roll_blockchain(&mut router, 1).unwrap();  
    let execute_swap = ExecuteMsg::SwapTokensForExact { 
        offer:offer.to_owned(),
        expected_return: Some(Uint128::new(900u128)), 
        path: vec![Hop{addr: amm_pairs[0].address.to_owned(), code_hash: amm_contract_info.code_hash.clone()}],
        recipient: Some(owner_addr.to_string())
    };

    let response = router.execute_contract(
        owner_addr.to_owned(), 
        &router_contract, 
        &execute_swap,  
        &[Coin{denom: "uscrt".to_string(), amount: Uint128::new(1000u128)}]).is_err();
        
    assert_eq!(response, true);

    // ASSERT BALANCE TOKEN_1
    let balance = snip_20_balance_query(
        &mut router,
        &owner_addr,
        "seed",
        &token_1_contract
    ).unwrap();

    assert_eq!(balance, Uint128::new(1000019000000000u128));

    // CREATE AMM_PAIR NATIVE - SNIP20
    roll_blockchain(&mut router, 1).unwrap();
    create_amm_pairs_to_factory(
        &mut router,
        &factory_contract,
        &create_token_pair_with_native(
            &convert_to_contract_link(&token_0_contract), 
        ),
        "seed",
        &create_staking_info_contract(
            staking_info.code_id, 
            &staking_info.code_hash, 
            Uint128::new(30000u128), 
            TokenType::CustomToken { 
                contract_addr: reward_contract.address.clone(), 
                token_code_hash: reward_contract.code_hash.clone() },
            Uint128::new(30000000000u128)
        ),
        &router_contract, 
        &owner_addr).unwrap();
       
      // LIST AMM PAIR
      let amm_pairs = list_amm_pairs_from_factory(
        &mut router,
        &factory_contract,
        0, 30
    ).unwrap();
    
    // ASSERT AMM PAIRS == 2
    assert_eq!(amm_pairs.len(), 2);

    // ADD LIQUIDITY TO AMM_PAIR NATIVE vs SNIP20
    increase_allowance(&mut router, &token_0_contract, Uint128::new(10000000000000000u128),&amm_pairs[1].address , &owner_addr).unwrap();
       
    let mut funds: Vec<Coin> = Vec::new();
    funds.push(Coin { denom: "uscrt".to_string(), amount: Uint128::new(10000u128)});

    add_liquidity_to_amm_pairs(
        &mut router,
        &ContractInfo{
            address: amm_pairs[1].address.clone(),
            code_hash: "".to_string(),
        },
        &amm_pairs[1].pair,
        Uint128::new(10000u128),
        Uint128::new(10000u128),
        Some(Uint128::new(10000u128)),
        Some(true),
        funds,
        &owner_addr
    ).unwrap();
    
    // ASSERT SWAPTOKENEXACT NATIVE EXCEPTION FOR SNIP 20 TOKEN
    let offer = TokenAmount{
        token: TokenType::NativeToken { denom: "uscrt".to_string()},
        amount: Uint128::new(2000u128)
    };

    let execute_swap = ExecuteMsg::SwapTokensForExact { 
        offer:offer.to_owned(),
        expected_return: Some(Uint128::new(1000u128)), 
        path: vec![Hop{addr: amm_pairs[1].address.to_owned(), code_hash: amm_contract_info.code_hash.clone()}],
        recipient: Some(owner_addr.to_string())
    };

    let _response = router.execute_contract(
        owner_addr.to_owned(), 
        &router_contract, 
        &execute_swap,  
        &[
            Coin{ 
                denom: "uscrt".to_string(), 
                amount: Uint128::new(2000u128)
            }
        ]).unwrap(); 
        
    // ASSERT STAKER SNIP20 BALANCE
    let _ = set_viewing_key(
        &mut router,
        &token_0_contract,
        "seed",
        &staker_a_addr).unwrap();

    let snip20_balance = get_snip20_balance(
        &mut router,
        &token_0_contract,
        STAKER_A,
        "seed");   

    // ASSERT BALANCE TOKEN 0 - STAKER A
    assert_eq!(snip20_balance, Uint128::new(2000u128));


 
}



#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn router_integration_tests_with_native_token() {    
    use std::io::{Stderr, Error};

    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::amm_pairs::amm_pairs_lib::amm_pairs_lib::{init_amm_pair, create_amm_settings, create_amm_pairs, create_custom_token, amm_pair_contract_store_in, add_liquidity_to_amm_pairs, get_amm_pair_info_query_liquidity};
    use multi_test::staking::staking_lib::staking_lib::{staking_contract_store_in, create_staking_info_contract};
    use router::contract::{instantiate, query, execute, reply};
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, mint_deposit_snip20, store_init_factory_contract, 
        convert_to_contract_link, snip20_lp_token_contract_store, create_token_pair, increase_allowance, snip_20_balance_query, create_token_pair_with_native, set_viewing_key, get_snip20_balance};
    use cosmwasm_std::{Uint128, Coin, ContractInfo, BlockInfo, StdError};
    use multi_test::util_addr::util_addr::{OWNER, STAKER_A};     
    use shadeswap_shared::core::{TokenAmount, ContractInstantiationInfo};
    use shadeswap_shared::msg::amm_pair::{InvokeMsg};
    use shadeswap_shared::msg::router::QueryMsgResponse;
    use shadeswap_shared::router::Hop;
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract};
    use multi_test::factory::factory_lib::factory_lib::{init_factory, create_amm_pairs_to_factory, list_amm_pairs_from_factory};
    use cosmwasm_std::Timestamp;
    
    pub fn router_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }
    
    let shade_dao_fee_address = Addr::unchecked("secret15yh35cflwdz6nn6zszvm90mf36a69uhm6rq6th");
    let staker_a_addr = Addr::unchecked(STAKER_A.to_owned());     
    let owner_addr = Addr::unchecked(OWNER);   
    let mut router = App::default();

    router.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(1 as u64),
        chain_id: "chain_id".to_string(),
    }); 

    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_addr.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });

    // GENERATE TOKEN PAIRS & REWARD TOKEN
    let token_0_contract = generate_snip20_contract(&mut router, "ETH".to_string(),"ETH".to_string(),18).unwrap();    
    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
   
    // MINT AND DEPOSIT FOR LIQUIDITY
    mint_deposit_snip20(&mut router,&token_0_contract,&owner_addr,Uint128::new(10000000000u128), &owner_addr);
    mint_deposit_snip20(&mut router,&reward_contract,&owner_addr,Uint128::new(10000000000u128), &owner_addr);           
    roll_blockchain(&mut router, 1).unwrap();    
   // INIT LP, STAKING, AMM PAIRS
    let admin_contract = init_admin_contract(&mut router, &owner_addr).unwrap(); //store_init_factory_contract(&mut router, &convert_to_contract_link(&admin_contract)).unwrap();
    let amm_contract_info = router.store_code(amm_pair_contract_store_in());
    let lp_token_info = router.store_code(snip20_lp_token_contract_store());
    let staking_info = router.store_code(staking_contract_store_in());
    
     // STORE ROUTER CONTRACT
     let router_contract_info = router.store_code(router_contract_store());
     roll_blockchain(&mut router, 1).unwrap();

    // INIT ROUTER CONTRACTs
    let init_msg = InitMsg {
        prng_seed: to_binary("password").unwrap(),
        entropy: to_binary("password").unwrap(),       
        admin_auth: convert_to_contract_link(&admin_contract),
    };    

    roll_blockchain(&mut router, 1).unwrap();
    let router_contract = router
        .instantiate_contract(
            router_contract_info,
            owner_addr.to_owned(),
            &init_msg,
            &[],
            "router",
            Some(OWNER.to_string()),
        ).unwrap();    

    // CREATE FACTORY
    roll_blockchain(&mut router, 1).unwrap(); 
    let factory_contract = init_factory(
        &mut router,
        &convert_to_contract_link(&admin_contract),
        &OWNER,
        false,
        create_amm_settings(
            3,
            100,
            8,
            100, 
            &shade_dao_fee_address
        ),
        ContractInstantiationInfo{
            code_hash: amm_contract_info.code_hash.clone(),
            id: amm_contract_info.code_id,
        },        
        ContractInstantiationInfo{
            code_hash: lp_token_info.code_hash.clone(),
            id: lp_token_info.code_id,
        },
        "seed",
        "api_key",
        None
    ).unwrap();

    // CREATE AMM_PAIR NATIVE - SNIP20
    roll_blockchain(&mut router, 1).unwrap();
    create_amm_pairs_to_factory(
        &mut router,
        &factory_contract,
        &create_token_pair_with_native(
            &convert_to_contract_link(&token_0_contract), 
        ),
        "seed",
        &create_staking_info_contract(
            staking_info.code_id, 
            &staking_info.code_hash, 
            Uint128::new(30000u128), 
            TokenType::CustomToken { 
                contract_addr: reward_contract.address.clone(), 
                token_code_hash: reward_contract.code_hash.clone() },
            Uint128::new(30000000000u128)
        ),
        &router_contract, 
        &owner_addr).unwrap();
       
      // LIST AMM PAIR
      let amm_pairs = list_amm_pairs_from_factory(
        &mut router,
        &factory_contract,
        0, 30
    ).unwrap();
    
    // ASSERT AMM PAIRS == 2
    assert_eq!(amm_pairs.len(), 2);

    // ADD LIQUIDITY TO AMM_PAIR NATIVE vs SNIP20
    increase_allowance(&mut router, &token_0_contract, Uint128::new(10000000000000000u128),&amm_pairs[1].address , &owner_addr).unwrap();
       
    let mut funds: Vec<Coin> = Vec::new();
    funds.push(Coin { denom: "uscrt".to_string(), amount: Uint128::new(10000u128)});

    add_liquidity_to_amm_pairs(
        &mut router,
        &ContractInfo{
            address: amm_pairs[1].address.clone(),
            code_hash: "".to_string(),
        },
        &amm_pairs[1].pair,
        Uint128::new(10000u128),
        Uint128::new(10000u128),
        Some(Uint128::new(10000u128)),
        Some(true),
        funds,
        &owner_addr
    ).unwrap();
    
    // ASSERT SWAPTOKENEXACT NATIVE EXCEPTION FOR SNIP 20 TOKEN
    let offer = TokenAmount{
        token: TokenType::NativeToken { denom: "uscrt".to_string()},
        amount: Uint128::new(2000u128)
    };

    let execute_swap = ExecuteMsg::SwapTokensForExact { 
        offer:offer.to_owned(),
        expected_return: Some(Uint128::new(1000u128)), 
        path: vec![Hop{addr: amm_pairs[1].address.to_owned(), code_hash: amm_contract_info.code_hash.clone()}],
        recipient: Some(owner_addr.to_string())
    };

    let _response = router.execute_contract(
        owner_addr.to_owned(), 
        &router_contract, 
        &execute_swap,  
        &[
            Coin{ 
                denom: "uscrt".to_string(), 
                amount: Uint128::new(2000u128)
            }
        ]).unwrap(); 
        
    // ASSERT STAKER SNIP20 BALANCE
    let _ = set_viewing_key(
        &mut router,
        &token_0_contract,
        "seed",
        &staker_a_addr).unwrap();

    let snip20_balance = get_snip20_balance(
        &mut router,
        &token_0_contract,
        STAKER_A,
        "seed");   

    // ASSERT BALANCE TOKEN 0 - STAKER A
    assert_eq!(snip20_balance, Uint128::new(2000u128));


 
}



