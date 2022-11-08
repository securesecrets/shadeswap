use cosmwasm_std::Coin;
use cosmwasm_std::Empty;
use cosmwasm_std::OwnedDeps;
use cosmwasm_std::QuerierResult;
use cosmwasm_std::QueryRequest;
use cosmwasm_std::WasmQuery;
use cosmwasm_std::from_slice;
use cosmwasm_std::testing::MockApi;
use cosmwasm_std::testing::MockStorage;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::to_binary;
use cosmwasm_std::Addr;

use shadeswap_shared::contract_interfaces::admin::ValidateAdminPermissionResponse;
use cosmwasm_std::{Querier, StdResult};
use serde::Deserialize;
use serde::Serialize;
use shadeswap_shared::utils::asset::Contract;
use shadeswap_shared::amm_pair::AMMSettings;
use shadeswap_shared::core::Fee;
use shadeswap_shared::core::{ContractInstantiationInfo};
use shadeswap_shared::msg::factory::InitMsg;
pub use shadeswap_shared::{msg::factory::QueryResponse, Pagination};
use shadeswap_shared::snip20::manager::Balance;
use crate::state::Config;

#[cfg(test)]
pub mod test_contract {
    use super::*;
    use crate::contract::execute;
    use crate::contract::instantiate;
    use crate::contract::query;
    use crate::operations::create_pair; 
    use crate::state::config_r;
    use crate::state::PAGINATION_LIMIT;
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
        let mut deps = mock_dependencies(&[]);
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
        let mut deps = mock_dependencies(&[]);
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
                admin_auth: None,
            },
        )
        .unwrap();

        let response: QueryResponse = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {})?)?;
        let compare: QueryResponse = create_query_response_from_config(&new_config);
        assert_eq!(compare, response);
        Ok(())
    }

    #[test]
    fn create_pair_ok() -> StdResult<()> {
        let ref mut deps = mock_dependencies(&[]);
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
             pair,
             to_binary(&"entropy").unwrap(), 
             None,);

        assert!(result.is_ok());
        Ok(())
    }
    #[test]
    fn add_amm_pairs() {
        let ref mut deps = mock_dependencies(&[]);
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
            shade_dao_address: Contract {
                address: Addr::unchecked("CALLBACKADDR"),
                code_hash: "Test".to_string(),
            },
        },
        lp_token_contract: config.lp_token_contract.clone(),
        prng_seed: to_binary(&"prng").unwrap(),
        api_key: "api_key".to_string(),
        authenticator: None,
        admin_auth: shadeswap_shared::Contract { 
            address: Addr::unchecked("admin"), 
            code_hash: "".to_string()
        },
    }
}

pub fn create_query_response_from_config(config: &Config) ->QueryResponse {
    QueryResponse::GetConfig {
        pair_contract: config.pair_contract.clone(),
        amm_settings: config.amm_settings.clone(),
        lp_token_contract: config.lp_token_contract.clone(),
        authenticator: None,
        admin_auth: config.admin_auth.clone(),
    }
}

    pub fn mock_dependencies(
        _contract_balance: &[Coin],
    ) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier { _portion: 100 },
            custom_query_type: std::marker::PhantomData,
        }
    }

    
    #[derive(Serialize, Deserialize)]
    struct IntBalanceResponse {
        pub balance: Balance,
    }

    pub struct MockQuerier {
        _portion: u128,
    }    
    
    impl Querier for MockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
            match &request {
                QueryRequest::Wasm(msg) => match msg {                  
                    WasmQuery::Smart { contract_addr, code_hash: _, msg: _} => { 
                        match contract_addr.as_str() {
                            "admin"  => {
                                QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(to_binary(&ValidateAdminPermissionResponse{
                                    has_permission: true,
                                }).unwrap()))
                            },
                            _ => unimplemented!(),
                        }
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }
    }

fn mkconfig(id: u64) -> Config {
    Config::from_init_msg(InitMsg {
        pair_contract: ContractInstantiationInfo {
            id,
            code_hash: "2341586789".to_string(),
        },
        amm_settings: AMMSettings {
            lp_fee: Fee::new(28, 10000),
            shade_dao_fee: Fee::new(2, 10000),
            shade_dao_address: Contract {
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
        admin_auth: shadeswap_shared::Contract { address: Addr::unchecked("admin"), code_hash: "".to_string() }
    })
}

fn pagination(start: u64, limit: u8) -> Pagination {
    Pagination { start, limit }
}
