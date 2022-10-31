use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::to_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::Deps;
use cosmwasm_std::DepsMut;
use cosmwasm_std::Env;
use cosmwasm_std::{Api, Binary, CanonicalAddr, Querier, StdError, StdResult, Storage};
use shadeswap_shared::amm_pair::AMMSettings;
use shadeswap_shared::core::Fee;
use shadeswap_shared::core::{ContractInstantiationInfo, ContractLink};
use shadeswap_shared::msg::factory::InitMsg;
pub use shadeswap_shared::{msg::factory::QueryResponse, Pagination};

use crate::state::Config;

#[cfg(test)]
pub mod test_contract {
    use super::*;
    use crate::contract::execute;
    use crate::contract::instantiate;
    use crate::contract::query;
    use crate::operations::create_pair;
    use crate::operations::create_signature;
    use crate::state::EPHEMERAL_STORAGE_KEY;
    use crate::state::NextPairKey;
    use crate::state::config_r;
    use crate::state::PAGINATION_LIMIT;
    use crate::state::config_w;
    use crate::state::ephemeral_storage_r;
    use crate::state::ephemeral_storage_w;
    use cosmwasm_std::from_binary;
    use cosmwasm_std::Addr;
    use cosmwasm_std::MessageInfo;
    use cosmwasm_std::testing::mock_info;
    use shadeswap_shared::amm_pair::AMMPair;
    use shadeswap_shared::core::TokenPair;
    use shadeswap_shared::core::TokenType;
    use shadeswap_shared::msg::factory::ExecuteMsg;
    use shadeswap_shared::msg::factory::QueryMsg;
    pub use shadeswap_shared::{msg::factory::QueryResponse, Pagination};

    #[test]
    fn init_ok() -> StdResult<()> {
        let mut deps = mock_dependencies();
        let config = mkconfig(0);
        let env = mock_env();
        assert!(instantiate(
            deps.as_mut(),
            env,
            MessageInfo {
                sender: Addr::unchecked("admin"),
                funds: vec![]
            },
            create_init_msg_from_config(&config)
        )
        .is_ok());
        assert_eq!(config, config_r(deps.as_ref().storage).load()?);
        Ok(())
    }

    #[test]
    fn get_set_config_ok() -> StdResult<()> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        instantiate(
            deps.as_mut(),
            env.clone(),
            MessageInfo {
                sender: Addr::unchecked("admin"),
                funds: vec![],
            },
            create_init_msg_from_config(&mkconfig(0)),
        )?;

        let new_config = mkconfig(5);
        execute(
            deps.as_mut(),
            env,
            MessageInfo {
                sender: Addr::unchecked("admin"),
                funds: vec![],
            },
            ExecuteMsg::SetConfig {
                pair_contract: Some(new_config.pair_contract.clone()),
                amm_settings: Some(new_config.amm_settings.clone()),
                lp_token_contract: Some(new_config.lp_token_contract.clone()),
                api_key: Some("api_key".to_string()),
            },
        )
        .unwrap();

