use cosmwasm_std::{
    to_binary, Addr, Empty, ContractInfo, StdResult,
};
use factory::contract::{execute, instantiate, query};
use multi_test::{help_lib::integration_help_lib::{convert_to_contract_link, roll_blockchain, generate_snip20_contract, store_init_auth_contract}, 
    amm_pairs::amm_pairs_lib::amm_pairs_lib::store_init_amm_pair_contract};
use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use shadeswap_shared::{utils::testing::TestingExt, core::{ContractInstantiationInfo, }, factory::{InitMsg, QueryResponse, QueryMsg}, Contract as SContract};

pub fn contract_counter() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
    Box::new(contract)
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn factory_integration_tests() {
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::amm_pairs::amm_pairs_lib::amm_pairs_lib::amm_pair_contract_store;
    use multi_test::help_lib::integration_help_lib::{convert_to_contract_link, create_token_pair};
    use shadeswap_shared::Pagination;
    use shadeswap_shared::amm_pair::AMMPair;
    use shadeswap_shared::factory::ExecuteMsg;
    use multi_test::help_lib::integration_help_lib::{roll_blockchain};
    
    use multi_test::util_addr::util_addr::{OWNER};       
        use shadeswap_shared::utils::testing::TestingExt;    
         
    let owner_addr = Addr::unchecked(OWNER);   
    let mut router = App::default();     
    let auth_contract = init_admin_contract(&mut router, &owner_addr).unwrap();
    let amm_pair_contract_id = router.store_code(amm_pair_contract_store());
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
            code_hash: "".to_string(),
            id: 0u64,
        },
        prng_seed: to_binary(&"".to_string()).unwrap(),
        api_key: "api_key".to_string(),
        authenticator: None,
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

    println!("{}", factory_contract.address.to_string());
    roll_blockchain(&mut router, 1).unwrap();
    let query: QueryResponse = router.query_test(factory_contract.clone(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryResponse::GetConfig { pair_contract: _, amm_settings, lp_token_contract: _, authenticator: _, admin_auth: _} => {
            assert_eq!(amm_settings.lp_fee, shadeswap_shared::core::Fee { nom: 2, denom: 100 });
            assert_eq!(amm_settings.shade_dao_fee, shadeswap_shared::core::Fee { nom: 2, denom: 100 });
        },
        _ => panic!("Query Responsedoes not match")
    }

    // Assert Add Amm_Pair
    let (token_0_contract, token_1_contract, mock_amm_pairs) = setup_create_amm_pairs(
        &mut router,  
        "ETH", 
        "USDT",
        &factory_contract,
    &owner_addr).unwrap();

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
    let (token_0_contract, token_1_contract, _mock_amm_pair) = setup_create_amm_pairs(
        &mut router,  
        "BTC", 
        "ETH",
        &factory_contract,
    &owner_addr).unwrap();
    let create_msg = ExecuteMsg::CreateAMMPair { 
        pair: create_token_pair(
            &convert_to_contract_link(&token_0_contract), 
            &convert_to_contract_link(&token_1_contract)
        ), 
        entropy: to_binary("seed").unwrap(), 
        staking_contract: None, 
        router_contract: None 
    };
    
    let _ = router.execute_contract(
        owner_addr.to_owned(),
        &factory_contract,
        &create_msg,
        &[], // 
    )
    .unwrap();  

    let query_response: QueryResponse = router.query_test(factory_contract.clone(), list_amm_pairs.clone()).unwrap();
    match query_response{       
        QueryResponse::ListAMMPairs { amm_pairs } => {
           assert_eq!(amm_pairs.len(), 2);
        },
        QueryResponse::GetConfig { pair_contract: _, amm_settings: _, lp_token_contract:_, authenticator: _, admin_auth: _ } => todo!(),
        QueryResponse::GetAMMPairAddress { address: _ } => todo!(),
        QueryResponse::AuthorizeApiKey { authorized: _ } => todo!(),  
        _ => {}      
    };

}


pub fn setup_create_amm_pairs(router: &mut App, symbol_0: &str, symbol_1: &str, factory_contract: &ContractInfo, sender: &Addr) 
    -> StdResult<(cosmwasm_std::ContractInfo, cosmwasm_std::ContractInfo, cosmwasm_std::ContractInfo)> {
    let token_0_contract = generate_snip20_contract(router, symbol_0.to_string(), symbol_0.to_string(), 18).unwrap();
    roll_blockchain(router, 1).unwrap();
    let token_1_contract = generate_snip20_contract(router, symbol_1.to_string(), symbol_1.to_string(), 18).unwrap();
    roll_blockchain(router, 1).unwrap();
    // auth contract
    let auth_query_contract = store_init_auth_contract(router)?;
    let mock_amm_pairs = store_init_amm_pair_contract(
        router, 
        &convert_to_contract_link(&token_0_contract), 
        &convert_to_contract_link(&token_1_contract),
        &convert_to_contract_link(factory_contract),
        &convert_to_contract_link(&auth_query_contract),
        &sender
    ).unwrap();
    let response = (token_0_contract, token_1_contract, mock_amm_pairs);
    Ok(response)
}



