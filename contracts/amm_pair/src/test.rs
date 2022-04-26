use shadeswap_shared::msg::amm_pair::{{InitMsg,QueryMsg, HandleMsg, InvokeMsg, QueryMsgResponse}};
use shadeswap_shared::token_amount::{{TokenAmount}};
use shadeswap_shared::token_pair::{{TokenPair}};
use shadeswap_shared::token_pair_amount::{{TokenPairAmount}};
use shadeswap_shared::token_type::{{TokenType}};
use crate::state::{Config, store_config, load_config};
use crate::state::swapdetails::{SwapInfo, SwapResult};
use shadeswap_shared::{ 
    fadroma::{
        scrt::{
            from_binary, log, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Decimal, Env,
            Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryRequest, QueryResult,
            StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery, 
            secret_toolkit::snip20,        
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
            testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
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
        let ref mut deps = mock_dependencies(8, &[]);
        let mut env = mock_env(HumanAddr(String::from("TESTADDRESS")), &[]);
        env.block.height = 200_000;
        let amm_pair =  TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("TOKEN0".to_string()),
                token_code_hash: "TOKEN0_HASH".to_string()
            },            
            TokenType::CustomToken {
                    contract_addr: HumanAddr("TOKEN1".to_string()),
                    token_code_hash: "TOKEN1_HASH".to_string()
                }
        );

        let msg = InitMsg {
            pair: amm_pair,
            lp_token_contract: ContractInstantiationInfo{
                  code_hash: "CODE_HASH".to_string(),
                  id :0
            },
            factory_info: ContractLink {
                address: HumanAddr(String::from("FACTORYADDR")),
                code_hash: "FACTORYADDR_HASH".to_string()
            },
            prng_seed: to_binary(&"FSDFSDFSDFSDF".to_string())?,
            entropy: to_binary(&"REWRQWERWERWER".to_string())?,
            callback: Callback {
                contract: ContractLink {
                    address: HumanAddr(String::from("CALLBACKADDR")),
                    code_hash: "Test".to_string()
                },
                msg: to_binary(&String::from("Welcome bytes"))?
            },
            symbol: "WETH".to_string(),
        };
        let result = init(deps, env.clone(), msg);
        assert!(result.is_ok());
        Ok(())
    }


    // fn mkinitconfig(id: u64) -> Config<HumanAddr> {
    //     Config::init(InitMsg {
    //         pair_contract: ContractInstantiationInfo {
    //             id,
    //             code_hash: "2341586789".into(),
    //         },
    //         amm_settings: AMMSettings {
    //             lp: Fee::new(28, 10000),
    //             shadeswap_fee: Fee::new(2, 10000),
    //             shadeswap_burner: None,
    //         },
    //     })
    // }
}

pub fn mock_env() -> Env {
    Env {
      block: BlockInfo {
        height: 12_345,
        time: 1_571_797_419,
        time_nanos: 879305533,
        chain_id: "cosmos-testnet-14002".to_string(),
      },
      contract: ContractInfo {
        address: HumanAddr::from(MOCK_CONTRACT_ADDR),
      },
    }
  }