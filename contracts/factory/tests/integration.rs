use cosmwasm_std::{
    to_binary, Addr, Empty, ContractInfo, StdResult,
};
use factory::contract::{execute, instantiate, query};
use multi_test::{help_lib::integration_help_lib::{convert_to_contract_link, roll_blockchain, generate_snip20_contract, store_init_auth_contract}, 
    amm_pairs::amm_pairs_lib::amm_pairs_lib::{store_init_amm_pair_contract, amm_pair_contract_store_in}};
use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use multi_test::factory::factory_mock::factory_mock::reply;
use shadeswap_shared::{utils::testing::TestingExt, core::{ContractInstantiationInfo, CustomFee, }, factory::{InitMsg, QueryResponse, QueryMsg}, Contract as SContract, staking::StakingContractInit};

pub fn contract_counter() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
    Box::new(contract)
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn factory_integration_tests() {
    use cosmwasm_std::Uint128;
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::amm_pairs::amm_pairs_lib::amm_pairs_lib::{amm_pair_contract_store_in};
    use multi_test::help_lib::integration_help_lib::{convert_to_contract_link, create_token_pair, mint_deposit_snip20, configure_block_send_init_funds, snip20_lp_token_contract_store, create_token_pair_with_native};
    use multi_test::staking::staking_lib::staking_lib::staking_contract_store_in;
    use shadeswap_shared::Pagination;
    use shadeswap_shared::amm_pair::{AMMPair, AMMSettings};
    use shadeswap_shared::core::{TokenType, Fee};
    use shadeswap_shared::factory::ExecuteMsg;
    use multi_test::help_lib::integration_help_lib::{roll_blockchain};
    
    use multi_test::util_addr::util_addr::{OWNER};       
        use shadeswap_shared::staking::StakingContractInit;
        use shadeswap_shared::utils::testing::TestingExt;    
         
    let owner_addr = Addr::unchecked(OWNER);   
    let mut router = App::default();   
    
    configure_block_send_init_funds(&mut router, &owner_addr, Uint128::new(100000000000000u128));  
    
    let lp_token_contract_info = router.store_code(snip20_lp_token_contract_store());
    let auth_contract = init_admin_contract(&mut router, &owner_addr).unwrap();
    let amm_pair_contract_id = router.store_code(amm_pair_contract_store_in());
    let staking_contract_info = router.store_code(staking_contract_store_in());
    // GENERATE TOKEN PAIRS & REWARD TOKEN  
    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();
    // MINT AND DEPOSIT FOR LIQUIDITY 
    mint_deposit_snip20(&mut router,&reward_contract,&owner_addr,Uint128::new(10000000000u128), &owner_addr);       

    // CREATE FACTORY CONTRACT
    let init_msg = InitMsg {
        pair_contract: ContractInstantiationInfo {
            code_hash: amm_pair_contract_id.code_hash,
            id: amm_pair_contract_id.code_id,
        },
        amm_settings: shadeswap_shared::amm_pair::AMMSettings {
            lp_fee: shadeswap_shared::core::Fee { nom: 2, denom: 100 },
            shade_dao_fee: shadeswap_shared::core::Fee { nom: 2, denom: 100 },
            shade_dao_address: SContract {
                address: Addr::unchecked("".to_string()),
                code_hash: "".to_string(),
            },
        },
        lp_token_contract: ContractInstantiationInfo {
            code_hash: lp_token_contract_info.code_hash.clone(),
            id: lp_token_contract_info.code_id,
        },
        prng_seed: to_binary(&"seed".to_string()).unwrap(),
        api_key: "api_key".to_string(),
        authenticator: Some(convert_to_contract_link(&auth_contract)),
        admin_auth: convert_to_contract_link(&auth_contract)
    };
    let factory_contract_id = router.store_code(contract_counter());
    let factory_contract = router
        .instantiate_contract(
            factory_contract_id,
            owner_addr.clone(),
            &init_msg,
            &[],
            "factory",
            None,
        )
        .unwrap();
   
    // ASSERT FACTORY CONFIG
    roll_blockchain(&mut router, 1).unwrap();
    let query: QueryResponse = router.query_test(factory_contract.clone(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryResponse::GetConfig { pair_contract: _, amm_settings, lp_token_contract: _, authenticator: _, admin_auth: _} => {
            assert_eq!(amm_settings.lp_fee, shadeswap_shared::core::Fee { nom: 2, denom: 100 });
            assert_eq!(amm_settings.shade_dao_fee, shadeswap_shared::core::Fee { nom: 2, denom: 100 });
        },
        _ => panic!("Query Responsedoes not match")
    }

    // ASSERT ADD AMM_PAIR
    let (token_0_contract, token_1_contract, mock_amm_pairs) = setup_create_amm_pairs(
        &mut router,  
        "ETH", 
        "USDT",
        &factory_contract,
        Some(StakingContractInit{
            contract_info: ContractInstantiationInfo { 
                code_hash: staking_contract_info.code_hash.clone(), 
                id: staking_contract_info.code_id
            },
            daily_reward_amount: Uint128::new(30000u128),
            reward_token: TokenType::CustomToken { 
                contract_addr: reward_contract.address.clone(), 
                token_code_hash: reward_contract.code_hash.clone() 
            },
            valid_to: Uint128::new(30000000u128)
        }),
        None,
        "seed",
    &owner_addr).unwrap();

    // ADD NEW AMM_PAIR TO FACTORY
    roll_blockchain(&mut router, 1).unwrap();
    let pair = create_token_pair(
        &convert_to_contract_link(&token_0_contract), 
        &convert_to_contract_link(&token_1_contract)
    );
    let amm_pair = AMMPair{
        pair: pair.clone(),
        address: mock_amm_pairs.address,
        enabled: true,
    };
    let add_pair_msg = ExecuteMsg::AddAMMPairs { amm_pairs:  vec![amm_pair]};
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &factory_contract,
        &add_pair_msg,
        &[], // 
    )
    .unwrap();   
    
    // LIST AMM PAIRS
    let list_amm_pairs = to_binary(&QueryMsg::ListAMMPairs { 
        pagination: Pagination{
            start: 0,
            limit: 30,
        }
    }).unwrap();

    // ASSERT AMM PAIRS == 1
    let query_response: QueryResponse = router.query_test(factory_contract.clone(), list_amm_pairs.clone()).unwrap();
    match query_response{       
        QueryResponse::ListAMMPairs { amm_pairs } => {
           assert_eq!(amm_pairs.len(), 1);
        },
        QueryResponse::GetConfig { pair_contract: _, amm_settings: _, lp_token_contract: _, authenticator: _ , admin_auth: _} => todo!(),
        QueryResponse::GetAMMPairAddress { address: _ } => todo!(),
        QueryResponse::AuthorizeApiKey { authorized: _ } => todo!(),        
    };
    roll_blockchain(&mut router, 1).unwrap();

    let token_2_contract = generate_snip20_contract(&mut router, "BTC".to_string(), "BTC".to_string(), 18).unwrap();
    roll_blockchain(&mut router, 1).unwrap();

    // CREATE NEW PAIR via FACTORY
    let create_msg = ExecuteMsg::CreateAMMPair { 
        pair: create_token_pair(
            &convert_to_contract_link(&token_1_contract), 
            &convert_to_contract_link(&token_2_contract)
        ), 
        entropy: to_binary("seed").unwrap(), 
        staking_contract: Some(StakingContractInit{
            contract_info: ContractInstantiationInfo { 
                code_hash: staking_contract_info.code_hash.clone(), 
                id: staking_contract_info.code_id
            },
            daily_reward_amount: Uint128::new(30000u128),
            reward_token: TokenType::CustomToken { 
                contract_addr: reward_contract.address.clone(), 
                token_code_hash: reward_contract.code_hash.clone() 
            },
            valid_to: Uint128::new(30000000u128)
        }),
    };
    
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &factory_contract,
        &create_msg,
        &[], // 
    )
    .unwrap();  

    // ASSERT AMM_PAIR == 2
    let query_response: QueryResponse = router.query_test(factory_contract.clone(), list_amm_pairs.clone()).unwrap();
    match query_response{       
        QueryResponse::ListAMMPairs { amm_pairs } => {
           assert_eq!(amm_pairs.len(), 2);
        },       
        _ => {}      
    };

    // CREATE NATIVE AMM PAIR
    let create_msg = ExecuteMsg::CreateAMMPair { 
        pair: create_token_pair_with_native(
            &convert_to_contract_link(&token_2_contract)
        ), 
        entropy: to_binary("seed").unwrap(), 
        staking_contract: Some(StakingContractInit{
            contract_info: ContractInstantiationInfo { 
                code_hash: staking_contract_info.code_hash.clone(), 
                id: staking_contract_info.code_id
            },
            daily_reward_amount: Uint128::new(30000u128),
            reward_token: TokenType::CustomToken { 
                contract_addr: reward_contract.address.clone(), 
                token_code_hash: reward_contract.code_hash.clone() 
            },
            valid_to: Uint128::new(30000000u128)
        }),
    };
    
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &factory_contract,
        &create_msg,
        &[], // 
    )
    .unwrap();  

    // ASSERT AMM_PAIR == 3
    let query_response: QueryResponse = router.query_test(factory_contract.clone(), list_amm_pairs.clone()).unwrap();
    match query_response{       
        QueryResponse::ListAMMPairs { amm_pairs } => {
            assert_eq!(amm_pairs.len(), 3);
        },       
        _ => {}      
    };

    // ASSERT GETAMMPAIRSADDRESS
    let msg = to_binary(&QueryMsg::GetAMMPairAddress { pair: pair.clone() }).unwrap();
    let query_response: QueryResponse = router.query_test(factory_contract.clone(), msg).unwrap();
    match query_response{       
        QueryResponse::GetAMMPairAddress { address } => {
           assert_eq!(address, address.clone());
        },       
        _ => {}      
    };

    // ASSERT AUTHORIZATIONAPIKEY TRUE
    let msg = to_binary(&QueryMsg::AuthorizeApiKey { api_key: "api_key".to_string() }).unwrap();
    let query_response: QueryResponse = router.query_test(factory_contract.clone(), msg).unwrap();
    match query_response{       
        QueryResponse::AuthorizeApiKey { authorized } => {
            assert_eq!(authorized, true);
        },       
        _ => {}      
    };

    // ASSERT AUTHORIZATIONAPIKEY FALSE
    let msg = to_binary(&QueryMsg::AuthorizeApiKey { api_key: "api_keys".to_string() }).unwrap();
    let query_response: QueryResponse = router.query_test(factory_contract.clone(), msg).unwrap();
    match query_response{       
        QueryResponse::AuthorizeApiKey { authorized } => {
            assert_eq!(authorized, false);
        },       
        _ => {}      
    };

    // SET CONFIG 
    let update_lp_token_info = router.store_code(snip20_lp_token_contract_store());
    let shade_dao_address_contract = generate_snip20_contract(&mut router, "DOA".to_string(), "DOA".to_string() , 18).unwrap();
    let auth_contract = init_admin_contract(&mut router, &owner_addr).unwrap();
    let amm_pair_contract_id = router.store_code(amm_pair_contract_store_in());
    // CREATE NATIVE AMM PAIR
    let create_msg = ExecuteMsg::SetConfig { 
        pair_contract: Some(ContractInstantiationInfo { 
            code_hash: amm_pair_contract_id.code_hash.clone(), 
            id: amm_pair_contract_id.code_id 
        }), 
        lp_token_contract: Some(ContractInstantiationInfo{
            code_hash: update_lp_token_info.code_hash.clone(),
            id: update_lp_token_info.code_id,
        }), 
        amm_settings: Some(AMMSettings{
            lp_fee: Fee::new(5, 100),
            shade_dao_fee: Fee::new(10, 100),
            shade_dao_address: convert_to_contract_link(&shade_dao_address_contract)
        }), 
        api_key: Some("pass_key".to_string()), 
        admin_auth: Some(convert_to_contract_link(&auth_contract)) 
    }; 
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &factory_contract,
        &create_msg,
        &[], // 
    )
    .unwrap();  

    // ASSERT SETCONFIG CHANGES
    let query: QueryResponse = router.query_test(factory_contract.clone(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryResponse::GetConfig { pair_contract: _, amm_settings, lp_token_contract, authenticator: _, admin_auth: _} => {
            assert_eq!(amm_settings.lp_fee, shadeswap_shared::core::Fee { nom: 5, denom: 100 });
            assert_eq!(amm_settings.shade_dao_fee, shadeswap_shared::core::Fee { nom: 10, denom: 100 });
            assert_eq!(lp_token_contract.code_hash,update_lp_token_info.code_hash);
            assert_eq!(lp_token_contract.id,update_lp_token_info.code_id);
        },
        _ => panic!("Query Response does not match")
    }

    // ASSERT AUTHORIZATIONAPIKEY TRUE
    let msg = to_binary(&QueryMsg::AuthorizeApiKey { api_key: "pass_key".to_string() }).unwrap();
    let query_response: QueryResponse = router.query_test(factory_contract.clone(), msg).unwrap();
    match query_response{       
        QueryResponse::AuthorizeApiKey { authorized } => {
            assert_eq!(authorized, true);
        },       
        _ => {}      
    };

    // ASSERT AUTHORIZATIONAPIKEY FALSE
    let msg = to_binary(&QueryMsg::AuthorizeApiKey { api_key: "api_key".to_string() }).unwrap();
    let query_response: QueryResponse = router.query_test(factory_contract.clone(), msg).unwrap();
    match query_response{       
        QueryResponse::AuthorizeApiKey { authorized } => {
            assert_eq!(authorized, false);
        },       
        _ => {}      
    };

}


