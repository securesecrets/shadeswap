use staking::contract::{execute, instantiate, query};
// use lp_token::contract::{execute as lp_execute, instantiate as lp_instantiate, query as lp_query};

use secret_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};
// use multi_test::{auth_query::execute};

use shadeswap_shared::msg::staking::{{InitMsg, QueryResponse, ExecuteMsg}};
use multi_test::help_lib::integration_help_lib::{mk_contract_link, mk_address};
use cosmwasm_std::{
    testing::{mock_env, MockApi},
    to_binary, Addr, Empty, Binary, ContractInfo,
};

use shadeswap_shared::utils::asset::Contract as AuthContract;
pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
    Box::new(contract)
} 



#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn staking_integration_tests() {        
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, store_init_auth_contract, mint_deposit_snipo20, send_snip20_to_stake};
    use cosmwasm_std::{Uint128, Coin, StdError, StdResult};
    use multi_test::util_addr::util_addr::{OWNER, OWNER_SIGNATURE, OWNER_PUB_KEY};       
    use shadeswap_shared::core::ContractLink;
    use shadeswap_shared::staking::{QueryMsg, AuthQuery};
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract,
         mk_create_permit_data, get_current_timestamp};
       
    
    let owner_address = Addr::unchecked(OWNER);
    let mut router = App::default();  
    let chain_id = "pulsar-2".to_string();
    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_address.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });

    router.block_info().chain_id = chain_id.to_string();
    roll_blockchain(&mut router, 1).unwrap();

    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
    println!("reward token A addr {}", reward_contract.address.to_string());
    let staking_contract_info = router.store_code(staking_contract_store());
    let auth_contract = store_init_auth_contract(&mut router).unwrap();
    let lp_token_contract = generate_snip20_contract(&mut router, "LPT".to_string(),"LPT".to_string(),18).unwrap();
    let init_msg = InitMsg {
        daily_reward_amount: Uint128::new(30000u128),
        reward_token: TokenType::CustomToken { contract_addr:reward_contract.address.to_owned(), token_code_hash: reward_contract.code_hash.to_owned() },
        pair_contract: ContractLink { address: Addr::unchecked("AMMPAIR"), code_hash: "".to_string() },
        prng_seed: to_binary(&"password").unwrap(),
        lp_token: ContractLink { address:lp_token_contract.address.to_owned(), code_hash: lp_token_contract.code_hash.to_owned() },
        authenticator: Some(AuthContract{
            address: auth_contract.address.to_owned(),
            code_hash: auth_contract.code_hash.to_owned()
        }),
        admin: Addr::unchecked(OWNER),
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
        QueryResponse::Config { reward_token, lp_token, daily_reward_amount, amm_pair: _ } => {
           assert_eq!(daily_reward_amount, Uint128::new(30000u128));
           assert_eq!(reward_token.address.to_string(), reward_contract.address.to_string());
           assert_eq!(lp_token.address.to_owned(), lp_token_contract.address.to_owned());
        },
        _ => panic!("Query Responsedoes not match")
    }

    roll_blockchain(&mut router, 1).unwrap();  
   
    // Assert Error StakingInfo not found
    let permit = mk_create_permit_data(OWNER_PUB_KEY, OWNER_SIGNATURE, &chain_id).unwrap();
    let query: StdResult<QueryResponse> = router.query_test(
        staking_contract.to_owned(),
        to_binary(&QueryMsg::WithPermit { 
            permit:permit,
            query: AuthQuery::GetClaimReward { time: get_current_timestamp().unwrap() 
            } 
        }).unwrap());

    match query {
        Ok(_) => todo!(),
        Err(err) =>assert_eq!(StdError::GenericErr{ msg: "Querier contract error: staking::state::StakingInfo not found".to_string() }, err),
    }

    // MINT & DEPOSIT 
    mint_deposit_snipo20(&mut router,&lp_token_contract,&staking_contract,Uint128::new(1000));
    mint_deposit_snipo20(&mut router,&reward_contract,&staking_contract,Uint128::new(10000000));
    // STAKE LP TOKEN 
    send_snip20_to_stake(&mut router, &lp_token_contract, &staking_contract, Uint128::new(1000u128), &owner_address).unwrap();
     
    roll_blockchain(&mut router, 1000).unwrap();  
    let permit_query: QueryResponse = router.query_test(
        staking_contract.to_owned(),
        to_binary(&QueryMsg::WithPermit { 
            permit:mk_create_permit_data(OWNER_PUB_KEY, OWNER_SIGNATURE, &chain_id).unwrap(),
            query: AuthQuery::GetClaimReward { time: get_current_timestamp().unwrap() 
            } 
        }).unwrap()).unwrap();
   
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),1); 
           assert_eq!(claimable_rewards[0].amount, Uint128::new(1300810143255u128))          ;
        },
        _ => panic!("Query Responsedoes not match")
    } 

    let msg = ExecuteMsg::ClaimRewards {  }; 
    router.execute_contract(
        Addr::unchecked(OWNER.to_owned()),
        &staking_contract.clone(),
        &msg,
        &[], // 
    )
    .unwrap();      
   
    roll_blockchain(&mut router, 1).unwrap();
    // set 2 reward token
    let reward_contract_b = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
    let set_reward_msg = ExecuteMsg::SetRewardToken { 
        reward_token: ContractLink { 
            address:reward_contract_b.address.to_owned(), 
            code_hash: reward_contract_b.code_hash.to_owned() 
        }, 
        daily_reward_amount: Uint128::new(3000u128), 
        valid_to: Uint128::new(3747905010000u128) 
    };

    let _ = router.execute_contract(owner_address, &staking_contract, &set_reward_msg, &[]).unwrap();
    mint_deposit_snipo20(&mut router, &reward_contract_b, &staking_contract, Uint128::new(100000));

    roll_blockchain(&mut router, 500).unwrap();    
    let msg = ExecuteMsg::ClaimRewards {  }; 
    router.execute_contract(
        Addr::unchecked(OWNER.to_owned()),
        &staking_contract.clone(),
        &msg,
        &[], // 
    )
    .unwrap(); 
    
    let permit_query: QueryResponse = router.query_test(
        staking_contract.to_owned(),
        to_binary(&QueryMsg::WithPermit { 
            permit:mk_create_permit_data(OWNER_PUB_KEY, OWNER_SIGNATURE, &chain_id).unwrap(),
            query: AuthQuery::GetClaimReward { time: get_current_timestamp().unwrap() 
            } 
        }).unwrap()).unwrap();
   
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           //assert_eq!(claimable_rewards.len(),2); 
           assert_eq!(claimable_rewards[0].amount, Uint128::new(1300810141521u128));
           assert_eq!(claimable_rewards[1].amount, Uint128::new(1300810140654u128));  
        },
        _ => panic!("Query Responsedoes not match")
    } 

    // Assert New Staker
    



}




