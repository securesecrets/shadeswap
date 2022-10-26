use secret_multi_test::{App, Contract, ContractWrapper, Executor};
use shadeswap_shared::msg::amm_pair::{{InitMsg,  ExecuteMsg}};
use multi_test::help_lib::integration_help_lib::{mk_contract_link, mk_address};
use cosmwasm_std::{
    testing::{mock_env, MockApi},
    to_binary, Addr, Empty, Binary, ContractInfo, Uint128,
};

use shadeswap_shared::utils::asset::Contract as AuthContract;

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn amm_pair_integration_tests() {    
    use amm_pair::contract::{instantiate, query, execute};
    use multi_test::help_lib::integration_help_lib::{roll_blockchain, mint_deposit_snip20, send_snip20_to_stake, snip20_send, increase_allowance, get_current_block_time, store_init_staking_contract, store_init_factory_contract, snip20_contract_store, staking_contract_store};
    use cosmwasm_std::{Uint128, Coin, StdError, StdResult, Timestamp, from_binary};
    use multi_test::util_addr::util_addr::{OWNER, OWNER_SIGNATURE, OWNER_PUB_KEY, STAKER_A, STAKER_B, PUB_KEY_STAKER_A};       
    use multi_test::util_addr::util_blockchain::CHAIN_ID;
    use shadeswap_shared::core::{ContractLink, ContractInstantiationInfo, TokenPair};
    use shadeswap_shared::staking::StakingContractInit;
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType}};
    use multi_test::help_lib::integration_help_lib::{generate_snip20_contract};
    use multi_test::help_lib::integration_help_lib::print_events;

    let staker_a_addr = Addr::unchecked(STAKER_A.to_owned());       
    let staker_b_addr = Addr::unchecked(STAKER_B.to_owned());       
    let owner_addr = Addr::unchecked(OWNER);   
    let mut router = App::default();  

    pub fn amm_contract_store() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
        Box::new(contract)
    }

    router.init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &owner_addr.clone(), vec![Coin{denom: "uscrt".into(), amount: Uint128::new(100000000000000u128)}])
            .unwrap();
    });
    router.block_info().chain_id = CHAIN_ID.to_string();
    roll_blockchain(&mut router, 1).unwrap();

    // GENERATE TOKEN PAIRS + FACTORY + STAKING 
    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();    
    let token_0_contract = generate_snip20_contract(&mut router, "ETH".to_string(),"ETH".to_string(),18).unwrap();    
    let token_1_contract = generate_snip20_contract(&mut router, "USDT".to_string(),"USDT".to_string(),18).unwrap();    
    let lp_conract_info = router.store_code(snip20_contract_store());
    let staking_contract_info = router.store_code(staking_contract_store());
    let factory_contract_info = store_init_factory_contract(&mut router).unwrap();
    let amm_pairs_info = router.store_code(amm_contract_store());

    // INIT AMM PAIR
    let init_msg = InitMsg { 
        pair: TokenPair(
            TokenType::CustomToken { 
                contract_addr: token_0_contract.address.to_owned(), 
                token_code_hash: token_0_contract.code_hash.to_owned(), 
            },
            TokenType::CustomToken { 
                contract_addr: token_1_contract.address.to_owned(), 
                token_code_hash: token_1_contract.code_hash.to_owned(), 
            },
        ), 
        lp_token_contract: ContractInstantiationInfo { 
            code_hash: lp_conract_info.code_hash.to_owned(), 
            id: lp_conract_info.to_owned().code_id 
        },
        factory_info: ContractLink { 
            address:factory_contract_info.address,
            code_hash: factory_contract_info.code_hash
        }, 
        prng_seed: to_binary("password").unwrap(), 
        entropy: to_binary("password").unwrap(),  
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

    let amm_pair_contract = router
        .instantiate_contract(
            amm_pairs_info,
            owner_addr.to_owned(),
            &init_msg,
            &[],
            "amm_pair",
            Some(OWNER.to_string()),
        ).unwrap();
}