        let response: QueryResponse = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {})?)?;
        let compare: QueryResponse = create_query_response_from_config(&new_config);
        assert_eq!(compare, response);
        Ok(())
    }

    #[test]
    fn register_amm_pair_ok() -> StdResult<()> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let config = mkconfig(0);

        config_w(deps.as_mut().storage).save(&config)?;
        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked("token_addr".to_string()),
                token_code_hash: "13123adasd".to_string(),
            },
            TokenType::NativeToken {
                denom: "test1".to_string(),
            },
        );

        let signature = create_signature(&env, &mock_info("admin", &[]))?;
        ephemeral_storage_w(&mut deps.storage).save(&NextPairKey {
            pair: pair.clone(),
            is_verified: true,
            key: signature.clone(),
        })?;

        execute(
            deps.as_mut(),
            env,
          mock_info("admin", &[]),
            ExecuteMsg::RegisterAMMPair {
                pair: pair.clone(),
                signature,
            },
        )?;

        let result = ephemeral_storage_r(&deps.storage).load();
        match result {
            Ok(_) => todo!(),
            Err(err) => {
                assert_eq!("factory::state::NextPairKey not found", &err.to_string())
            },
        }       
        Ok(())
    }

    #[test]
    fn create_pair_ok() -> StdResult<()> {
        let ref mut deps = mock_dependencies();
        let env = mock_env();
        let config = mkconfig(0);
        assert!(instantiate(
            deps.as_mut(),
            env,
            MessageInfo {
                sender: Addr::unchecked("admin"),
                funds: vec![]
            },
            create_init_msg_from_config(&config)
        )
        .is_ok());

        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: Addr::unchecked("token_addr"),
                token_code_hash: "diff".to_string(),
            },
            TokenType::CustomToken {
                contract_addr: Addr::unchecked("token_addr"),
                token_code_hash: "13123adasd".to_string(),
            },
        );

        let result = create_pair(
            deps.as_mut(),
             mock_env(), 
             &mock_info("admin", &[]),
             pair,  
             Addr::unchecked("admin"),
             to_binary(&"entropy").unwrap(), 
             None,
            None);

        assert!(result.is_ok());
        Ok(())
    }
    #[test]
    fn add_amm_pairs() {
        let ref mut deps = mock_dependencies();
        let config = mkconfig(0);
        let env = mock_env();

        instantiate(deps.as_mut(), env.clone(), MessageInfo {
            sender: Addr::unchecked("admin"),
            funds: vec![]
        }, create_init_msg_from_config(&config)).unwrap();

        let mut amm_pairs: Vec<AMMPair> = vec![];

        for i in 0..5 {
            amm_pairs.push(AMMPair {
                pair: TokenPair(
                    TokenType::CustomToken {
                        contract_addr: Addr::unchecked(format!("token_0_addr_{}", i)),
                        token_code_hash: format!("token_0_hash_{}", i),
                    },
                    TokenType::CustomToken {
                        contract_addr: Addr::unchecked(format!("token_1_addr_{}", i)),
                        token_code_hash: format!("token_1_hash_{}", i),
                    },
                ),
                address: Addr::unchecked(format!("pair_addr_{}", i)),
                enabled: true,
            });
        }

        execute(
            deps.as_mut(),
            env,
            MessageInfo {
                sender: Addr::unchecked("admin"),
                funds: vec![]
            },
            ExecuteMsg::AddAMMPairs {
                amm_pairs: amm_pairs.clone()[0..].into(),
            },
        )
        .unwrap();

        let result = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ListAMMPairs {
                pagination: pagination(0, PAGINATION_LIMIT),
            },
        )
        .unwrap();

        let response: QueryResponse = from_binary(&result).unwrap();

        match response {
            QueryResponse::ListAMMPairs { amm_pairs: stored } => {
                assert_eq!(amm_pairs, stored)
            }
            _ => panic!("QueryResponse::ListExchanges"),
        }
    }
}

pub fn create_init_msg_from_config(config: &Config) -> InitMsg{
    InitMsg {
        pair_contract: config.pair_contract.clone(),
        amm_settings: AMMSettings {
            lp_fee: Fee::new(28, 10000),
            shade_dao_fee: Fee::new(2, 10000),
            shade_dao_address: ContractLink {
                address: Addr::unchecked("CALLBACKADDR"),
                code_hash: "Test".to_string(),
            },
        },
        lp_token_contract: config.lp_token_contract.clone(),
        prng_seed: to_binary(&"prng").unwrap(),
        api_key: "api_key".to_string(),
        authenticator: None,
    }
}

pub fn create_query_response_from_config(config: &Config) ->QueryResponse {
    QueryResponse::GetConfig {
        pair_contract: config.pair_contract.clone(),
        amm_settings: config.amm_settings.clone(),
        lp_token_contract: config.lp_token_contract.clone(),
        authenticator: None,
    }
}


// pub mod test_state {
//     use shadeswap_shared::{amm_pair::AMMPair, core::Canonize};

//     use super::*;

//     fn swap_pair<A: Clone>(pair: &TokenPair) -> TokenPair {
//         TokenPair(pair.1.clone(), pair.0.clone())
//     }

