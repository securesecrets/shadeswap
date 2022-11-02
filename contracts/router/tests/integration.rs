use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use shadeswap_shared::msg::router::{InitMsg, ExecuteMsg, QueryMsg};
use cosmwasm_std::{
    to_binary, Addr, Empty,
};

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn router_integration_tests() {    
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::amm_pairs::amm_pairs_lib::amm_pairs_lib::store_init_amm_pair_contract;
    use router::contract::{instantiate, query, execute, reply};
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, mint_deposit_snip20, store_init_factory_contract, 
        convert_to_contract_link};
    use cosmwasm_std::{Uint128, Coin};
    use multi_test::util_addr::util_addr::{OWNER, STAKER_A};       
    use multi_test::util_addr::util_blockchain::CHAIN_ID;
    use shadeswap_shared::core::{TokenAmount};
    use shadeswap_shared::msg::amm_pair::InvokeMsg;
    use shadeswap_shared::msg::router::QueryMsgResponse;
    use shadeswap_shared::router::Hop;
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract};
    use multi_test::help_lib::integration_help_lib::print_events;

    let staker_a_addr = Addr::unchecked(STAKER_A.to_owned());     
    let owner_addr = Addr::unchecked(OWNER);   
    let mut router = App::default();  
    
    pub fn router_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }

    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_addr.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });

    // GENERATE TOKEN PAIRS   
    let token_0_contract = generate_snip20_contract(&mut router, "ETH".to_string(),"ETH".to_string(),18).unwrap();    
    let token_1_contract = generate_snip20_contract(&mut router, "USDT".to_string(),"USDT".to_string(),18).unwrap();    
    let offer = TokenAmount{
        token: TokenType::CustomToken { 
            contract_addr: token_0_contract.address.to_owned(), 
            token_code_hash: token_0_contract.code_hash.to_owned() 
        },
        amount: Uint128::new(1000u128)
    };
    // MINT AND DEPOSIT FOR LIQUIDITY
    mint_deposit_snip20(&mut router,&token_0_contract,&owner_addr,Uint128::new(10000000000), &owner_addr);
    mint_deposit_snip20(&mut router,&token_1_contract,&owner_addr,Uint128::new(10000000000), &owner_addr);   
    router.block_info().chain_id = CHAIN_ID.to_string();
    roll_blockchain(&mut router, 1).unwrap();
    let admin_contract = init_admin_contract(&mut router, &owner_addr).unwrap();
    let factory_contract = store_init_factory_contract(&mut router, &convert_to_contract_link(&admin_contract)).unwrap();
    let amm_pair_contract = store_init_amm_pair_contract(
        &mut router, 
        &convert_to_contract_link(&token_0_contract), 
        &convert_to_contract_link(&token_1_contract), 
        &convert_to_contract_link(&factory_contract),
        &convert_to_contract_link(&admin_contract),
        &owner_addr
    ).unwrap();
    
    // Create Router
    let router_contract_info = router.store_code(router_contract_store());
    roll_blockchain(&mut router, 1).unwrap();
    
    // INIT AMM PAIR
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

    roll_blockchain(&mut router, 1).unwrap();   
    // Register SNIP20
    let msg = ExecuteMsg::RegisterSNIP20Token { 
        token_addr: token_0_contract.address.to_string() , 
        token_code_hash: token_0_contract.code_hash.to_owned() 
    };
    roll_blockchain(&mut router, 1).unwrap();
    let _ = router.execute_contract(
        owner_addr.to_owned(), 
        &router_contract, 
        &msg, 
        &[]).unwrap();

    roll_blockchain(&mut router, 1).unwrap();
    let swap_query = QueryMsg::SwapSimulation { 
        offer: offer.to_owned(),
        path: vec![Hop{addr: amm_pair_contract.address.to_owned(), code_hash: amm_pair_contract.code_hash.clone()}] 
    };

    let query_response:QueryMsgResponse = router.query_test(
        router_contract.to_owned(),
        to_binary(&swap_query).unwrap()
    ).unwrap();

    match query_response {
        QueryMsgResponse::SwapSimulation {
            total_fee_amount,
            lp_fee_amount,
            shade_dao_fee_amount:_,
            result,
            price:_,
        } => {
        // Verify result not actual amount
           assert_ne!(total_fee_amount, Uint128::zero());
           assert_ne!(lp_fee_amount, Uint128::zero());
           assert_ne!(lp_fee_amount, Uint128::zero());
           assert_ne!(result.return_amount, Uint128::zero());
        },
        _ => panic!("Query Responsedoes not match")
    }

    let invoke_msg = to_binary(&InvokeMsg::SwapTokens { 
        expected_return: Some(Uint128::new(1000u128)), 
        to: Some(staker_a_addr.to_owned()), 
    }).unwrap();
   
    let msg = snip20_reference_impl::msg::ExecuteMsg::Send {
        recipient: amm_pair_contract.address.to_owned(),
        recipient_code_hash: Some(amm_pair_contract.code_hash.clone()),
        amount: Uint128::new(1000u128),
        msg: Some(invoke_msg),
        memo: None,
        padding: None,
    };

    let response = router.execute_contract(
        owner_addr.to_owned(),
        &token_0_contract,
        &msg,
        &[], // 
    )
    .unwrap();                

    print_events(response);
    roll_blockchain(&mut router, 1).unwrap();  
    let execute_swap = ExecuteMsg::SwapTokensForExact { 
        offer:offer.to_owned(),
        expected_return: Some(Uint128::new(1000u128)), 
        path: vec![Hop{addr: amm_pair_contract.address.to_owned(), code_hash: amm_pair_contract.code_hash.clone()}],
        recipient: Some(owner_addr.to_string())
    };

    let _response = router.execute_contract(
        owner_addr.to_owned(), 
        &router_contract, 
        &execute_swap,  
        &[]);   

}



