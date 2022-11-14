use staking::contract::{execute, instantiate, query};
use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use shadeswap_shared::msg::staking::{{InitMsg, QueryResponse, ExecuteMsg}};
use multi_test::help_lib::integration_help_lib::{mk_address};
use cosmwasm_std::{
    to_binary, Addr, Empty,
};

use shadeswap_shared::utils::asset::Contract as AuthContract;
pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
    Box::new(contract)
}
use shadeswap_shared::Contract as SContract;



#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn staking_integration_tests_without_proxy() {        
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, store_init_auth_contract, mint_deposit_snip20, send_snip20_to_stake, get_current_block_time, convert_to_contract_link};
    use cosmwasm_std::{Uint128, Coin, StdError};
    use multi_test::util_addr::util_addr::{OWNER, OWNER_PUB_KEY, STAKER_A, PUB_KEY_STAKER_A};       
    use multi_test::util_addr::util_blockchain::CHAIN_ID;
    use shadeswap_shared::staking::{QueryMsg};
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract};
    
    use crate::staking_help_query::query_claimable_reward;
       
    let staker_a_addr = Addr::unchecked(STAKER_A.to_owned());       
    let owner_addr = Addr::unchecked(OWNER);   
    let mut router = App::default();  

    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_addr.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });

    router.block_info().chain_id = CHAIN_ID.to_string();
    roll_blockchain(&mut router, 1).unwrap();
    let admin_contract = init_admin_contract(&mut router, &owner_addr).unwrap();
    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
    let staking_contract_info = router.store_code(staking_contract_store());
    let auth_contract = store_init_auth_contract(&mut router).unwrap();
    let lp_token_contract = generate_snip20_contract(&mut router, "LPT".to_string(),"LPT".to_string(),18).unwrap();
    let init_msg = InitMsg {
        daily_reward_amount: Uint128::new(30000u128),
        reward_token: TokenType::CustomToken { contract_addr:reward_contract.address.to_owned(), token_code_hash: reward_contract.code_hash.to_owned() },
        pair_contract: SContract { address: Addr::unchecked("AMMPAIR"), code_hash: "".to_string() },
        prng_seed: to_binary(&"password").unwrap(),
        lp_token: SContract { address:lp_token_contract.address.to_owned(), code_hash: lp_token_contract.code_hash.to_owned() },
        authenticator: Some(AuthContract{
            address: auth_contract.address.to_owned(),
            code_hash: auth_contract.code_hash.to_owned()
        }),
        admin_auth: convert_to_contract_link(&admin_contract),
        valid_to: Uint128::new(3747905010000u128) 
    };

    let staking_contract = router
        .instantiate_contract(
            staking_contract_info,
            mk_address(&OWNER).to_owned(),
            &init_msg,
            &[],
            "staking",
            Some(OWNER.to_string()),
        ).unwrap();
    
    roll_blockchain(&mut router, 2).unwrap();
    
    // Assert Staking Config
    let query: QueryResponse = router.query_test(staking_contract.to_owned(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryResponse::Config { reward_token, lp_token, daily_reward_amount, amm_pair: _, admin_auth: _ } => {
           assert_eq!(daily_reward_amount, Uint128::new(30000u128));
           assert_eq!(reward_token.address.to_string(), reward_contract.address.to_string());
           assert_eq!(lp_token.address.to_owned(), lp_token_contract.address.to_owned());
        },
        _ => panic!("Query Responsedoes not match")
    }

    roll_blockchain(&mut router, 1).unwrap();  
   
    // Assert Error StakingInfo not found
    let permit_query = query_claimable_reward(&router, 
        &staking_contract,OWNER_PUB_KEY, OWNER_PUB_KEY, get_current_block_time(&router));
    match permit_query {
        Ok(_) => todo!(),
        Err(err) =>assert_eq!(StdError::GenericErr{ msg: "Querier contract error: staking::state::StakingInfo not found".to_string() }, err),
    }

    // MINT & DEPOSIT LP TOKEN & REWARD TOKEN
    mint_deposit_snip20(&mut router,&lp_token_contract,&owner_addr,Uint128::new(100000000), &owner_addr);
    mint_deposit_snip20(&mut router,&reward_contract,&staking_contract.address,Uint128::new(100000000), &owner_addr);
    // STAKE LP TOKEN 
    send_snip20_to_stake(&mut router, 
        &lp_token_contract, 
        &staking_contract, 
        Uint128::new(1000u128),
        &owner_addr,
        &owner_addr).unwrap();
        
    // Assert zero for the same time
    let permit_query = query_claimable_reward(&router, 
        &staking_contract,OWNER_PUB_KEY, OWNER_PUB_KEY, get_current_block_time(&router)).unwrap();
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),1); 
           assert_eq!(claimable_rewards[0].amount, Uint128::zero());
        },
        _ => panic!("Query Responsedoes not match")
    } 

    roll_blockchain(&mut router, 1000).unwrap(); 
    let msg = ExecuteMsg::ClaimRewards {  }; 
    router.execute_contract(
        owner_addr.to_owned(),
        &staking_contract.clone(),
        &msg,
        &[], // 
    )
    .unwrap(); 
    
    // Assert claim_rewards to set claimable_reward to zero (already paid)
    let permit_query = query_claimable_reward(&router, 
        &staking_contract,OWNER_PUB_KEY, OWNER_PUB_KEY, get_current_block_time(&router) ).unwrap();
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),1); 
           assert_eq!(claimable_rewards[0].amount, Uint128::zero());
        },
        _ => panic!("Query Responsedoes not match")
    } 
   
    roll_blockchain(&mut router, 1).unwrap();
    // set 2 reward token
    let reward_contract_b = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
    let set_reward_msg = ExecuteMsg::SetRewardToken { 
        reward_token: SContract { 
            address:reward_contract_b.address.to_owned(), 
            code_hash: reward_contract_b.code_hash.to_owned() 
        }, 
        daily_reward_amount: Uint128::new(3000u128), 
        valid_to: Uint128::new(3747905010000u128) 
    };

    let _ = router.execute_contract(owner_addr.to_owned(), &staking_contract, &set_reward_msg, &[]).unwrap();
    mint_deposit_snip20(&mut router, &reward_contract_b, &staking_contract.address, Uint128::new(100000), &owner_addr);

    roll_blockchain(&mut router, 500).unwrap();    
    let msg = ExecuteMsg::ClaimRewards {  }; 
    router.execute_contract(
        Addr::unchecked(OWNER.to_owned()),
        &staking_contract.clone(),
        &msg,
        &[], // 
    )
    .unwrap(); 

    // Assert 2 Reward Token + Claimable Reward
    let permit_query = query_claimable_reward(&router, &staking_contract,OWNER_PUB_KEY, OWNER_PUB_KEY, get_current_block_time(&router)  + Uint128::new(1000u128) ).unwrap();   
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),2); 
           assert_eq!(claimable_rewards[0].amount, Uint128::new(347u128));
           assert_eq!(claimable_rewards[1].amount, Uint128::new(34u128));  
        },
        _ => panic!("Query Responsedoes not match")
    } 

    // Assert New Staker A
    mint_deposit_snip20(&mut router,&lp_token_contract, &staker_a_addr, Uint128::new(10000u128), &owner_addr);
    let _ = send_snip20_to_stake(&mut router, &lp_token_contract, &staking_contract, Uint128::new(1000u128), &staker_a_addr, &staker_a_addr).unwrap();
    // Query Balance
    let permit_query = query_claimable_reward(
        &router, 
        &staking_contract,
        PUB_KEY_STAKER_A, 
        PUB_KEY_STAKER_A, 
        get_current_block_time(&router) + Uint128::new(1000u128)
    ).unwrap();   

    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),2); 
           assert_eq!(claimable_rewards[0].amount, Uint128::new(173u128));
           assert_eq!(claimable_rewards[1].amount, Uint128::new(17u128));  
        },
        _ => panic!("Query Responsedoes not match")
    } 

    // Assert Unstake amount < total amount 
    roll_blockchain(&mut router, 1000).unwrap();
    let unstake_msg = ExecuteMsg::Unstake { amount: Uint128::new(500u128), remove_liqudity: Some(false)};
    let _ = router.execute_contract(owner_addr.to_owned(), &staking_contract, &unstake_msg, &[]).unwrap();
    
    let permit_query = query_claimable_reward(&router, 
        &staking_contract,OWNER_PUB_KEY, OWNER_PUB_KEY, get_current_block_time(&router) + Uint128::new(1000)).unwrap();
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),2); 
           assert_eq!(claimable_rewards[0].amount, Uint128::new(86u128));
           assert_eq!(claimable_rewards[1].amount, Uint128::new(8u128));  
        },
        _ => panic!("Query Responsedoes not match")
    } 
    // Assert Unstake the whole amount
    roll_blockchain(&mut router, 1).unwrap();
    let _ = router.execute_contract(owner_addr.to_owned(), &staking_contract, &unstake_msg, &[]).unwrap();
    let permit_query = query_claimable_reward(&router, 
        &staking_contract,OWNER_PUB_KEY, OWNER_PUB_KEY, get_current_block_time(&router)).unwrap();
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
            assert_eq!(claimable_rewards.len(),2); 
            assert_eq!(claimable_rewards[0].amount, Uint128::zero());
            assert_eq!(claimable_rewards[1].amount, Uint128::zero());  
        },
        _ => panic!("Query Responsedoes not match")
    } 
}


