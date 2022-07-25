#[cfg(test)]
pub mod tests {
    use crate::contract::init;
    use crate::contract::EPHEMERAL_STORAGE_KEY;
    use crate::state::config_read;
    use crate::state::Config;
    use crate::state::CurrentSwapInfo;

    use crate::contract::handle;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use shadeswap_shared::custom_fee::Fee;
    use shadeswap_shared::{
        fadroma::{
            from_slice,
            scrt::{
                from_binary,
                testing::{mock_dependencies, mock_env, MockApi, MockStorage},
                to_binary, Api, Env, Extern, HumanAddr, Querier, StdError,
                StdResult, Storage, Uint128,
            },
            scrt_link::{ContractInstantiationInfo, ContractLink},
            scrt_storage::{load, save},
            secret_toolkit::snip20::{self, Balance},
            Coin, CosmosMsg, Empty, InitResponse, QuerierResult, QueryRequest, WasmMsg,
            WasmQuery,
        },
        msg::{
            amm_pair::QueryMsgResponse as AMMPairQueryMsgResponse,
            factory::QueryResponse as FactoryQueryResponse,
            router::{HandleMsg, InitMsg, InvokeMsg},
        },
        TokenAmount, TokenPair, TokenType,
    };

    pub const FACTORY_ADDRESS: &str = "FACTORY_ADDRESS";
    pub const PAIR_CONTRACT_1: &str = "PAIR_CONTRACT_1";
    pub const PAIR_CONTRACT_2: &str = "PAIR_CONTRACT_2";
    pub const CUSTOM_TOKEN_1: &str = "CUSTOM_TOKEN_1";

