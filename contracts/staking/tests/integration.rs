use staking::contract::{execute, instantiate, query};
// use lp_token::contract::{execute as lp_execute, instantiate as lp_instantiate, query as lp_query};

use secret_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};
// use multi_test::{auth_query::execute};

use shadeswap_shared::{   
    core::{ContractInstantiationInfo, ContractLink},
    c_std::{QueryRequest, WasmQuery},
    utils::testing::TestingExt
};
use shadeswap_shared::msg::staking::{{InitMsg, QueryResponse}};
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
pub const OWNER: &str = "secret1pf42ypa2awg0pxkx8lfyyrjvm28vq0qpffa8qx";

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn staking_integration_tests() {    
    use std::ptr::eq;
    use multi_test::help_lib::integration_help_lib::snip20_contract_store;
    use cosmwasm_std::{Uint128, from_binary, Coin, BlockInfo, Timestamp, Env, StdError, StdResult};
    use secret_multi_test::next_block;
    use shadeswap_shared::query_auth::QueryPermit;
    use shadeswap_shared::staking::{QueryMsg, AuthQuery};
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType, TokenPair}, snip20::{InstantiateMsg, InitConfig}, stake_contract::StakingContractInit};
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

    router.update_block(next_block);

    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();
    let snip20_contract_code_id = router.store_code(snip20_contract_store());
    let staking_contract = router.store_code(staking_contract_store());
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

    let mocked_contract_addr = router
        .instantiate_contract(
            staking_contract,
            mk_address(&OWNER).to_owned(),
            &init_msg,
            &[],
            "staking",
            Some(OWNER.to_owned()),
        ).unwrap();
    
    router.update_block(next_block);
    
    // Assert Staking Config
    let query: QueryResponse = router.query_test(mocked_contract_addr.to_owned(),to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryResponse::Config { reward_token, lp_token, daily_reward_amount, amm_pair } => {
           assert_eq!(daily_reward_amount, Uint128::new(30000u128));
           assert_eq!(lp_token.address.to_owned(), lp_token_contract.address.to_owned());
        },
        _ => panic!("Query Responsedoes not match")
    }

    router.update_block(next_block);

    let msg_get_claimable = to_binary(&AuthQuery::GetClaimReward { time: Uint128::new(1600000000u128) }).unwrap();
    // Assert Error StakingInfo not found
    let query: StdResult<QueryResponse> = router.query_test(
        mocked_contract_addr.to_owned(),
        to_binary(&QueryMsg::WithPermit { 
            permit:mk_create_permit_data().unwrap(),
            query: AuthQuery::GetClaimReward { time: get_current_timestamp().unwrap() 
            } 
        }).unwrap());

    match query {
        Ok(_) => todo!(),
        Err(err) =>assert_eq!(StdError::GenericErr{ msg: "Querier contract error: staking::state::StakingInfo not found".to_string() }, err),
    }

    // mint lp token for test
    deposit_snip20(&mut router,lp_token_contract.to_owned(),Uint128::new(10000000)).unwrap();
    mint_snip20(&mut router, Uint128::new(1000000),Addr::unchecked(OWNER),lp_token_contract.to_owned()).unwrap();
    send_snip20_with_msg(&mut router, &lp_token_contract, &mocked_contract_addr, Uint128::new(1000u128), &owner_address).unwrap();

    let msg_get_claimable = to_binary(&AuthQuery::GetClaimReward { time: Uint128::new(1600000000u128) }).unwrap();
    // Assert Error StakingInfo not found
    let permit_query: QueryResponse = router.query_test(
        mocked_contract_addr.to_owned(),
        to_binary(&QueryMsg::WithPermit { 
            permit:mk_create_permit_data().unwrap(),
            query: AuthQuery::GetClaimReward { time: get_current_timestamp().unwrap() 
            } 
        }).unwrap()).unwrap();
   
    match permit_query {
        QueryResponse::ClaimRewards { claimable_rewards  } => {
           assert_eq!(claimable_rewards.len(),0 );           
        },
        _ => panic!("Query Responsedoes not match")
    }  
}