#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn staking_integration_tests_with_proxy() {        
    use multi_test::admin::admin_help::init_admin_contract;
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, store_init_auth_contract, mint_deposit_snip20, send_snip20_to_proxy_stake, set_viewing_key, convert_to_contract_link, get_current_block_time};
    use cosmwasm_std::{Uint128, Coin, StdError};
    use multi_test::util_addr::util_addr::{OWNER, OWNER_PUB_KEY, STAKER_A, PUB_KEY_STAKER_A};       
    use multi_test::util_addr::util_blockchain::CHAIN_ID;
    use shadeswap_shared::staking::{QueryMsg};
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract, snip_20_balance_query};
    use crate::staking_help_query::query_claimable_reward;
       
    let staker_a_addr = Addr::unchecked(STAKER_A.to_owned());       
    let owner_addr = Addr::unchecked(OWNER);   
    let mut router = App::default();  

    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_addr.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });

    router.block_info().chain_id = CHAIN_ID.to_string();
    roll_blockchain(&mut router, 1).unwrap();
    let admin_contract = init_admin_contract(&mut router, &owner_addr).unwrap();
    roll_blockchain(&mut router, 1).unwrap();
    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
    let staking_contract_info = router.store_code(staking_contract_store());
    let auth_contract = store_init_auth_contract(&mut router).unwrap();
    let lp_token_contract = generate_snip20_contract(&mut router, "LPT".to_string(),"LPT".to_string(),18).unwrap();
    let init_msg = InitMsg {
        daily_reward_amount: Uint128::new(30000u128),
        reward_token: TokenType::CustomToken { contract_addr:reward_contract.address.to_owned(), token_code_hash: reward_contract.code_hash.to_owned() },
        pair_contract: SContract { address: Addr::unchecked("AMMPAIR"), code_hash: "".to_string() },
        prng_seed: to_binary(&"password").unwrap(),
        lp_token: SContract { address:lp_token_contract.address.to_owned(), code_hash: lp_token_contract.code_hash.to_owned() },
        authenticator: Some(AuthContract{
            address: auth_contract.address.to_owned(),
            code_hash: auth_contract.code_hash.to_owned()
        }),
        admin_auth: convert_to_contract_link(&admin_contract),
        valid_to: Uint128::new(3747905010000u128) 
    };

    let staking_contract = router
        .instantiate_contract(
            staking_contract_info,
            mk_address(&OWNER).to_owned(),
            &init_msg,
            &[],
            "staking",
            Some(OWNER.to_string()),
        ).unwrap();
    
    roll_blockchain(&mut router, 2).unwrap();
    
    // Assert Staking Config
    let query: QueryResponse = router.query_test(staking_contract.to_owned(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryResponse::Config { reward_token, lp_token, daily_reward_amount, amm_pair: _, admin_auth: _ } => {
           assert_eq!(daily_reward_amount, Uint128::new(30000u128));
           assert_eq!(reward_token.address.to_string(), reward_contract.address.to_string());
           assert_eq!(lp_token.address.to_owned(), lp_token_contract.address.to_owned());
        },
        _ => panic!("Query Responsedoes not match")
    }

    roll_blockchain(&mut router, 1).unwrap();  
   
    // Assert Error StakingInfo not found
    let permit_query = query_claimable_reward(&router, 
        &staking_contract,OWNER_PUB_KEY, OWNER_PUB_KEY, get_current_block_time(&router));
    match permit_query {
        Ok(_) => todo!(),
        Err(err) => assert_eq!(StdError::GenericErr{ msg: "Querier contract error: staking::state::StakingInfo not found".to_string() }, err),
    }

    // MINT & DEPOSIT LP TOKEN & REWARD TOKEN
    mint_deposit_snip20(&mut router,&lp_token_contract,&owner_addr,Uint128::new(100000000), &owner_addr);
    mint_deposit_snip20(&mut router,&reward_contract,&staking_contract.address,Uint128::new(100000000), &owner_addr);
    // STAKE LP TOKEN 
    send_snip20_to_proxy_stake(&mut router, 
        &lp_token_contract, 
        &staking_contract, 
        Uint128::new(1000u128),
        &staker_a_addr,
        &owner_addr,
        &owner_addr).unwrap();
        
    // Assert zero for the Staker A
    let permit_query = query_claimable_reward(&router, 
        &staking_contract,PUB_KEY_STAKER_A, PUB_KEY_STAKER_A, get_current_block_time(&router)).unwrap();
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),1); 
           assert_eq!(claimable_rewards[0].amount, Uint128::zero());
        },
        _ => panic!("Query Responsedoes not match")
    } 

    let permit_query = query_claimable_reward(&router, 
        &staking_contract,PUB_KEY_STAKER_A, PUB_KEY_STAKER_A, get_current_block_time(&router) + Uint128::new(1000u128)).unwrap();
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),1); 
           assert_eq!(claimable_rewards[0].amount, Uint128::new(347u128));
        },
        _ => panic!("Query Responsedoes not match")
    }   
    roll_blockchain(&mut router, 200).unwrap(); 
    let msg = ExecuteMsg::ClaimRewards {  }; 
    router.execute_contract(
        staker_a_addr.to_owned(),
        &staking_contract.clone(),
        &msg,
        &[], // 
    )
    .unwrap();     
    // Assert claim_rewards to set claimable_reward to zero (already paid)
    let permit_query = query_claimable_reward(&router, 
        &staking_contract,PUB_KEY_STAKER_A, PUB_KEY_STAKER_A, get_current_block_time(&router)).unwrap();
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),1); 
           assert_eq!(claimable_rewards[0].amount, Uint128::zero());
        },
        _ => panic!("Query Responsedoes not match")
    } 


    set_viewing_key(&mut router, &reward_contract, "password", &staker_a_addr).unwrap();
    let balance = snip_20_balance_query(&router, &staker_a_addr,"password",&reward_contract).unwrap();
    assert_eq!(balance, Uint128::new(347u128));
    
    // Assert Unstake amount < total amount 
    roll_blockchain(&mut router, 100).unwrap();
    let unstake_msg = ExecuteMsg::ProxyUnstake { for_addr: staker_a_addr.to_string(), amount: Uint128::new(1000u128) };
    let _ = router.execute_contract(owner_addr.to_owned(), &staking_contract, &unstake_msg, &[]).unwrap();
    // ASSERT Claimable reward    
    let permit_query = query_claimable_reward(&router, 
        &staking_contract,PUB_KEY_STAKER_A, PUB_KEY_STAKER_A, get_current_block_time(&router)).unwrap();
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
            assert_eq!(claimable_rewards.len(),1); 
            assert_eq!(claimable_rewards[0].amount, Uint128::zero());
        },
        _ => panic!("Query Responsedoes not match")
    } 
}

pub mod staking_help_query{
    use cosmwasm_std::{StdResult, ContractInfo, to_binary, Uint128};
    use multi_test::{util_addr::util_blockchain::CHAIN_ID, help_lib::integration_help_lib::{ mk_create_permit_data}};
    use secret_multi_test::App;
    use shadeswap_shared::staking::{QueryResponse, QueryMsg, AuthQuery};
    use shadeswap_shared::utils::testing::TestingExt;
    
    pub fn query_claimable_reward(router: &App, staking_contract: &ContractInfo, pub_key: &str, signature: &str, time: Uint128) 
    -> StdResult<QueryResponse> {
        let permit = mk_create_permit_data(pub_key, signature, CHAIN_ID).unwrap();
        let query: StdResult<QueryResponse> = router.query_test(
            staking_contract.to_owned(),
            to_binary(&QueryMsg::WithPermit { 
                permit:permit,
                query: AuthQuery::GetClaimReward { time: time} 
            })?);
        return query
    }
}





