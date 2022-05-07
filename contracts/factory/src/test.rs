
use shadeswap_shared::amm_pair::Fee;
use shadeswap_shared::amm_pair::AMMSettings;
use shadeswap_shared::msg::factory::InitMsg;
pub use shadeswap_shared::{
    fadroma::{
        scrt_addr::Canonize,
        scrt_link::{ContractLink, ContractInstantiationInfo},
        scrt::{
            from_binary,
            testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
            to_binary, Api, Binary, Env, HandleResponse, HumanAddr, Querier, StdError,
            StdResult, Storage, Uint128, Extern
        },
        scrt_storage::{load, save},
    },
    msg::factory::{ QueryResponse},
    Pagination, TokenPair, TokenType,
};


use crate::state::Config;

#[cfg(test)]
pub mod test_contract {
    use crate::contract::create_signature;
use crate::contract::EPHEMERAL_STORAGE_KEY;
use crate::contract::handle;
    use crate::contract::query;
    use crate::state::PAGINATION_LIMIT;
use super::*;
    use crate::contract::create_pair;
    use crate::contract::init;
    use crate::state::config_read;
    use crate::state::config_write;
    use shadeswap_shared::amm_pair::AMMPair;
    use shadeswap_shared::msg::factory::HandleMsg;
    use shadeswap_shared::msg::factory::QueryMsg;
    pub use shadeswap_shared::{
        fadroma::{
            scrt_addr::Canonize,
            scrt_link::{ContractLink, ContractInstantiationInfo},
            scrt::{
                from_binary,
                testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
                to_binary, Api, Binary, Env, HandleResponse, HumanAddr, Querier, StdError,
                StdResult, Storage, Uint128,
            },
            scrt_storage::{load, save},
        },
        msg::factory::{ QueryResponse},
        Pagination, TokenPair, TokenType,
    };

