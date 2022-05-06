use shadeswap_shared::msg::amm_pair::{{InitMsg,QueryMsg, HandleMsg, InvokeMsg, QueryMsgResponse}};
use shadeswap_shared::token_amount::{{TokenAmount}};
use shadeswap_shared::token_pair::{{TokenPair}};
use shadeswap_shared::token_pair_amount::{{TokenPairAmount}};
use shadeswap_shared::token_type::{{TokenType}};
use shadeswap_shared::amm_pair::{{AMMPair, AMMSettings, Fee}};
use crate::state::{Config, store_config, load_config};
use crate::state::swapdetails::{SwapInfo, SwapResult};
use crate::contract::{{create_viewing_key, calculate_price, swap_tokens, initial_swap}};
use std::hash::Hash;
use shadeswap_shared::{ 
    fadroma::{
        scrt::{
            from_binary, log, to_binary, Api, BankMsg, Binary, MessageInfo, ContractInfo, Coin, CosmosMsg, Decimal, Env,
            Extern, HandleResponse, HumanAddr, InitResponse,  Querier, QueryRequest, QueryResult,
            StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery, 
            secret_toolkit::snip20,  BlockInfo
        },
        scrt_uint256::Uint256,
        scrt_callback::Callback,
        scrt_link::{ContractLink, ContractInstantiationInfo},
        scrt_vk::ViewingKey,
    },
 
};

use shadeswap_shared::{
    fadroma::{
        scrt::{
            testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR},
        },
    }
};

use composable_snip20::msg::{{InitMsg as Snip20ComposableMsg, InitConfig as Snip20ComposableConfig}};

#[cfg(test)]
mod amm_pair_test_contract {
    use super::*;
    use crate::contract::init;

    #[test]
    fn assert_init_config() -> StdResult<()> {       
        // let info = mock_info("amm_pair_contract", &amount);
        let seed = to_binary(&"SEED".to_string())?;
        let entropy = to_binary(&"ENTROPY".to_string())?;

        let ref mut deps = mock_dependencies(30, &[]);
        let mut env = mkenv("test");
        env.block.height = 200_000;
        env.contract.address = HumanAddr("ContractAddress".to_string());
        let token_pair = mk_token_pair("token0".to_string(), "token1".to_string());
        let msg = InitMsg {
            pair: token_pair,
            lp_token_contract: ContractInstantiationInfo{
                  code_hash: "CODE_HASH".to_string(),
                  id :0
            },
            factory_info: ContractLink {
                address: HumanAddr(String::from("FACTORYADDR")),
                code_hash: "FACTORYADDR_HASH".to_string()
            },
            prng_seed: seed.clone(),
            entropy: entropy.clone(),
            callback: Callback {
                contract: ContractLink {
                    address: HumanAddr(String::from("CALLBACKADDR")),
                    code_hash: "Test".to_string()
                },
                msg: to_binary(&String::from("Welcome bytes"))?
            },
            symbol: "WETH".to_string(),
        };     
        assert!(init(deps, env.clone(), msg).is_ok());
      
 
        let test_view_key = create_viewing_key(&env,seed.clone(),entropy.clone());
        // load config
        let config = load_config(deps).unwrap();
        assert_eq!("WETH".to_string(), config.symbol);
        assert_eq!(test_view_key, config.viewing_key);
        Ok(())
    }

    #[test]
    fn assert_calculate_price() -> StdResult<()>{     
        let price = calculate_price(Uint256::from(2000), Uint256::from(10000), Uint256::from(100000));
        assert_eq!(Uint256::from(196), price?);
        Ok(())
    }

   /* #[test]
    fn assert_initial_swap_with_wrong_token_exception() -> StdResult<()>{     
        let token_pair = mk_token_pair("TOKEN0".to_string(), "TOKEN1".to_string());
        let amm_settings = mk_amm_settings();
        let offer_amount: u128 = 34028236692093846346337460;
        let expected_return_amount: u128 = 34028236692093846346337460;
        let expected_amount: u128 = 34028236692093846346337460;
        let deps = mkdeps();
        let env = mkenv("sender");
        let swap_result = initial_swap(
            &deps.querier, 
            &amm_settings, 
            &mock_config(env)?,
            &mk_custom_token_amount("", Uint128::from(offer_amount)), 
        );

        assert_eq!(Uint128::from(expected_amount), swap_result?.result.return_amount);
        Ok(())
    }*/
}