//     #[test]
//     fn generate_pair_key_ok() -> StdResult<()> {
//         fn cmp_pair<S: Storage, A: Api, Q: Querier>(
//             deps: &Deps<S, A, Q>,
//             pair: TokenPair<HumanAddr>,
//         ) -> StdResult<()> {
//             let stored_pair = pair.clone().canonize(&deps.api)?;
//             let key = generate_pair_key(&stored_pair);

//             let pair = swap_pair(&pair.clone());

//             let stored_pair = pair.canonize(&deps.api)?;
//             let swapped_key = generate_pair_key(&stored_pair);

//             assert_eq!(key, swapped_key);

//             Ok(())
//         }

//         let ref deps = mock_dependencies();

//         cmp_pair(
//             deps,
//             TokenPair(
//                 TokenType::CustomToken {
//                     contract_addr: Addr::unchecked("first_addr".to_string()),
//                     token_code_hash: "13123adasd".to_string(),
//                 },
//                 TokenType::CustomToken {
//                     contract_addr: Addr::unchecked("scnd_addr".to_string()),
//                     token_code_hash: "4534qwerqqw".to_string(),
//                 },
//             ),
//         )?;

//         cmp_pair(
//             deps,
//             TokenPair(
//                 TokenType::NativeToken {
//                     denom: "test1".to_string(),
//                 },
//                 TokenType::NativeToken {
//                     denom: "test2".to_string(),
//                 },
//             ),
//         )?;

//         cmp_pair(
//             deps,
//             TokenPair(
//                 TokenType::NativeToken {
//                     denom: "test3".to_string(),
//                 },
//                 TokenType::CustomToken {
//                     contract_addr: Addr::unchecked("third_addr".to_string()),
//                     token_code_hash: "asd21312asd".to_string(),
//                 },
//             ),
//         )?;

//         Ok(())
//     }

//     #[test]
//     fn store_and_get_amm_pairs_ok() {
//         let ref mut deps = mock_dependencies();
//         let mut amm_pairs: Vec<AMMPair> = vec![];
//         amm_pairs.push(AMMPair {
//             pair: TokenPair(
//                 TokenType::CustomToken {
//                     contract_addr: format!("token_0_addr_{}", 0).to_string(),
//                     token_code_hash: format!("token_0_hash_{}", 0),
//                 },
//                 TokenType::CustomToken {
//                     contract_addr: format!("token_1_addr_{}", 0).to_string(),
//                     token_code_hash: format!("token_1_hash_{}", 0),
//                 },
//             ),
//             address: format!("pair_addr_{}", 0).to_string(),
//         });
//         save_amm_pairs(deps, amm_pairs.clone()).unwrap();
//         let result = load_amm_pairs(deps, pagination(0, 1)).unwrap();

//         //Check Count was updated
//         assert_eq!(1, load_amm_pairs_count(&mut deps.storage).unwrap());

//         //Check number of result was returned
//         assert_eq!(1, result.len());

//         //Match result
//         assert_eq!(amm_pairs[0], result[0]);
//     }

//     #[test]
//     fn save_and_load_amm_pairs_count_ok() {
//         let ref mut deps = mock_dependencies();
//         save_amm_pairs_count(&mut deps.storage, 1).unwrap();
//         assert_eq!(1, load_amm_pairs_count(&mut deps.storage).unwrap());
//         assert_ne!(2, load_amm_pairs_count(&mut deps.storage).unwrap())
//     }
// }

fn mkconfig(id: u64) -> Config {
    Config::from_init_msg(InitMsg {
        pair_contract: ContractInstantiationInfo {
            id,
            code_hash: "2341586789".to_string(),
        },
        amm_settings: AMMSettings {
            lp_fee: Fee::new(28, 10000),
            shade_dao_fee: Fee::new(2, 10000),
            shade_dao_address: ContractLink {
                address: Addr::unchecked("CALLBACKADDR"),
                code_hash: "Test".to_string(),
            },
        },
        lp_token_contract: ContractInstantiationInfo {
            id,
            code_hash: "123".to_string(),
        },
        prng_seed: to_binary(&"prng").unwrap(),
        api_key: "api_key".to_string(),
        authenticator: None,
    })
}

fn pagination(start: u64, limit: u8) -> Pagination {
    Pagination { start, limit }
}
