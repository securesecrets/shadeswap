use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use shadeswap_shared::{msg::amm_pair::{{InitMsg,  ExecuteMsg, QueryMsg, QueryMsgResponse}}};
use cosmwasm_std::{
    to_binary, Addr, Empty, ContractInfo,
};
use shadeswap_shared::c_std::BlockInfo;


#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn amm_pair_integration_tests_with_custom_token() {    
    use amm_pair::contract::{instantiate, query, execute};
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, mint_deposit_snip20, increase_allowance, store_init_factory_contract, 
        create_token_pair, convert_to_contract_link, send_snip20_with_msg, get_snip20_balance, set_viewing_key, get_amm_pair_config, get_pair_liquidity_pool_balance};
    use cosmwasm_std::{Uint128, Coin, Timestamp};
    use multi_test::util_addr::util_addr::{OWNER};    
    use shadeswap_shared::core::{ ContractInstantiationInfo, TokenPairAmount, TokenAmount, CustomFee, Fee};
    use shadeswap_shared::msg::amm_pair::InvokeMsg; 
    use shadeswap_shared::staking::StakingContractInit;   
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract};    
    use multi_test::help_lib::integration_help_lib::snip20_lp_token_contract_store;
    use shadeswap_shared::Contract as SContract;
    use multi_test::amm_pairs::amm_pairs_mock::amm_pairs_mock::reply;
    use staking::contract::{execute as staking_execute, instantiate as staking_instantiate, query as staking_query};  
    let owner_addr = Addr::unchecked(OWNER);   
    
    let mut router = App::default();  
    router.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(1 as u64),
        chain_id: "chain_id".to_string(),
    });

    pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(staking_execute, staking_instantiate, staking_query);
        Box::new(contract)
    }

    pub fn amm_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query)
        .with_reply(reply);
        Box::new(contract)
    }

    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_addr.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });
 
    roll_blockchain(&mut router, 1).unwrap();    
    // GENERATE TOKEN PAIRS + FACTORY + STAKING 
    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
    let token_0_contract = generate_snip20_contract(&mut router, "ETH".to_string(),"ETH".to_string(),18).unwrap();    
    let token_1_contract = generate_snip20_contract(&mut router, "USDT".to_string(),"USDT".to_string(),18).unwrap();    

    // MINT AND DEPOSIT FOR LIQUIDITY
    mint_deposit_snip20(&mut router,&token_0_contract,&owner_addr,Uint128::new(10000000000u128), &owner_addr);
    mint_deposit_snip20(&mut router,&token_1_contract,&owner_addr,Uint128::new(10000000000u128), &owner_addr);
    let admin_contract = init_admin_contract(&mut router, &owner_addr).unwrap();
    let lp_contract_info = router.store_code(snip20_lp_token_contract_store());
    let staking_contract_info = router.store_code(staking_contract_store());
    let factory_contract_info = store_init_factory_contract(&mut router, &convert_to_contract_link(&admin_contract)).unwrap();
    let amm_pairs_info = router.store_code(amm_contract_store());
    roll_blockchain(&mut router, 1).unwrap();
    
    let pair = create_token_pair(
        &convert_to_contract_link(&token_0_contract), 
        &convert_to_contract_link(&token_1_contract)
    );

    let factory_link = SContract { 
        address:factory_contract_info.address,
        code_hash: factory_contract_info.code_hash
    };

    // INIT AMM PAIR
    let init_msg = InitMsg { 
        pair: pair.clone(), 
        lp_token_contract: ContractInstantiationInfo { 
            code_hash: lp_contract_info.code_hash.to_owned(), 
            id: lp_contract_info.code_id
        },
        factory_info: Some(factory_link.to_owned()), 
        prng_seed: to_binary("seed").unwrap(), 
        entropy: to_binary("seed").unwrap(),  
        admin_auth: convert_to_contract_link(&admin_contract),
        staking_contract: Some(StakingContractInit{
            contract_info:  ContractInstantiationInfo { 
                code_hash: staking_contract_info.code_hash.to_owned(), 
                id: staking_contract_info.code_id},
            daily_reward_amount: Uint128::new(30000u128),
            reward_token: TokenType::CustomToken { 
                contract_addr: reward_contract.address.to_owned(), 
                token_code_hash: reward_contract.code_hash.to_owned()
            },
            valid_to: Uint128::new(3747905010000u128),
            custom_label: None
        }), 
        custom_fee: None,
        arbitrage_contract: None,
        lp_token_decimals: 18u8,
        lp_token_custom_label: None
    };       
    
    roll_blockchain(&mut router, 1).unwrap();
    let amm_pair_contract = router
        .instantiate_contract(
            amm_pairs_info,
            owner_addr.to_owned(),
            &init_msg,
            &[],
            "amm_pair",
            Some(OWNER.to_string()),
        ).unwrap();

    // Assert AMM PAIR Config
    roll_blockchain(&mut router, 2).unwrap();
    let query: QueryMsgResponse = router.query_test(
        amm_pair_contract.to_owned(),
        to_binary(&QueryMsg::GetConfig { }).unwrap()
    ).unwrap();
    match query {
        QueryMsgResponse::GetConfig { 
            factory_contract, 
            lp_token, 
            staking_contract, 
            pair: _, 
            custom_fee 
        } => {
           assert_eq!(factory_contract.to_owned(),Some(factory_link.to_owned()));
           assert_eq!(custom_fee, None);
           assert_ne!(lp_token.address.to_string(), "".to_string());
           assert_ne!(staking_contract.unwrap().address.to_string(), "".to_string());           
        },
        _ => panic!("Query Responsedoes not match")
    }

    mint_deposit_snip20(
        &mut router, 
        &token_0_contract, 
        &owner_addr, 
        Uint128::new(100000000000u128), 
        &owner_addr
    );
    roll_blockchain(&mut router, 1).unwrap();
    mint_deposit_snip20(
        &mut router, 
        &token_1_contract, 
        &owner_addr, 
        Uint128::new(100000000000u128), 
        &owner_addr
    );    
    roll_blockchain(&mut router, 1).unwrap();
    let pair = create_token_pair(
        &convert_to_contract_link(&token_0_contract),
        &convert_to_contract_link(&token_1_contract)
    );

    increase_allowance(&mut router, &token_0_contract, Uint128::new(10000000000000u128), &amm_pair_contract.address, &owner_addr).unwrap();
    roll_blockchain(&mut router, 1).unwrap();
    increase_allowance(&mut router, &token_1_contract, Uint128::new(10000000000000u128), &amm_pair_contract.address, &owner_addr).unwrap();
    roll_blockchain(&mut router, 1).unwrap();
    
    // ADD LIQIDITY WITH STAKING
    let add_liqudity_msg = ExecuteMsg::AddLiquidityToAMMContract { 
        deposit: TokenPairAmount{
            pair: pair.clone(),
            amount_0: Uint128::new(100000000u128),
            amount_1: Uint128::new(100000000u128),
        }, 
        expected_return: Some(Uint128::new(1000u128)), 
        staking: Some(true),
        execute_sslp_virtual_swap: None,
    };
 
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &add_liqudity_msg,
        &[]
    ).unwrap();
    roll_blockchain(&mut router, 2);
    // ASSERT Token's Pool Liquidity
    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    assert_eq!(total_liquidity.0, Uint128::new(100000000u128));
    assert_eq!(total_liquidity.1, Uint128::new(100000000u128));
    assert_eq!(total_liquidity.2, Uint128::new(100000000u128));
    
    let query: QueryMsgResponse = router.query_test(amm_pair_contract.to_owned(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryMsgResponse::GetConfig { 
            factory_contract: _, 
            lp_token, 
            staking_contract: _, 
            pair: _, 
            custom_fee: _ 
        } => {
            let contract_info  =ContractInfo{
                address: lp_token.address.clone(),
                code_hash: lp_token.code_hash.to_string(),
            };
           let _ = set_viewing_key(&mut router, &contract_info, "seed", &owner_addr).unwrap();
           let balance = get_snip20_balance(&mut router, &ContractInfo{
            address: lp_token.address.clone(),
            code_hash: lp_token.code_hash.to_string(),
        }, OWNER, "seed");
           assert_eq!(balance, Uint128::zero());          
        },
        _ => panic!("Query Responsedoes not match")
    }

     // ADD LIQIDITY WITHOUT STAKING
     let add_liqudity_msg = ExecuteMsg::AddLiquidityToAMMContract { 
        deposit: TokenPairAmount{
            pair: pair.clone(),
            amount_0: Uint128::new(100000000u128),
            amount_1: Uint128::new(100000000u128),
        }, 
        expected_return: None, 
        staking: Some(false),
        execute_sslp_virtual_swap: None,
    };   
 
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &add_liqudity_msg,
        &[]
    ).unwrap();

    roll_blockchain(&mut router, 2);   

    let query: QueryMsgResponse = router.query_test(amm_pair_contract.to_owned(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryMsgResponse::GetConfig { 
            factory_contract: _, 
            lp_token, 
            staking_contract: _, 
            pair: _, 
            custom_fee: _ 
        } => {
            let contract_info  =ContractInfo{
                address: lp_token.address.clone(),
                code_hash: lp_token.code_hash.to_string(),
            };
            let _ = set_viewing_key(&mut router, &contract_info, "seed", &owner_addr).unwrap();
            let balance = get_snip20_balance(&mut router, &ContractInfo{
                address: lp_token.address.clone(),
                code_hash: lp_token.code_hash.to_string(),
            }, OWNER, "seed");
            assert_eq!(balance, Uint128::new(100000000u128));
          
        },
        _ => panic!("Query Responsedoes not match")
    }
    // ASSERT Token's Pool Liquidity
    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    assert_eq!(total_liquidity.0, Uint128::new(200000000u128));
    assert_eq!(total_liquidity.1, Uint128::new(200000000u128));
    assert_eq!(total_liquidity.2, Uint128::new(200000000u128));
     
    // SWAP TOKENS
    roll_blockchain(&mut router, 1).unwrap();
    let swap_msg = to_binary(&ExecuteMsg::SwapTokens { 
        offer: TokenAmount{
            token: TokenType::CustomToken { 
                contract_addr: token_0_contract.address.clone(), 
                token_code_hash: token_0_contract.code_hash.clone()
            },
            amount: Uint128::new(1000u128),
        }, 
        expected_return: Some(Uint128::new(500u128)), 
        to: Some(owner_addr.to_string()),
        execute_arbitrage: None,
    }).unwrap();

    let _ = send_snip20_with_msg(
        &mut router,
        &token_0_contract,
        &amm_pair_contract,
        Uint128::new(1000u128),
        &owner_addr,
        &swap_msg
    ).unwrap();    
    // ASSERT Token's Pool Liquidity _ OWNER -> SHADEADDRESS
    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    assert_eq!(total_liquidity.0, Uint128::new(200000000u128));
    assert_eq!(total_liquidity.1, Uint128::new(200001000u128));
    assert_eq!(total_liquidity.2, Uint128::new(199999030u128));

    // REMOVE LIQUIDITY
    roll_blockchain(&mut router, 1).unwrap();
    let remove_msg = to_binary(&InvokeMsg::RemoveLiquidity { 
        from: Some(owner_addr.to_string()),
        single_sided_withdraw_type: None,
        single_sided_expected_return: None,
    }).unwrap();
    
    let config = get_amm_pair_config(&mut router, &amm_pair_contract);
    let _ = send_snip20_with_msg(
        &mut router,
        &ContractInfo { 
            address: config.1.address, 
            code_hash: config.1.code_hash },
        &amm_pair_contract,
        Uint128::new(1000u128),
        &owner_addr,
        &remove_msg
    ).unwrap();       
    
    // SET CUSTOM FEE
    roll_blockchain(&mut router, 1).unwrap();
    let set_custom_fee = ExecuteMsg::SetCustomPairFee { 
        custom_fee: Some(CustomFee{
            shade_dao_fee: Fee::new(5, 100),
            lp_fee: Fee::new(3,100),
        })
    };

    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &set_custom_fee,
        &[]
    ).unwrap();

    let config = get_amm_pair_config(&mut router, &amm_pair_contract);
    let custom_fee: CustomFee = config.4.unwrap();
    assert_eq!(custom_fee.shade_dao_fee.to_owned(), Fee::new(5,100));
    assert_eq!(custom_fee.lp_fee.to_owned(), Fee::new(3,100));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn amm_pair_integration_tests_native_token() {    
    use amm_pair::contract::{instantiate, query, execute};
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, mint_deposit_snip20, increase_allowance, store_init_factory_contract, create_token_pair, convert_to_contract_link, send_snip20_with_msg, get_snip20_balance, set_viewing_key, get_amm_pair_config, get_pair_liquidity_pool_balance, create_token_pair_with_native};
    use cosmwasm_std::{Uint128, Coin, Timestamp, from_binary};
    use multi_test::util_addr::util_addr::{OWNER, STAKER_A, STAKER_B};       
    use shadeswap_shared::amm_pair::ExecuteMsgResponse;
    use shadeswap_shared::core::{ContractInstantiationInfo, TokenPairAmount, TokenAmount, CustomFee, Fee};
    use shadeswap_shared::msg::amm_pair::InvokeMsg;
    
    use shadeswap_shared::staking::StakingContractInit;
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract};    
    use multi_test::help_lib::integration_help_lib::snip20_lp_token_contract_store;
    use multi_test::amm_pairs::amm_pairs_mock::amm_pairs_mock::reply;
    use shadeswap_shared::Contract as SContract;
    use staking::contract::{execute as staking_execute, instantiate as staking_instantiate, query as staking_query};   
    let owner_addr = Addr::unchecked(OWNER);   
    
    let mut router = App::default();  
    router.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(1 as u64),
        chain_id: "chain_id".to_string(),
    });

    pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(staking_execute, staking_instantiate, staking_query);
        Box::new(contract)
    }

    pub fn amm_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query)
        .with_reply_empty(reply);
        Box::new(contract)
    }

    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_addr.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });
 
    roll_blockchain(&mut router, 1).unwrap();    
    // GENERATE TOKEN PAIRS + FACTORY + STAKING 
    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
    let token_0_contract = generate_snip20_contract(&mut router, "ETH".to_string(),"ETH".to_string(),18).unwrap();    
 
    // MINT AND DEPOSIT FOR LIQUIDITY
    mint_deposit_snip20(&mut router,&token_0_contract,&owner_addr,Uint128::new(10000000000u128), &owner_addr);
    let admin_contract = init_admin_contract(&mut router, &owner_addr).unwrap();
    let lp_contract_info = router.store_code(snip20_lp_token_contract_store());
    let staking_contract_info = router.store_code(staking_contract_store());
    let factory_contract_info = store_init_factory_contract(&mut router, &convert_to_contract_link(&admin_contract)).unwrap();
    let amm_pairs_info = router.store_code(amm_contract_store());
    roll_blockchain(&mut router, 1).unwrap();
    
    let pair = create_token_pair_with_native(
        &convert_to_contract_link(&token_0_contract)
    );

    let factory_link = SContract { 
        address:factory_contract_info.address,
        code_hash: factory_contract_info.code_hash
    };

    // INIT AMM PAIR
    let init_msg = InitMsg { 
        pair: pair.clone(), 
        lp_token_contract: ContractInstantiationInfo { 
            code_hash: lp_contract_info.code_hash.to_owned(), 
            id: lp_contract_info.code_id
        },
        factory_info: Some(factory_link.to_owned()), 
        prng_seed: to_binary("seed").unwrap(), 
        entropy: to_binary("seed").unwrap(),  
        admin_auth: convert_to_contract_link(&admin_contract),
        staking_contract: Some(StakingContractInit{
            contract_info:  ContractInstantiationInfo { 
                code_hash: staking_contract_info.code_hash.to_owned(), 
                id: staking_contract_info.code_id},
            daily_reward_amount: Uint128::new(30000u128),
            reward_token: TokenType::CustomToken { 
                contract_addr: reward_contract.address.to_owned(), 
                token_code_hash: reward_contract.code_hash.to_owned()
            },
            valid_to: Uint128::new(3747905010000u128),
            custom_label: None
        }), 
        custom_fee: None,
        arbitrage_contract: None,
        lp_token_decimals: 18u8,
        lp_token_custom_label: None
    };       
    
    roll_blockchain(&mut router, 1).unwrap();
    let amm_pair_contract = router
        .instantiate_contract(
            amm_pairs_info,
            owner_addr.to_owned(),
            &init_msg,
            &[],
            "amm_pair",
            Some(OWNER.to_string()),
        ).unwrap();

    // Assert AMM PAIR Config
    roll_blockchain(&mut router, 2).unwrap();
    let query: QueryMsgResponse = router.query_test(amm_pair_contract.to_owned(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryMsgResponse::GetConfig { 
            factory_contract, 
            lp_token, 
            staking_contract, 
            pair: _, 
            custom_fee 
        } => {
           assert_eq!(factory_contract.to_owned(),Some(factory_link.to_owned()));
           assert_eq!(custom_fee, None);
           assert_ne!(lp_token.address.to_string(), "".to_string());
           assert_ne!(staking_contract.unwrap().address.to_string(), "".to_string());           
        },
        _ => panic!("Query Responsedoes not match")
    }

    mint_deposit_snip20(
        &mut router, 
        &token_0_contract, 
        &owner_addr, 
        Uint128::new(100000000000u128), 
        &owner_addr
    );
    roll_blockchain(&mut router, 1).unwrap();

    let pair = create_token_pair_with_native(
        &convert_to_contract_link(&token_0_contract));
    increase_allowance(&mut router, &token_0_contract, Uint128::new(10000000000000u128), &amm_pair_contract.address, &owner_addr).unwrap();
    roll_blockchain(&mut router, 1).unwrap(); 
    
    // ADD LIQIDITY WITH STAKING
    let add_liqudity_msg = ExecuteMsg::AddLiquidityToAMMContract { 
        deposit: TokenPairAmount{
            pair: pair.clone(),
            amount_0: Uint128::new(100000000u128),
            amount_1: Uint128::new(100000000u128),
        }, 
        expected_return: Some(Uint128::new(1000u128)), 
        staking: Some(true),
        execute_sslp_virtual_swap: None,
    };
 
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &add_liqudity_msg,
        &[Coin{ denom: "uscrt".to_string(), amount: Uint128::new(100000000u128) }]
    ).unwrap();

    roll_blockchain(&mut router, 1).unwrap();
    // ASSERT Token's Pool Liquidity _ OWNER -> SHADEADDRESS
    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    assert_eq!(total_liquidity.0, Uint128::new(100000000u128));
    assert_eq!(total_liquidity.1, Uint128::new(100000000u128));
    assert_eq!(total_liquidity.2, Uint128::new(100000000u128));
    let query: QueryMsgResponse = router.query_test(amm_pair_contract.to_owned(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryMsgResponse::GetConfig { 
            factory_contract: _, 
            lp_token, 
            staking_contract: _, 
            pair: _, 
            custom_fee: _ 
        } => {
            let contract_info  =ContractInfo{
                address: lp_token.address.clone(),
                code_hash: lp_token.code_hash.to_string(),
            };
           let _ = set_viewing_key(&mut router, &contract_info, "seed", &owner_addr).unwrap();
           let balance = get_snip20_balance(&mut router, &ContractInfo{
            address: lp_token.address.clone(),
            code_hash: lp_token.code_hash.to_string(),
        }, OWNER, "seed");
           assert_eq!(balance, Uint128::zero());          
        },
        _ => panic!("Query Responsedoes not match")
    }

     // ADD LIQIDITY WITHOUT STAKING
     let add_liqudity_msg = ExecuteMsg::AddLiquidityToAMMContract { 
        deposit: TokenPairAmount{
            pair: pair.clone(),
            amount_0: Uint128::new(100000000u128),
            amount_1: Uint128::new(100000000u128),
        }, 
        expected_return: None, 
        staking: Some(false),
        execute_sslp_virtual_swap: None,
    };
 
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &add_liqudity_msg,
        &[Coin{ denom: "uscrt".to_string(), amount: Uint128::new(100000000u128) }]
    ).unwrap();
    roll_blockchain(&mut router, 1).unwrap();
    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    assert_eq!(total_liquidity.0, Uint128::new(200000000u128));
    assert_eq!(total_liquidity.1, Uint128::new(200000000u128));
    assert_eq!(total_liquidity.2, Uint128::new(200000000u128));

    let query: QueryMsgResponse = router.query_test(amm_pair_contract.to_owned(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryMsgResponse::GetConfig { 
            factory_contract: _, 
            lp_token, 
            staking_contract: _, 
            pair: _, 
            custom_fee: _ 
        } => {
            let contract_info  =ContractInfo{
                address: lp_token.address.clone(),
                code_hash: lp_token.code_hash.to_string(),
            };
            let _ = set_viewing_key(&mut router, &contract_info, "seed", &owner_addr).unwrap();
            let balance = get_snip20_balance(&mut router, &ContractInfo{
                address: lp_token.address.clone(),
                code_hash: lp_token.code_hash.to_string(),
            }, OWNER, "seed");
            assert_eq!(balance, Uint128::new(100000000));
          
        },
        _ => panic!("Query Responsedoes not match")
    }

    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    assert_eq!(total_liquidity.0, Uint128::new(200000000u128));
    assert_eq!(total_liquidity.1, Uint128::new(200000000u128));
    assert_eq!(total_liquidity.2, Uint128::new(200000000u128));
     
    // SWAP TOKENS
    roll_blockchain(&mut router, 1).unwrap();
    let swap_msg = ExecuteMsg::SwapTokens { 
        offer: TokenAmount{
            token: TokenType::NativeToken { denom: "uscrt".to_string()},
            amount: Uint128::new(1000u128),
        }, 
        expected_return: Some(Uint128::new(500u128)), 
        to: Some(owner_addr.to_string()),
        execute_arbitrage: None
    };

    let result = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &swap_msg,
        &[Coin{ denom: "uscrt".to_string(), amount: Uint128::new(1000u128) }]
    ).unwrap();

    match result.data {
        Some(d) => {
            match from_binary(&Some(d).unwrap()).unwrap() {
                ExecuteMsgResponse::SwapResult { price, amount_in, amount_out, total_fee_amount, lp_fee_amount, shade_dao_fee_amount } => {
                    assert_eq!(price, "0.941");
                    assert_eq!(amount_in, Uint128::new(1000u128));
                    assert_eq!(amount_out, Uint128::new(941u128));
                    assert_eq!(total_fee_amount, Uint128::new(58u128));
                    assert_eq!(lp_fee_amount, Uint128::new(29u128));
                    assert_eq!(shade_dao_fee_amount, Uint128::new(29u128));
                }
                _ => panic!("Failed to match result")
            }
        },
        None => panic!("Failed to match result"),
    }

 

    // REMOVE LIQUIDITY
    roll_blockchain(&mut router, 1).unwrap();
    let remove_msg = to_binary(&InvokeMsg::RemoveLiquidity { 
        from: Some(owner_addr.to_string()),
        single_sided_withdraw_type: None,
        single_sided_expected_return: None,
    }).unwrap();
    
    let config = get_amm_pair_config(&mut router, &amm_pair_contract);
    let _ = send_snip20_with_msg(
        &mut router,
        &ContractInfo { 
            address: config.1.address, 
            code_hash: config.1.code_hash },
        &amm_pair_contract,
        Uint128::new(1000u128),
        &owner_addr,
        &remove_msg
    );    

    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    assert_eq!(total_liquidity.0, Uint128::new(199999000u128));
    assert_eq!(total_liquidity.0, Uint128::new(199999000u128));
    assert_eq!(total_liquidity.2, Uint128::new(199998031u128));
    
    // SET CUSTOM FEE
    roll_blockchain(&mut router, 1).unwrap();
    let set_custom_fee = ExecuteMsg::SetCustomPairFee { 
        custom_fee: Some(CustomFee{
            shade_dao_fee: Fee::new(5, 100),
            lp_fee: Fee::new(3,100),
        })
    };

    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &set_custom_fee,
        &[]
    ).unwrap();

    let config = get_amm_pair_config(&mut router, &amm_pair_contract);
    let custom_fee: CustomFee = config.4.unwrap();
    assert_eq!(custom_fee.shade_dao_fee.to_owned(), Fee::new(5,100));
    assert_eq!(custom_fee.lp_fee.to_owned(), Fee::new(3,100));
}