fn mkenv(sender: impl Into<HumanAddr>) -> Env {
    mock_env(sender, &[])
}

fn mkdeps() -> Extern<impl Storage, impl Api, impl Querier> {
    mock_dependencies(30, &[])
}

fn mk_token_pair(token0: String, token1: String) -> TokenPair<HumanAddr>{
    let pair =  TokenPair(
        TokenType::CustomToken {
            contract_addr: HumanAddr(token0.clone()),
            token_code_hash: "TOKEN0_HASH".to_string()
        },            
        TokenType::CustomToken {
            contract_addr: HumanAddr(token1.clone()),
            token_code_hash: "TOKEN1_HASH".to_string()
        }
    );
    pair
}

fn mk_custom_token_amount(address: &str, amount: Uint128) -> TokenAmount<HumanAddr>{  
    let token = TokenAmount{
        token: mk_custom_token(address.to_string()),
        amount: amount.clone(),
    };
    token
}

fn mk_custom_token(address: String) -> TokenType<HumanAddr>{
    let copy_address = HumanAddr(address.clone());
    let token = TokenType::CustomToken{
        contract_addr : copy_address,
        token_code_hash : address.clone(),
    };
    token
}

fn mk_amm_settings() -> AMMSettings<HumanAddr>{
    AMMSettings{
        lp_fee: Fee{
            nom: 3,
            denom: 18
        },
        shade_dao_fee: Fee {
            nom: 1,
            denom: 18
        },
        shade_dao_address: ContractLink{
            code_hash: "CODEHAS".to_string(),
            address: HumanAddr("TEST".to_string())
        }
    }
}

// pub fn mock_env_for_swap(code_hash: String, contract_key: String) -> Env {
//   Env {
//     contract_key: Some(contract_key.to_string()),
//     contract_code_hash: code_hash.to_string(),
//     block: BlockInfo {
//       height: 12_345,
//       time: 1_571_797_419,
//       chain_id: "pulsar-2".to_string(),
//     },
//     message: mock_info_for_swap("sender", &[]),
//     contract: mock_contract_info(MOCK_CONTRACT_ADDR)
//   }
// }

// pub fn mock_info_for_swap<U: Into<HumanAddr>>(sender: U, sent: &[Coin]) -> MessageInfo {
//     MessageInfo {
//       sender: sender.into(),
//       sent_funds: sent.to_vec(),
//     }
//   }

fn mock_config(env: Env) -> StdResult<Config<HumanAddr>>
{    
    let seed = to_binary(&"SEED".to_string())?;
    let entropy = to_binary(&"ENTROPY".to_string())?;

    Ok(Config {
        symbol:  "WETH".to_string(),
        factory_info: mock_contract_link("FACTORY".to_string()),
        lp_token_info: mock_contract_link("LPTOKEN".to_string()),
        pair:      mk_token_pair("TOKEN0".to_string(), "TOKEN1".to_string()),
        contract_addr: HumanAddr::from(MOCK_CONTRACT_ADDR),
        viewing_key:  create_viewing_key(&env, seed.clone(), entropy.clone()),
    })
}

pub fn mock_contract_link(address: String)-> ContractLink<HumanAddr>{
    ContractLink{
        address: HumanAddr::from(address.clone()),
        code_hash: "CODEHASH".to_string()
    }
}

fn mock_contract_info(address: &str) -> ContractInfo{
    ContractInfo{
        address :HumanAddr::from(address.clone())
    }
}

// pub fn mock_dependencies_for_swap(
//     contract_balance: &[Coin],
//   ) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
//     let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
//     OwnedDeps {
//       storage: MockStorage::default(),
//       api: MockApi::default(),
//       querier: MockQuerier::new(&[(&contract_addr, contract_balance)]),
//     }
//   }