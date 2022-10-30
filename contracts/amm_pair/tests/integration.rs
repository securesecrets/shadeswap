use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use shadeswap_shared::msg::amm_pair::{{InitMsg,  ExecuteMsg, QueryMsg, QueryMsgResponse}};
use multi_test::help_lib::integration_help_lib::{mk_contract_link, mk_address};
use cosmwasm_std::{
    testing::{mock_env},
    to_binary, Addr, Empty, Binary, ContractInfo, Uint128,
};
use shadeswap_shared::c_std::BlockInfo;
use shadeswap_shared::utils::asset::Contract as AuthContract;

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn amm_pair_integration_tests() {    
    use amm_pair::contract::{instantiate, query, execute};
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, mint_deposit_snip20, send_snip20_to_stake, snip20_send, increase_allowance, get_current_block_time, 
        store_init_staking_contract, store_init_factory_contract, snip20_contract_store, create_token_pair, convert_to_contract_link, send_snip20_with_msg, staking_contract_store};
    use cosmwasm_std::{Uint128, Coin, StdError, StdResult, Timestamp, from_binary, Api};
    use multi_test::util_addr::util_addr::{OWNER, OWNER_SIGNATURE, OWNER_PUB_KEY, STAKER_A, STAKER_B, PUB_KEY_STAKER_A};       
    use multi_test::util_addr::util_blockchain::CHAIN_ID;
    use shadeswap_shared::core::{ContractLink, ContractInstantiationInfo, TokenPair, TokenPairAmount, TokenAmount};
    use shadeswap_shared::staking::StakingContractInit;
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract};
    use multi_test::help_lib::integration_help_lib::print_events;
    use multi_test::help_lib::integration_help_lib::snip20_lp_token_contract_store;
    use multi_test::amm_pair::amm_pair_mock::amm_pair_mock::reply;
    // use staking::contract::{execute as staking_execute, instantiate as staking_instantiate, query as staking_query};
    let staker_a_addr = Addr::unchecked(STAKER_A.to_owned());       
    let staker_b_addr = Addr::unchecked(STAKER_B.to_owned());       
    let owner_addr = Addr::unchecked(OWNER);   
    
    let mut router = App::default();  
    router.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_seconds(1 as u64),
        chain_id: "chain_id".to_string(),
    });

    // pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
    //     let contract = ContractWrapper::new_with_empty(staking_execute, staking_instantiate, staking_query);
    //     Box::new(contract)
    // }

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

    let lp_contract_info = router.store_code(snip20_lp_token_contract_store());
    let staking_contract_info = router.store_code(staking_contract_store());
    let factory_contract_info = store_init_factory_contract(&mut router).unwrap();
    let amm_pairs_info = router.store_code(amm_contract_store());
    roll_blockchain(&mut router, 1).unwrap();
    
    let pair = create_token_pair(
        &convert_to_contract_link(&token_0_contract), 
        &convert_to_contract_link(&token_1_contract)
    );

    let factory_link = ContractLink { 
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
        factory_info: factory_link.to_owned(), 
        prng_seed: to_binary("seed").unwrap(), 
        entropy: to_binary("seed").unwrap(),  
        admin: Some(owner_addr.to_owned()),
        staking_contract: Some(StakingContractInit{
            contract_info:  ContractInstantiationInfo { 
                code_hash: staking_contract_info.code_hash.to_owned(), 
                id: staking_contract_info.code_id},
            daily_reward_amount: Uint128::new(30000u128),
            reward_token: TokenType::CustomToken { 
                contract_addr: reward_contract.address.to_owned(), 
                token_code_hash: reward_contract.code_hash.to_owned()
            },
        }), 
        custom_fee: None, 
        callback: None 
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
            pair, 
            custom_fee 
        } => {
           assert_eq!(factory_contract.to_owned(),factory_link.to_owned());
           assert_eq!(custom_fee, None);
           assert_ne!(lp_token.address.to_string(), "".to_string());
           assert_eq!(staking_contract, None);
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
    
    // ADD LIQIDITY
    let add_liqudity_msg = ExecuteMsg::AddLiquidityToAMMContract { 
        deposit: TokenPairAmount{
            pair: pair.clone(),
            amount_0: Uint128::new(100000000u128),
            amount_1: Uint128::new(100000000u128),
        }, 
        expected_return: None, 
        staking: Some(true) 
    };
 
    let res = router.execute_contract(
        owner_addr.to_owned(),
        &amm_pair_contract,
        &add_liqudity_msg,
        &[]
    ).unwrap();
    
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
        to: Some(owner_addr.to_owned()), 
        router_link: None, 
        callback_signature: None 
    }).unwrap();

    let msg = send_snip20_with_msg(
        &mut router,
        &token_0_contract,
        &amm_pair_contract,
        Uint128::new(1000u128),
        &owner_addr,
        &swap_msg
    );    
}

