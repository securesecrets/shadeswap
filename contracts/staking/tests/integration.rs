use staking::contract::{execute, instantiate, query};
// use lp_token::contract::{execute as lp_execute, instantiate as lp_instantiate, query as lp_query};

use secret_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};
// use multi_test::{auth_query::execute};

use shadeswap_shared::{   
    core::{ContractInstantiationInfo, ContractLink},
    c_std::{QueryRequest, WasmQuery},
};
use shadeswap_shared::msg::staking::{{InitMsg, QueryResponse, ExecuteMsg}};
use multi_test::help_lib::integration_help_lib::{mk_contract_link, mk_address};
use cosmwasm_std::{
    testing::{mock_env, MockApi},
    to_binary, Addr, Empty, Binary, ContractInfo,
};

pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
    Box::new(contract)
} 

pub const CONTRACT_ADDRESS: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy6";
pub const TOKEN_A: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
pub const TOKEN_B: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy4";
pub const SENDER: &str = "secret13q9rgw3ez5mf808vm6k0naye090hh0m5fe2436";
pub const OWNER: &str = "secret138pqmt4gyyhjrtzj9vnf2k622d5cdvwucr423q";
pub const OWNER_PUB_KEY: &str = "At+anoZyimgcnRd4u+4a5v6yWrsLUpj2UPdHOftbaeD7";

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn staking_integration_tests() {    
    use std::ptr::eq;
    use multi_test::help_lib::integration_help_lib::{snip20_contract_store, roll_blockchain, print_events, snip20_send, get_snip20_balance};
    use cosmwasm_std::{Uint128, from_binary, Coin, BlockInfo, Timestamp, Env, StdError, StdResult};
    use query_authentication::transaction::{PubKey, PubKeyValue};
    use secret_multi_test::{next_block, AppResponse};
    use shadeswap_shared::query_auth::QueryPermit;
    use shadeswap_shared::snip20::manager::Balance;
    use shadeswap_shared::staking::{QueryMsg, AuthQuery};
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}, snip20::{InstantiateMsg, InitConfig}, stake_contract::StakingContractInit};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract, mint_snip20,deposit_snip20,
         send_snip20_with_msg, mk_create_permit_data, get_current_timestamp};
   
    let owner_address = Addr::unchecked(OWNER);
    let mut router = App::default();  
  
    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_address.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });

    roll_blockchain(&mut router, 1).unwrap();

    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
    let staking_contract_info = router.store_code(staking_contract_store());
    let lp_token_contract = generate_snip20_contract(&mut router, "LPT".to_string(),"LPT".to_string(),18).unwrap();
    let init_msg = InitMsg {
        daily_reward_amount: Uint128::new(30000u128),
        reward_token: TokenType::CustomToken { contract_addr:reward_contract.address.to_owned(), token_code_hash: reward_contract.code_hash.to_owned() },
        pair_contract: ContractLink { address: Addr::unchecked("AMMPAIR"), code_hash: "".to_string() },
        prng_seed: to_binary(&"password").unwrap(),
        lp_token: ContractLink { address:lp_token_contract.address.to_owned(), code_hash: lp_token_contract.code_hash.to_owned() },
        authenticator: None,
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
        QueryResponse::Config { reward_token, lp_token, daily_reward_amount, amm_pair } => {
           assert_eq!(daily_reward_amount, Uint128::new(30000u128));
           assert_eq!(lp_token.address.to_owned(), lp_token_contract.address.to_owned());
        },
        _ => panic!("Query Responsedoes not match")
    }

    router.update_block(next_block);

    // Assert Error StakingInfo not found
    let query: StdResult<QueryResponse> = router.query_test(
        staking_contract.to_owned(),
        to_binary(&QueryMsg::WithPermit { 
            permit:mk_create_permit_data(OWNER_PUB_KEY).unwrap(),
            query: AuthQuery::GetClaimReward { time: get_current_timestamp().unwrap() 
            } 
        }).unwrap());

    match query {
        Ok(_) => todo!(),
        Err(err) =>assert_eq!(StdError::GenericErr{ msg: "Querier contract error: staking::state::StakingInfo not found".to_string() }, err),
    }

    // LP TOKEN
    deposit_snip20(&mut router,lp_token_contract.to_owned(),Uint128::new(10000000)).unwrap();
    mint_snip20(&mut router, Uint128::new(1000000),Addr::unchecked(OWNER),lp_token_contract.to_owned()).unwrap();
    send_snip20_with_msg(&mut router, &lp_token_contract, &staking_contract, Uint128::new(1000u128), &owner_address).unwrap();

    // REWARD TOKEN
    deposit_snip20(&mut router,reward_contract.to_owned(),Uint128::new(10000000)).unwrap();
    mint_snip20(&mut router, Uint128::new(1000000),staking_contract.address.to_owned(),reward_contract.to_owned()).unwrap();   
    // TODO Fix GetClaimRewards 
    let permit_query: QueryResponse = router.query_test(
        staking_contract.to_owned(),
        to_binary(&QueryMsg::WithPermit { 
            permit:mk_create_permit_data("At+anoZyimgcnRd4u+4a5v6yWrsLUpj2UPdHOftbaeD7").unwrap(),
            query: AuthQuery::GetClaimReward { time: get_current_timestamp().unwrap() 
            } 
        }).unwrap()).unwrap();
   
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),0 );           
        },
        _ => panic!("Query Responsedoes not match")
    }  

    roll_blockchain(&mut router, 10000).unwrap();  
    let permit_query: QueryResponse = router.query_test(
        staking_contract.to_owned(),
        to_binary(&QueryMsg::WithPermit { 
            permit:mk_create_permit_data(OWNER_PUB_KEY).unwrap(),
            query: AuthQuery::GetClaimReward { time: get_current_timestamp().unwrap() 
            } 
        }).unwrap()).unwrap();
   
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),0 );           
        },
        _ => panic!("Query Responsedoes not match")
    } 

    let msg = ExecuteMsg::ClaimRewards {  };
 
    let response: AppResponse = router.execute_contract(
        Addr::unchecked(OWNER.to_owned()),
        &staking_contract.clone(),
        &msg,
        &[], // 
    )
    .unwrap();   
    print_events(response);
   
    roll_blockchain(&mut router, 1).unwrap();
    let user_balance = get_snip20_balance(&mut router, &reward_contract, OWNER.to_owned(), "".to_string());
    assert_ne!(user_balance, Uint128::zero());
}