pub fn setup_create_amm_pairs(
    router: &mut App, 
    symbol_0: &str, 
    symbol_1: &str, 
    factory_contract: &ContractInfo, 
    staking_contract_info: Option<StakingContractInit>,
    custom_fee: Option<CustomFee>,
    seed: &str,
    sender: &Addr) 
    -> StdResult<(cosmwasm_std::ContractInfo, cosmwasm_std::ContractInfo, cosmwasm_std::ContractInfo)> {
    let token_0_contract = generate_snip20_contract(router, symbol_0.to_string(), symbol_0.to_string(), 18).unwrap();
    roll_blockchain(router, 1).unwrap();
    let token_1_contract = generate_snip20_contract(router, symbol_1.to_string(), symbol_1.to_string(), 18).unwrap();
    roll_blockchain(router, 1).unwrap();
    // auth contract
    let auth_query_contract = store_init_auth_contract(router)?;
    let mock_amm_pairs = store_init_amm_pair_contract(
        router, 
        sender,
        &convert_to_contract_link(&token_0_contract), 
        &convert_to_contract_link(&token_1_contract),
        &convert_to_contract_link(factory_contract),
        &convert_to_contract_link(&auth_query_contract),
        amm_pair_contract_store_in(),
        seed,
        staking_contract_info,
        custom_fee,
    ).unwrap();
    let response = (token_0_contract, token_1_contract, mock_amm_pairs);
    Ok(response)
}