    #[test]
    fn init_ok() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let env = mkenv("admin");
        let config = mkconfig(0);
        //assert!(init(deps, env, (&config).into()).is_ok());
        //assert_eq!(config, config_read(deps)?);
        Ok(())
    }


    #[test]
    fn get_set_config_ok() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let env = mkenv("admin");
        init(deps, env.clone(), (&mkconfig(0)).into());

        let new_config = mkconfig(5);
        handle(
            deps,
            env,
            HandleMsg::SetConfig { 
                pair_contract: Some(new_config.pair_contract.clone()),
                amm_settings: Some(new_config.amm_settings.clone()),
                lp_token_contract: Some(new_config.lp_token_contract.clone())
            }
        )
        .unwrap();

        let response: QueryResponse = from_binary(&query(deps, QueryMsg::GetConfig {})?)?;
        let compare: QueryResponse = (&new_config).into();
        assert_eq!(compare, response);
        Ok(())
    }

    #[test]
    fn register_amm_pair_ok() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let env = mkenv("admin");
        let config = mkconfig(0);

        config_write(deps, &config)?;

        let signature = create_signature(&env)?;
        save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;

        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into(),
            },
            TokenType::NativeToken {
                denom: "test1".into(),
            },
        );

        handle(
            deps,
            env,
            HandleMsg::RegisterAMMPair { 
                pair: pair.clone(),
                signature,
            },
        )?;

        let result: Option<Binary> = load(&deps.storage, EPHEMERAL_STORAGE_KEY)?;
        match result {
            None => {}
            _ => panic!("Ephemeral storage should be empty!"),
        }

        Ok(())
    }

    #[test]
    fn create_pair_ok() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let env = mkenv("admin");
        let config = mkconfig(0);
        //assert!(init(deps, env, (&config).into()).is_ok());

        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "diff".into(),
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into(),
            },
        );

        let result = create_pair(deps, mkenv("sender"), pair, to_binary(&"entropy").unwrap());
        //let error: StdError = result.unwrap_err();
        print!("BOPOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO");
        //print!("{}",error);
        assert!(result.is_ok());
        Ok(())
    }
    
    #[test]
    fn add_amm_pairs() {
        let ref mut deps = mkdeps();
        let config = mkconfig(0);
        let env = mkenv("admin");

        init(deps, env.clone(), (&config).into()).unwrap();

        let mut amm_pairs: Vec<AMMPair<HumanAddr>> = vec![];

        for i in 0..5 {
            amm_pairs.push(AMMPair {
                pair: TokenPair::<HumanAddr>(
                    TokenType::CustomToken {
                        contract_addr: format!("token_0_addr_{}", i).into(),
                        token_code_hash: format!("token_0_hash_{}", i),
                    },
                    TokenType::CustomToken {
                        contract_addr: format!("token_1_addr_{}", i).into(),
                        token_code_hash: format!("token_1_hash_{}", i),
                    },
                ),
                address: format!("pair_addr_{}", i).into(),
            });
        }

        handle(
            deps,
            env,
            HandleMsg::AddAMMPairs {
                amm_pair: amm_pairs.clone()[0..].into(),
            },
        )
        .unwrap();

        let result = query(
            deps,
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

    /*
    #[test]
    fn increment() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can increment
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Increment {};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // not anyone can reset
        let unauth_env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let res = handle(&mut deps, unauth_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_env = mock_env("creator", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let _res = handle(&mut deps, auth_env, msg).unwrap();

        // should now be 5
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }*/
}

pub mod test_state {
    use shadeswap_shared::amm_pair::AMMPair;

    use crate::state::{save_amm_pairs_count, load_amm_pairs_count, save_amm_pairs, load_amm_pairs, generate_pair_key};

    use super::*;

    fn swap_pair<A: Clone>(pair: &TokenPair<A>) -> TokenPair<A> {
        TokenPair(pair.1.clone(), pair.0.clone())
    }

    #[test]
    fn generate_pair_key_ok() -> StdResult<()> {
        fn cmp_pair<S: Storage, A: Api, Q: Querier>(
            deps: &Extern<S, A, Q>,
            pair: TokenPair<HumanAddr>,
        ) -> StdResult<()> {
            let stored_pair = pair.canonize(&deps.api)?;
            let key = generate_pair_key(&stored_pair);

            let pair = swap_pair(&pair);

            let stored_pair = pair.canonize(&deps.api)?;
            let swapped_key = generate_pair_key(&stored_pair);

            assert_eq!(key, swapped_key);

            Ok(())
        }

        let ref deps = mkdeps();

        cmp_pair(
            deps,
            TokenPair(
                TokenType::CustomToken {
                    contract_addr: HumanAddr("first_addr".into()),
                    token_code_hash: "13123adasd".into(),
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("scnd_addr".into()),
                    token_code_hash: "4534qwerqqw".into(),
                },
            ),
        )?;

        cmp_pair(
            deps,
            TokenPair(
                TokenType::NativeToken {
                    denom: "test1".into(),
                },
                TokenType::NativeToken {
                    denom: "test2".into(),
                },
            ),
        )?;

        cmp_pair(
            deps,
            TokenPair(
                TokenType::NativeToken {
                    denom: "test3".into(),
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("third_addr".into()),
                    token_code_hash: "asd21312asd".into(),
                },
            ),
        )?;

        Ok(())
    }

    #[test]
    fn store_and_get_amm_pairs_ok() {
        let ref mut deps = mkdeps();
        let mut amm_pairs: Vec<AMMPair<HumanAddr>> = vec![];
        amm_pairs.push(AMMPair {
            pair: TokenPair::<HumanAddr>(
                TokenType::CustomToken {
                    contract_addr: format!("token_0_addr_{}", 0).into(),
                    token_code_hash: format!("token_0_hash_{}", 0),
                },
                TokenType::CustomToken {
                    contract_addr: format!("token_1_addr_{}", 0).into(),
                    token_code_hash: format!("token_1_hash_{}", 0),
                },
            ),
            address: format!("pair_addr_{}", 0).into(),
        });
        save_amm_pairs(deps, amm_pairs.clone()).unwrap();
        let result = load_amm_pairs(deps, pagination(0, 1)).unwrap();

        //Check Count was updated
        assert_eq!(1, load_amm_pairs_count(&mut deps.storage).unwrap());

        //Check number of result was returned
        assert_eq!(1, result.len());

        //Match result
        assert_eq!(amm_pairs[0], result[0]);
    }

    #[test]
    fn save_and_load_amm_pairs_count_ok() {
        let ref mut deps = mkdeps();
        save_amm_pairs_count(&mut deps.storage, 1).unwrap();
        assert_eq!(1, load_amm_pairs_count(&mut deps.storage).unwrap());
        assert_ne!(2, load_amm_pairs_count(&mut deps.storage).unwrap())
    }
}

fn mkconfig(id: u64) -> Config<HumanAddr> {
    Config::from_init_msg(InitMsg {
        pair_contract: ContractInstantiationInfo {
            id,
            code_hash: "2341586789".into(),
        },
        amm_settings: AMMSettings {
            lp_fee: Fee::new(28, 10000),
            shade_dao_fee: Fee::new(2, 10000),
            shade_dao_address: ContractLink {
                address: HumanAddr(String::from("CALLBACKADDR")),
                code_hash: "Test".to_string()
            },
        },
        lp_token_contract: ContractInstantiationInfo { 
            id,
            code_hash: "123".into()
        },
        prng_seed: to_binary(&"prng").unwrap()
    })
}

fn mkdeps() -> Extern<impl Storage, impl Api, impl Querier> {
    mock_dependencies(30, &[])
}

fn mkenv(sender: impl Into<HumanAddr>) -> Env {
    mock_env(sender, &[])
}

fn pagination(start: u64, limit: u8) -> Pagination {
    Pagination { start, limit }
}

impl Into<InitMsg> for &Config<HumanAddr> {
    fn into(self) -> InitMsg {
        InitMsg {
            pair_contract: self.pair_contract.clone(),
            amm_settings: AMMSettings {
                lp_fee: Fee::new(28, 10000),
                shade_dao_fee: Fee::new(2, 10000),
                shade_dao_address: ContractLink {
                    address: HumanAddr(String::from("CALLBACKADDR")),
                    code_hash: "Test".to_string()
                }
            },
            lp_token_contract: self.lp_token_contract.clone(),
            prng_seed: to_binary(&"prng").unwrap()
        }
    }
}

impl Into<QueryResponse> for &Config<HumanAddr> {
    fn into(self) -> QueryResponse {
        QueryResponse::GetConfig { 
            pair_contract: self.pair_contract.clone(),
            amm_settings: self.amm_settings.clone(),
            lp_token_contract: self.lp_token_contract.clone()
        }
    }
}