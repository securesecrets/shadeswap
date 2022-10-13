use staking::contract::{execute, instantiate, query};
use snip20_reference_impl::contract::{execute as snip20_execute, instantiate as snip20_instantiate, query as  snip20_query};
// use lp_token::contract::{execute as lp_execute, instantiate as lp_instantiate, query as lp_query};

use secret_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};
use shadeswap_shared::{   
    core::{ContractInstantiationInfo, ContractLink},
    c_std::{QueryRequest, WasmQuery},
    utils::testing::TestingExt
};
use shadeswap_shared::msg::staking::{{InitMsg, QueryResponse}};
use crate::integration_help_lib::{mk_contract_link, mk_address};
use cosmwasm_std::{
    testing::{mock_env, MockApi},
    to_binary, Addr, Empty, Binary, ContractInfo,
};


pub fn staking_contract_store() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
    Box::new(contract)
} 

pub fn snip20_contract_store() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(snip20_execute, snip20_instantiate, snip20_query);
    Box::new(contract)
} 

// pub fn lp_token_contract_store() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new_with_empty(lp_execute, lp_instantiate, lp_query); //.with_reply(reply);
//     Box::new(contract)
// } 

pub const CONTRACT_ADDRESS: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy6";
pub const TOKEN_A: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy2";
pub const TOKEN_B: &str = "secret12qmz6uuapxgz7t0zed82wckl4mff5pt5czcmy4";
pub const FACTORY: &str = "secret13q9rgw3ez5mf808vm6k0naye090hh0m5fe2436";
pub const OWNER: &str = "secret1pf42ypa2awg0pxkx8lfyyrjvm28vq0qpffa8qx";

#[cfg(not(target_arch = "wasm32"))]
#[test]
pub fn staking_integration_tests() {    
    use cosmwasm_std::{Uint128, from_binary};
    use shadeswap_shared::staking::QueryMsg;
    use shadeswap_shared::utils::testing::TestingExt;
    use shadeswap_shared::{core::{TokenType, TokenPair}, snip20::{InstantiateMsg, InitConfig}, stake_contract::StakingContractInit};
    use crate::integration_help_lib::{generate_snip20_contract, mint_snip20};

    let mut router = App::default();   
    let reward_contract = generate_snip20_contract(&mut router, "RWD".to_string(),"RWD".to_string(),18).unwrap();
    let snip20_contract_code_id = router.store_code(snip20_contract_store());
    let staking_contract = router.store_code(staking_contract_store());
    let lp_token_contract = generate_snip20_contract(&mut router, "LPT".to_string(),"LPT".to_string(),18).unwrap();
    let init_msg = InitMsg {
        daily_reward_amount: Uint128::new(50u128),
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
            None,
        ).unwrap();

    // mint lp token for test
    mint_snip20(&mut router, Uint128::new(1000),Addr::unchecked(OWNER),lp_token_contract.to_owned()).unwrap();

    println!("{}", mocked_contract_addr.address.to_string());
    let query: QueryResponse = router.query_test(mocked_contract_addr,to_binary(&QueryMsg::GetConfig { }).unwrap()).unwrap();
    match query {
        QueryResponse::Config { reward_token, lp_token, daily_reward_amount, amm_pair } => {
           assert_eq!(daily_reward_amount, Uint128::new(50u128));
           assert_eq!(lp_token.address.to_owned(), lp_token_contract.address.to_owned());
        },
        _ => panic!("Query Responsedoes not match")
    }
}



pub mod integration_help_lib{   
    use cosmwasm_std::{Addr, ContractInfo, StdResult, Uint128};
    use secret_multi_test::{App, Executor};
    use shadeswap_shared::{msg::staking::{InitMsg, InvokeMsg}, core::TokenPair, core::{TokenType, ContractLink}, snip20::{InitConfig, InstantiateMsg}};
    use crate::{{TOKEN_A, TOKEN_B}, OWNER, snip20_contract_store};      
    use cosmwasm_std::to_binary;

    pub fn mk_token_pair() -> TokenPair{
        return TokenPair(
            TokenType::CustomToken { contract_addr: mk_address(TOKEN_A), token_code_hash: "".to_string() },
            TokenType::CustomToken { contract_addr: mk_address(TOKEN_B), token_code_hash: "".to_string() }
        );
    }

    pub fn mk_address(address: &str) -> Addr{
        return Addr::unchecked(address.to_string())
    }

    pub fn mk_contract_link(address: &str) -> ContractLink{
        return ContractLink{
            address: mk_address(address),
            code_hash: "".to_string(),
        }       
    }

    pub fn send_snip20(
        contract: &ContractInfo,
        stake_contract: &ContractInfo,
        lp_token: Uint128
    ) -> StdResult<()>{
        let invoke_msg = to_binary(&InvokeMsg::Stake {
            from: info.sender.clone(),
        })?;
        // SEND LP Token to Staking Contract with Staking Message
        let msg = to_binary(&snip20_reference_impl::msg::ExecuteMsg::Send {
            recipient: stake_contract.address.to_string(),
            recipient_code_hash: Some(stake_contract.code_hash.clone()),
            amount: lp_token,
            msg: Some(invoke_msg),
            memo: None,
            padding: None,
        })?;

        let _ = router.execute_contract(
            Addr::unchecked(OWNER.to_owned()),
            &contract.clone(),
            &msg,
            &[],
        )
        .unwrap();

        Ok(())
    }

    pub fn mint_snip20(
        router: &mut App, 
        amount: Uint128,
        recipient: Addr,
        contract: ContractInfo
    ) -> StdResult<()>{
        let msg = snip20_reference_impl::msg::ExecuteMsg::Mint { 
            recipient: recipient, 
            amount: amount, 
            memo: None, 
            padding: None 
        };

        let _ = router.execute_contract(
            Addr::unchecked(OWNER.to_owned()),
            &contract.clone(),
            &msg,
            &[],
        )
        .unwrap();

        Ok(())
    }

    // pub fn send_amount_snip20() -> StdResult<()>{
        
    // } 
    
    pub fn generate_snip20_contract(
        router: &mut App, 
        name: String, 
        symbol: String, 
        decimal: u8) -> StdResult<ContractInfo> {

        let snip20_contract_code_id = router.store_code(snip20_contract_store());        
        let init_snip20_msg = InstantiateMsg {
            name: name.to_string(),
            admin: Some(OWNER.to_string()),
            symbol: symbol.to_string(),
            decimals: decimal,
            initial_balances: None,
            prng_seed: to_binary("password")?,
            config: Some(InitConfig {
                public_total_supply: Some(true),
                enable_deposit: Some(true),
                enable_redeem: Some(false),
                enable_mint: Some(true),
                enable_burn: Some(true),
                enable_transfer: Some(true),
            }),
            query_auth: None,
        };
        let init_snip20_code_id = router
            .instantiate_contract(
                snip20_contract_code_id,
                mk_address(&OWNER).to_owned(),
                &init_snip20_msg,
                &[],
                "token_a",
                None,
            ).unwrap();
        Ok(init_snip20_code_id)
    }


}