    #[test]
    fn ok_init() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let env = mkenv("admin");
        let config = mkconfig(env.clone(), 0);
        assert!(init(deps, env.clone(), (&config).into()).is_ok());
        assert_eq!(config, config_read(deps)?);
        Ok(())
    }

    #[test]
    fn swap_native_for_snip20_tokens_ok() -> StdResult<()> {
        let (init_result, mut deps) = init_helper(100);
        let mut env = mkenv("admin");

        env.message.sent_funds = vec![Coin {
            denom: "uscrt".into(),
            amount: Uint128(10),
        }];

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let result = handle(
            &mut deps,
            env,
            HandleMsg::SwapTokensForExact {
                offer: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(10),
                },
                expected_return: None,
                path: vec![HumanAddr("token_addr".into())],
                recipient: None,
            },
        )
        .unwrap();

        assert!(result.messages.len() > 0);
        let result: Option<CurrentSwapInfo> = load(&deps.storage, EPHEMERAL_STORAGE_KEY)?;
        match result {
            Some(info) => {
                assert_eq!(
                    info.amount,
                    TokenAmount {
                        token: TokenType::NativeToken {
                            denom: "uscrt".into(),
                        },
                        amount: Uint128(10),
                    }
                );

                assert_eq!(info.paths, vec![HumanAddr("token_addr".into())]);
            }
            None => panic!("Ephemeral storage should not be empty!"),
        }

        Ok(())
    }

    /*#[test]
    fn swap_snip20_native_for_tokens_ok() -> StdResult<()> {
        let (init_result, mut deps) = init_helper(100);
        let mut env = mkenv("admin");

        env.message.sent_funds = vec![Coin {
            denom: "uscrt".into(),
            amount: Uint128(10),
        }];

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let result = handle(
            &mut deps,
            env,
            snip20::SwapTokensForExact {
                offer: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(10),
                },
                expected_return: None,
                path: vec![HumanAddr("token_addr".into())],
            },
        )
        .unwrap();

        assert!(result.messages.len() > 0);
        let result: Option<CurrentSwapInfo> = load(&deps.storage, EPHEMERAL_STORAGE_KEY)?;
        match result {
            Some(info) => {
                assert_eq!(
                    info.amount,
                    TokenAmount {
                        token: TokenType::NativeToken {
                            denom: "uscrt".into(),
                        },
                        amount: Uint128(10),
                    }
                );
                assert_eq!(
                    info.paths,
                    vec![HumanAddr("token_addr".into())]
                );
            }
            None => panic!("Ephemeral storage should not be empty!"),
        }

        Ok(())
    }*/

    #[test]
    fn snip20_swap() -> StdResult<()> {
        let (init_result, mut deps) = init_helper(100);
        let mut env = mkenv("admin");

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        save(
            &mut deps.storage,
            EPHEMERAL_STORAGE_KEY,
            &CurrentSwapInfo {
                amount_out_min: Some(Uint128(10)),
                amount: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(10),
                },
                paths: vec![
                    HumanAddr(PAIR_CONTRACT_1.into()),
                    HumanAddr(PAIR_CONTRACT_2.into()),
                ],
                signature: to_binary("this is signature").unwrap(),
                recipient: HumanAddr("recipient".into()),
                current_index: 0,
            },
        )?;

        let result = handle(
            &mut deps,
            mkenv("CUSTOM_TOKEN_1"),
            HandleMsg::Receive {
                from: HumanAddr("recipient".into()),
                msg: Some(
                    to_binary(&InvokeMsg::SwapTokensForExact {
                        expected_return: Some(Uint128(1000)),
                        paths: vec![PAIR_CONTRACT_1.into()],
                        recipient: None,
                    })
                    .unwrap(),
                ),
                amount: Uint128(100),
            },
        );

        match result {
            Ok(info) => {
                println!("{:?}", info.messages);
            }
            Err(err) => {
                let test = err.to_string();
                panic!("{}","Must not return error ".to_string() + &test)
            }
        }

        Ok(())
    }

    #[test]
    fn first_swap_callback_with_one_more_unauthorized() -> StdResult<()> {
        let (init_result, mut deps) = init_helper(100);
        let mut env = mkenv("admin");

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        save(
            &mut deps.storage,
            EPHEMERAL_STORAGE_KEY,
            &CurrentSwapInfo {
                amount_out_min: Some(Uint128(10)),
                amount: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(10),
                },
                paths: vec![
                    HumanAddr(PAIR_CONTRACT_1.into()),
                    HumanAddr(PAIR_CONTRACT_2.into()),
                ],
                signature: to_binary("this is signature").unwrap(),
                recipient: HumanAddr("recipient".into()),
                current_index: 0,
            },
        )?;

        let result = handle(
            &mut deps,
            env,
            HandleMsg::SwapCallBack {
                last_token_out: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(100),
                },
                signature: to_binary("wrong signature").unwrap(),
            },
        );

        match result {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        Ok(())
    }

    #[test]
    fn first_swap_callback_with_one_more_ok() -> StdResult<()> {
        let (init_result, mut deps) = init_helper(100);
        let mut env = mkenv("admin");

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        save(
            &mut deps.storage,
            EPHEMERAL_STORAGE_KEY,
            &CurrentSwapInfo {
                amount_out_min: Some(Uint128(10)),
                amount: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(10),
                },
                paths: vec![
                    HumanAddr(PAIR_CONTRACT_1.into()),
                    HumanAddr(PAIR_CONTRACT_2.into()),
                ],
                signature: to_binary("this is signature").unwrap(),
                recipient: HumanAddr("recipient".into()),
                current_index: 0,
            },
        )?;

        let result = handle(
            &mut deps,
            env,
            HandleMsg::SwapCallBack {
                last_token_out: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(10),
                },
                signature: to_binary("this is signature").unwrap(),
            },
        )
        .unwrap();

        println!("{:?}", result.messages);

        Ok(())
    }

    #[test]
    fn first_swap_callback_with_no_more_ok() -> StdResult<()> {
        let (init_result, mut deps) = init_helper(100);
        let mut env = mkenv("admin");

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        save(
            &mut deps.storage,
            EPHEMERAL_STORAGE_KEY,
            &CurrentSwapInfo {
                amount_out_min: Some(Uint128(10)),
                amount: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(10),
                },
                paths: vec![HumanAddr(PAIR_CONTRACT_1.into())],
                signature: to_binary("this is signature").unwrap(),
                recipient: HumanAddr("recipient".into()),
                current_index: 0,
            },
        )?;

        let result = handle(
            &mut deps,
            env.clone(),
            HandleMsg::SwapCallBack {
                last_token_out: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(10),
                },
                signature: to_binary("this is signature").unwrap(),
            },
        )
        .unwrap();

        assert_eq!(result.messages.len(), 1);

        println!("{:?}", result.messages[0]);
        let test: CosmosMsg<WasmMsg> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: HumanAddr::from(CUSTOM_TOKEN_1),
            callback_code_hash: "hash".into(),
            msg: to_binary(&snip20::HandleMsg::Send {
                recipient: HumanAddr("recipient".into()),
                amount: Uint128(10),
                padding: None,
                msg: None,
            })?,
            send: vec![],
        });
        println!("{:?}", test);
        assert!(result.messages.contains(&CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: HumanAddr::from(CUSTOM_TOKEN_1),
            callback_code_hash: "hash".into(),
            msg: to_binary(&snip20::HandleMsg::Send {
                recipient: HumanAddr("recipient".into()),
                amount: Uint128(10), //This is how much balance the address has
                padding: None,
                msg: None
            })?,
            send: vec![]
        })));
        Ok(())
    }

    #[test]
    fn first_swap_callback_with_no_more_not_enough_return() -> StdResult<()> {
        let (init_result, mut deps) = init_helper(100);
        let mut env = mkenv("admin");

        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        save(
            &mut deps.storage,
            EPHEMERAL_STORAGE_KEY,
            &CurrentSwapInfo {
                amount_out_min: Some(Uint128(100)),
                amount: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(10),
                },
                paths: vec![HumanAddr(PAIR_CONTRACT_1.into())],
                signature: to_binary("this is signature").unwrap(),
                recipient: HumanAddr("recipient".into()),
                current_index: 0,
            },
        )?;

        let result = handle(
            &mut deps,
            env.clone(),
            HandleMsg::SwapCallBack {
                last_token_out: TokenAmount {
                    token: TokenType::NativeToken {
                        denom: "uscrt".into(),
                    },
                    amount: Uint128(10),
                },
                signature: to_binary("this is signature").unwrap(),
            },
        );

        match result {
            Err(StdError::GenericErr { .. }) => {}
            _ => panic!("Must return error"),
        }
        Ok(())
    }

    /*

        //*** */
        #[test]
        fn swap_tokens_for_exact_tokens() -> StdResult<()> {
            Ok(())
        }
    */
    fn mkconfig(env: Env, id: u64) -> Config<HumanAddr> {
        Config::from_init_msg(
            env,
            InitMsg {
                factory_address: ContractLink {
                    address: HumanAddr(String::from(FACTORY_ADDRESS)),
                    code_hash: "Test".to_string(),
                },
                prng_seed: to_binary(&"prng").unwrap(),
                entropy: to_binary(&"entropy").unwrap(),
                viewing_key: None
            },
        )
    }
    fn mkdeps() -> Extern<impl Storage, impl Api, impl Querier> {
        mock_dependencies(30, &[])
    }
    fn mkenv(sender: impl Into<HumanAddr>) -> Env {
        mock_env(sender, &[])
    }

    impl Into<InitMsg> for &Config<HumanAddr> {
        fn into(self) -> InitMsg {
            InitMsg {
                factory_address: self.factory_address.clone(),
                prng_seed: to_binary(&"prng").unwrap(),
                entropy: to_binary(&"entropy").unwrap(),
                viewing_key: None
            }
        }
    }

    fn init_helper(
        contract_bal: u128,
    ) -> (
        StdResult<InitResponse>,
        Extern<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_deps();
        let env = mock_env("admin", &[]);

        let init_msg = InitMsg {
            factory_address: ContractLink {
                address: HumanAddr(String::from(FACTORY_ADDRESS)),
                code_hash: "Test".to_string(),
            },
            prng_seed: to_binary(&"prng").unwrap(),
            entropy: to_binary(&"entropy").unwrap(),
            viewing_key: None
        };

        (init(&mut deps, env, init_msg), deps)
    }

    fn mock_deps() -> Extern<MockStorage, MockApi, MockQuerier> {
        Extern {
            storage: MockStorage::default(),
            api: MockApi::new(123),
            querier: MockQuerier { portion: 2500 },
        }
    }
    struct MockQuerier {
        portion: u128,
    }
    impl Querier for MockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
            match &request {
                QueryRequest::Wasm(msg) => match msg {
                    WasmQuery::Smart { contract_addr, .. } => {
                        println!("{}", contract_addr);
                        match contract_addr.as_str() {
                            FACTORY_ADDRESS => {
                                QuerierResult::Ok(to_binary(&FactoryQueryResponse::GetConfig {
                                    pair_contract: ContractInstantiationInfo {
                                        code_hash: "".to_string(),
                                        id: 1,
                                    },
                                    amm_settings: shadeswap_shared::amm_pair::AMMSettings {
                                        lp_fee: Fee::new(28, 10000),
                                        shade_dao_fee: Fee::new(2, 10000),
                                        shade_dao_address: ContractLink {
                                            address: HumanAddr(String::from("DAO")),
                                            code_hash: "".to_string(),
                                        },
                                    },
                                    lp_token_contract: ContractInstantiationInfo {
                                        code_hash: "".to_string(),
                                        id: 1,
                                    },
                                }))
                            }
                            PAIR_CONTRACT_1 => QuerierResult::Ok(to_binary(
                                &AMMPairQueryMsgResponse::GetPairInfo {
                                    liquidity_token: ContractLink {
                                        address: HumanAddr::from("asd"),
                                        code_hash: "".to_string(),
                                    },
                                    factory: ContractLink {
                                        address: HumanAddr::from("asd"),
                                        code_hash: "".to_string(),
                                    },
                                    pair: TokenPair(
                                        TokenType::CustomToken {
                                            contract_addr: CUSTOM_TOKEN_1.into(),
                                            token_code_hash: "hash".into(),
                                        },
                                        TokenType::NativeToken {
                                            denom: "denom".into(),
                                        },
                                    ),
                                    amount_0: Uint128(100),
                                    amount_1: Uint128(101),
                                    total_liquidity: Uint128(100),
                                    contract_version: 1,
                                },
                            )),
                            CUSTOM_TOKEN_1 => QuerierResult::Ok(to_binary(&IntBalanceResponse {
                                balance: Balance {
                                    amount: Uint128(100),
                                },
                            })),
                            _ => unimplemented!(),
                        }
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }

        fn query<T: DeserializeOwned>(&self, request: &QueryRequest<Empty>) -> StdResult<T> {
            self.custom_query(request)
        }

        fn custom_query<T: serde::Serialize, U: DeserializeOwned>(
            &self,
            request: &QueryRequest<T>,
        ) -> StdResult<U> {
            let raw = match shadeswap_shared::fadroma::to_vec(request) {
                Ok(raw) => raw,
                Err(e) => {
                    return Err(StdError::generic_err(format!(
                        "Serializing QueryRequest: {}",
                        e
                    )))
                }
            };
            match self.raw_query(&raw) {
                Err(sys) => Err(StdError::generic_err(format!(
                    "Querier system error: {}",
                    sys
                ))),
                Ok(Err(err)) => Err(err),
                // in theory we would process the response, but here it is the same type, so just pass through
                Ok(Ok(res)) => from_binary(&res),
            }
        }

        fn query_balance<U: Into<HumanAddr>>(&self, address: U, denom: &str) -> StdResult<Coin> {
            let request = shadeswap_shared::fadroma::BankQuery::Balance {
                address: address.into(),
                denom: denom.to_string(),
            }
            .into();
            let res: shadeswap_shared::fadroma::BalanceResponse = self.query(&request)?;
            Ok(res.amount)
        }

        fn query_all_balances<U: Into<HumanAddr>>(&self, address: U) -> StdResult<Vec<Coin>> {
            let request = shadeswap_shared::fadroma::BankQuery::AllBalances {
                address: address.into(),
            }
            .into();
            let res: shadeswap_shared::fadroma::AllBalanceResponse = self.query(&request)?;
            Ok(res.amount)
        }
    }
    #[derive(Serialize, Deserialize)]
    struct IntBalanceResponse {
        pub balance: Balance,
    }
}