#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn test_sslp_with_two_virtual_providers() {    
    use std::str::FromStr;

    use amm_pair::contract::{instantiate, query, execute};
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, mint_deposit_snip20, increase_allowance, store_init_factory_contract, create_token_pair, convert_to_contract_link, send_snip20_with_msg, get_snip20_balance, set_viewing_key, get_amm_pair_config, get_pair_liquidity_pool_balance, create_token_pair_with_native};
    use cosmwasm_std::{Uint128, Coin, Timestamp, from_binary, Response, StdError};
    use multi_test::util_addr::util_addr::{OWNER, STAKER_A, STAKER_B};       
    use secret_multi_test::AppResponse;
    use shadeswap_shared::core::{ContractInstantiationInfo, TokenPairAmount, TokenAmount, CustomFee, Fee};
    use shadeswap_shared::msg::amm_pair::InvokeMsg;
    
    use shadeswap_shared::staking::StakingContractInit;
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract};    
    use multi_test::help_lib::integration_help_lib::snip20_lp_token_contract_store;
    use multi_test::amm_pairs::amm_pairs_mock::amm_pairs_mock::reply;
    use shadeswap_shared::Contract as SContract;
    use staking::contract::{execute as staking_execute, instantiate as staking_instantiate, query as staking_query};   
    let owner_addr = Addr::unchecked(OWNER);   
    
    let mut router = App::default();  
    router.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(1 as u64),
        chain_id: "chain_id".to_string(),
    });

    pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(staking_execute, staking_instantiate, staking_query);
        Box::new(contract)
    }

    pub fn amm_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query)
        .with_reply_empty(reply);
        Box::new(contract)
    }

    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_addr.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });
 
    roll_blockchain(&mut router, 1).unwrap();    
    // GENERATE TOKEN PAIRS + FACTORY + STAKING 
    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
    let token_0_contract = generate_snip20_contract(&mut router, "ETH".to_string(),"ETH".to_string(),18).unwrap();    
 
    // MINT AND DEPOSIT FOR LIQUIDITY
    mint_deposit_snip20(&mut router,&token_0_contract,&owner_addr,Uint128::new(10000000000u128), &owner_addr);
    let admin_contract = init_admin_contract(&mut router, &owner_addr).unwrap();
    let lp_contract_info = router.store_code(snip20_lp_token_contract_store());
    let staking_contract_info = router.store_code(staking_contract_store());
    let factory_contract_info = store_init_factory_contract(&mut router, &convert_to_contract_link(&admin_contract)).unwrap();
    let amm_pairs_info = router.store_code(amm_contract_store());
    roll_blockchain(&mut router, 1).unwrap();
    
    let pair = create_token_pair_with_native(
        &convert_to_contract_link(&token_0_contract)
    );

    let factory_link = SContract { 
        address:factory_contract_info.address,
        code_hash: factory_contract_info.code_hash
    };

    // INIT AMM PAIR
    let init_msg = InitMsg { 
        pair: pair.clone(), 
        lp_token_contract: ContractInstantiationInfo { 
            code_hash: lp_contract_info.code_hash.to_owned(), 
            id: lp_contract_info.code_id
        },
        factory_info: Some(factory_link.to_owned()), 
        prng_seed: to_binary("seed").unwrap(), 
        entropy: to_binary("seed").unwrap(),
        admin_auth: convert_to_contract_link(&admin_contract),
        staking_contract: Some(StakingContractInit{
            contract_info:  ContractInstantiationInfo { 
                code_hash: staking_contract_info.code_hash.to_owned(), 
                id: staking_contract_info.code_id},
            daily_reward_amount: Uint128::new(30000u128),
            reward_token: TokenType::CustomToken { 
                contract_addr: reward_contract.address.to_owned(), 
                token_code_hash: reward_contract.code_hash.to_owned()
            },
            valid_to: Uint128::new(3747905010000u128),
            custom_label: None
        }), 
        custom_fee: None,
        arbitrage_contract: None,
        lp_token_decimals: 18u8,
        lp_token_custom_label: None
    };       
    
    roll_blockchain(&mut router, 1).unwrap();
    let amm_pair_contract = router
        .instantiate_contract(
            amm_pairs_info,
            owner_addr.to_owned(),
            &init_msg,
            &[],
            "amm_pair",
            Some(OWNER.to_string()),
        ).unwrap();

    // Assert AMM PAIR Config
    roll_blockchain(&mut router, 2).unwrap();
    let query: QueryMsgResponse = router.query_test(amm_pair_contract.to_owned(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryMsgResponse::GetConfig { 
            factory_contract, 
            lp_token, 
            staking_contract, 
            pair: _, 
            custom_fee 
        } => {
           assert_eq!(factory_contract.to_owned(),Some(factory_link.to_owned()));
           assert_eq!(custom_fee, None);
           assert_ne!(lp_token.address.to_string(), "".to_string());
           assert_ne!(staking_contract.unwrap().address.to_string(), "".to_string());           
        },
        _ => panic!("Query Responsedoes not match")
    }

    mint_deposit_snip20(
        &mut router, 
        &token_0_contract, 
        &owner_addr, 
        Uint128::new(100000000000u128), 
        &owner_addr
    );
    roll_blockchain(&mut router, 1).unwrap();

    let pair = create_token_pair_with_native(
        &convert_to_contract_link(&token_0_contract));
    increase_allowance(&mut router, &token_0_contract, Uint128::new(10000000000000u128), &amm_pair_contract.address, &owner_addr).unwrap();
    roll_blockchain(&mut router, 1).unwrap(); 

    // ASSERT EXCEPTION
     let add_liqudity_msg = ExecuteMsg::AddLiquidityToAMMContract { 
        deposit: TokenPairAmount{
            pair: pair.clone(),
            amount_0: Uint128::new(100000000u128),
            amount_1: Uint128::new(100000000u128),
        }, 
        expected_return: Some(Uint128::new(100000001u128)), 
        staking: Some(false),
        execute_sslp_virtual_swap: None,
    };
    let result = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &add_liqudity_msg,
        &[Coin{ denom: "uscrt".to_string(), amount: Uint128::new(100000000u128) }]
    );
    assert!(result.is_err());
    
     // *** user 1 add balanced liqidity without staking
     let add_liqudity_msg = ExecuteMsg::AddLiquidityToAMMContract { 
        deposit: TokenPairAmount{
            pair: pair.clone(),
            amount_0: Uint128::new(100000000u128),
            amount_1: Uint128::new(100000000u128),
        }, 
        expected_return: None, 
        staking: Some(false),
        execute_sslp_virtual_swap: None,
    };
 
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &add_liqudity_msg,
        &[Coin{ denom: "uscrt".to_string(), amount: Uint128::new(100000000u128) }]
    ).unwrap();
    roll_blockchain(&mut router, 1).unwrap();

    let query: QueryMsgResponse = router.query_test(amm_pair_contract.to_owned(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    let user1_lp_balance = match query {
        QueryMsgResponse::GetConfig { 
            factory_contract: _, 
            lp_token, 
            staking_contract: _, 
            pair: _, 
            custom_fee: _ 
        } => {
            let contract_info  =ContractInfo{
                address: lp_token.address.clone(),
                code_hash: lp_token.code_hash.to_string(),
            };
            let _ = set_viewing_key(&mut router, &contract_info, "seed", &owner_addr).unwrap();
            let balance = get_snip20_balance(&mut router, &ContractInfo{
                address: lp_token.address.clone(),
                code_hash: lp_token.code_hash.to_string(),
            }, OWNER, "seed");
            assert_eq!(balance, Uint128::new(100000000));
            balance
          
        },
        _ => panic!("Query Responsedoes not match")
    };

    // ASSERT POOL LIQUIDITY
    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    assert_eq!(total_liquidity.0, Uint128::new(100000000u128));
    assert_eq!(total_liquidity.1, Uint128::new(100000000u128));
    assert_eq!(total_liquidity.2, Uint128::new(100000000u128));
    
    /// ADD LIQIDITY
    let add_liqudity_msg = ExecuteMsg::AddLiquidityToAMMContract { 
        deposit: TokenPairAmount{
            pair: pair.clone(),
            amount_0: Uint128::new(1000000u128),
            amount_1: Uint128::new(100000000u128),
        }, 
        expected_return: None, 
        staking: None,
        execute_sslp_virtual_swap: Some(true),
    };
 
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &add_liqudity_msg,
        &[Coin{ denom: "uscrt".to_string(), amount: Uint128::new(1000000u128) }]
    ).unwrap();

    roll_blockchain(&mut router, 1).unwrap();
    // ASSERT POOL LIQUIDITY
    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    assert_eq!(total_liquidity.0, Uint128::new(147995300u128));
    assert_eq!(total_liquidity.1, Uint128::new(99500150u128));
    assert_eq!(total_liquidity.2, Uint128::new(200000000u128));

    // *** user 2 add sslp liqidity without staking
    let add_liqudity_msg = ExecuteMsg::AddLiquidityToAMMContract { 
        deposit: TokenPairAmount{
            pair: pair.clone(),
            amount_0: Uint128::new(200u128),
            amount_1: Uint128::zero(),
        }, 
        expected_return: None, 
        staking: Some(false),
        execute_sslp_virtual_swap: Some(true),
    };
 
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &add_liqudity_msg,
        &[Coin{ denom: "uscrt".to_string(), amount: Uint128::new(200u128) }]
    ).unwrap();

    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    let user2_lp_balance = total_liquidity.0 - user1_lp_balance;    

    // *** user 2 withdraws all their liquidity
    roll_blockchain(&mut router, 1).unwrap();
    let remove_msg = to_binary(&InvokeMsg::RemoveLiquidity { 
        from: Some(owner_addr.to_string()),
        single_sided_withdraw_type: Some(TokenType::NativeToken { denom: "uscrt".to_string() }),
        single_sided_expected_return: None,
    }).unwrap();
    
    let config = get_amm_pair_config(&mut router, &amm_pair_contract);
    let app_response = send_snip20_with_msg(
        &mut router,
        &ContractInfo { 
            address: config.1.address, 
            code_hash: config.1.code_hash },
        &amm_pair_contract,
        user2_lp_balance,
        &owner_addr,
        &remove_msg
    ).unwrap();
    // let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    // assert_eq!(total_liquidity.0, Uint128::new(100000000u128));
    // assert!(total_liquidity.1 > Uint128::new(100000000u128));
    // assert_eq!(total_liquidity.2, Uint128::new(100000000u128));

    // *** test expected return on withdraw
    roll_blockchain(&mut router, 1).unwrap();
    let remove_msg = to_binary(&InvokeMsg::RemoveLiquidity { 
        from: Some(owner_addr.to_string()),
        single_sided_withdraw_type: Some(TokenType::NativeToken { denom: "uscrt".to_string() }),
        single_sided_expected_return: Some(Uint128::new(300000000u128)),
    }).unwrap();
    
    let config = get_amm_pair_config(&mut router, &amm_pair_contract);
    // let response = send_snip20_with_msg(
    //     &mut router,
    //     &ContractInfo { 
    //         address: config.1.address.clone(), 
    //         code_hash: config.1.code_hash.clone() },
    //     &amm_pair_contract,
    //     user1_lp_balance,
    //     &owner_addr,
    //     &remove_msg
    // );
    let msg = snip20_reference_impl::msg::ExecuteMsg::Send {
        recipient: amm_pair_contract.address.to_owned(),
        recipient_code_hash: Some(amm_pair_contract.code_hash.clone()),
        amount: user1_lp_balance,
        msg: Some(remove_msg.clone()),
        memo: None,
        padding: None,
    };
    let contract = &ContractInfo { 
            address: config.1.address.clone(), 
            code_hash: config.1.code_hash.clone() };
    let response = router.execute_contract(
        owner_addr.to_owned(),
        &contract.clone(),
        &msg,
        &[], // 
    );
    assert!(response.is_err());
     
    // *** user 1 withdraws all their liquidity, should get original amount back because no trades have been executed and all sslp removed
    roll_blockchain(&mut router, 1).unwrap();
    let remove_msg = to_binary(&InvokeMsg::RemoveLiquidity { 
        from: Some(owner_addr.to_string()),
        single_sided_withdraw_type: None,
        single_sided_expected_return: None,
    }).unwrap();
    
    let config = get_amm_pair_config(&mut router, &amm_pair_contract);
    let app_response = send_snip20_with_msg(
        &mut router,
        &ContractInfo { 
            address: config.1.address, 
            code_hash: config.1.code_hash },
        &amm_pair_contract,
        user1_lp_balance,
        &owner_addr,
        &remove_msg
    ).unwrap();


    let total_liquidity: (Uint128, Uint128, Uint128) = get_pair_liquidity_pool_balance(&mut router,&amm_pair_contract);
    assert_eq!(total_liquidity.0, Uint128::zero());
    assert_eq!(total_liquidity.1, Uint128::zero());
    assert_eq!(total_liquidity.2, Uint128::zero());
    
}